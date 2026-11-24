# Parity artifact write contract

## Scope and failure modes

This change is confined to the `orca_parity` test executable. It does not change the upstream OrcaSlicer invocation or any comparison, slicer, or public API behavior.

In `crates/ares-cli/tests/orca_parity/runner/lifecycle_tests.rs:10–91`, the call must return within two seconds and before the descendant finishes. The test also requires a timeout classification, direct-child reap, and immutable stdout/stderr snapshots containing the early binary bytes but not the descendant's late output. It checks that the live capture does contain that output after the helper exits, and that a timeout cannot count as an input rejection. The direct child has a 500 ms deadline and the descendant sleeps for three seconds.

The failure surface is:

- Fixture/launch: temporary directories, helper mode files, executable lookup, command metadata and live file creation can fail. A bad working directory or spawn failure can keep the helper from writing markers. A delayed or unscheduled child may never flush its early bytes before the deadline.
- Process lifecycle: a wait error, a premature child exit, late deadline detection, slow or unsuccessful kill/reap, or waiting for descendant-held output handles can change the error kind, overrun the bound, or leave the child unreaped. The descendant can fail to start, produce its late output, or write the marker in time for the test's cleanup check. The marker and elapsed assertion are separate: a return can miss the bound even with the descendant still running.
- Capture: open, metadata and read failures, or an incorrect length boundary, can leave missing, short, or wrong snapshots. Snapshotting must not wait for output EOF or absorb bytes written after return. An absent or corrupt `lifecycle.json`, `status.json` or `timings.json` is an artifact failure, not a valid command outcome.
- Publication: a name collision, missing directory, denied permissions, exhausted space or quota, or an I/O error during create or write can leave a missing or partial artifact. A concurrent observer can see a file before the write completes. A device sync can fail or stall even after a successful write. Loss of the host before writeback can leave missing or partial files; callers must not treat them as a complete result.
- Error reporting: capture errors must not replace a primary timeout or wait error. The existing collision check at `runner/lifecycle_tests.rs:125–148` exercises this. The runner reports other artifact I/O failures rather than silently returning success.

In the observed timeout failure, kill/reap and snapshots finished at about 517 ms. The call took about 2.766 s, with about 2.248 s in `finish`; the descendant had not finished when the call returned. The gap between the stdout and stderr snapshot file mtimes is consistent with a stall in the first file's `sync_all`; the measurements do not exclude a scheduling stall in the same interval. See `runner/command.rs:135–190` for deadline, shutdown and snapshot timing, `runner/command.rs:191–219` for finalization timing, and `runner/stages.rs:247–299` for error precedence and the three writes.

## Consumers and durability

Paths below are relative to `crates/ares-cli/tests/orca_parity/`. This inventories the direct call sites, including the unqualified calls inside `artifacts.rs`, and the readers of runner captures.

| Writer | Consumers | Needs device sync? |
| --- | --- | --- |
| `artifacts.rs:203–222` | Case inputs, Native and Ares output, diagnostics and the case manifest. Readers inspect returned case paths or completed runs; the manifest lists byte counts and hashes. | No. The manifest is written after its listed files. |
| `replay.rs:115–118` | The replay summary is read after the harness returns or exits. | No. Process exit is not host crash recovery. |
| `bridge_over_infill_zero_density.rs:88–122`; `option_coverage/clearance/geometry.rs:156–204`; `option_coverage/initialized_object_id.rs:19–23` | Prepared project inputs are read by the later command invocation; comparison streams and diagnostics are read by the test or retained for inspection. | No. These children are started after the input write returns. |
| `option_coverage/multitool_ramming_footer.rs:30`; `option_coverage/owner_pair_tests.rs:33–119`; `option_coverage/printable_height/loaded.rs:72–115`; `option_coverage/printable_height/scene.rs:123–174` | Test-created input files, comparison streams, and case or aggregate results. Each child uses an already written input, and output is read after writing. | No. No host-crash replay depends on these particular files. |
| `runner/compatible_regions/global_pattern.rs:73–102`; `runner/compatible_regions/solid_pattern.rs:43–45`; `runner/compatible_regions/top_pattern.rs:45–47`; `runner/compatible_regions/tool_templates/layer_context.rs:60–62`; `runner/compatible_regions/tool_templates/timelapse_context.rs:59–61`; `runner/stock_validation/project_value_out_of_range.rs:27–43` | Same-attempt project inputs, command streams and result documents. Reads occur after writing or in later report inspection. | No. Closing the files before launching a consumer is sufficient. |
| `runner/stages.rs:256–288` | `stdout.bin`, `stderr.bin` and `status.json` are read after the command wrapper returns by lifecycle, path and stage tests and by cache producers. `runner/command_cache.rs:110–161` reads the completed capture before publishing a separate cache entry. `runner/cache/cli_result/create.rs:67–90` reads the completed successful capture; `runner/cache/processor_diagnostic.rs:111–162` copies completed files into cache staging. | No. These reads use closed files. Cache publication and integrity checks are separate; syncing these source files alone does not make the published cache durable. |
| `runner/stages.rs:285–288` | An external test controller also scans `status.json` while tests run. It ignores missing or incompletely parsed JSON on a scan; valid JSON identifies a signal and is hashed with the command record. | No. This reader needs complete JSON visible on the running host. The current `sync_all` happens after writing and cannot prevent an observer from seeing a partial write. |

The artifacts must remain readable after a **child** timeout, crash or nonzero exit while the harness and filesystem are still alive. There is no consumer that recovers an interrupted verification after host failure. Even the current call does not sync parent directories or make a multi-file publication atomic. Close-then-open visibility, write error propagation and post-crash durability are distinct contracts.

## Change and verification

Remove only `sync_all` from `artifacts::write` (`artifacts.rs:41–49`). Keep `create_new`, `write_all`, synchronous return after the file is closed, path-qualified errors, bytes and ordering. Do not add an async writer, retries, or a durability flag. Leave `artifacts::overwrite`, production output writes, and cache publication unchanged. This removes a blocking device flush from the timeout return path; it does not promise that ordinary filesystem writes or scheduling are bounded.

Keep the lifecycle fixture and its assertions unchanged. Verify the lifecycle tests (including the existing collision check), matched inventories and paired `runner::` outcomes in both reference modes, and formatting and native and WebAssembly lint checks. The historical failure is the regression witness; a passing short run does not recreate host-wide filesystem contention or establish a latency ceiling.
