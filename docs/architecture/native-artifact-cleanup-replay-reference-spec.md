# Replay family: runtime Native reference lifetime

## Source boundary

This is test-adapter lifetime cleanup around OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, not a new slicing pipeline.
`src/libslic3r/GCode.cpp:2030+` (`GCode::do_export`) and `:2461+`
(`_do_export`) own complete G-code emission, including the generator line
at `:2576` and serialized configuration in `append_full_config`, `:5591-5643`.
`GCode/GCodeProcessor.hpp:905,914-915` owns full file/buffer processing and
finalization. These boundaries require retaining executable commands, timing,
configuration and statistics when comparing streams.

The Rust destination is `ares-cli/tests/orca_parity/replay_tests.rs` and its
exclusive `runner/native_replay_reference_migration.rs` helper. Existing `runner::OrcaRunner`
and `slice_case` remain the Native adapter and cache owner. Existing
`replay.rs` and `artifacts.rs` continue exercising public `ares_core::slice_project`
and `golden::compare_ordered_bytes_generator_only` without behavior changes.
The frozen fixture-copy scaffold is replaced only in the replay test setup.

## Inputs and ownership

Both inputs remain byte-identical:

| Input | SHA-256 |
| --- | --- |
| `tests/parity/replay/ender3-classic.3mf` | `abd8882f86e789f727fe362a11e3b57eb52e846c712b7a0b102b0a4fe3d05bc3` |
| `tests/parity/orca_cli_ender3.3mf` | `7fb06c23baed9c8e06757cc0f7128c827e0b232eaa2655af687e4708014037c5` |

The handoff's original input is the latter existing file; there is no
`tests/parity/replay/ender3.3mf`. Classic changes only the original project's
effective `wall_generator` to `classic`, with JSON whitespace reserialization.
Both current positive replay cases require complete stream parity.

Writable scope: replay tests and exclusive helper, replay README and this spec.
`runner.rs` adds only this helper's module registration: placing it under the
runner keeps private work paths and `ExportedCase` construction private.
Production, `replay.rs`, shared caches, `.config`, KSR and
`ksr_fdmtest_v4/ordered_bytes_tests.rs` remain untouched.

`ender3-classic.orca.gcode` remains temporarily for KSR's independent include
consumer (`ordered_bytes_tests.rs:63`). Replay no longer reads it.
Audit also found `ares-wasm/tests/encode_roundtrip.rs:28,163` actively reading
`ender3.orca.gcode` for CONFIG_BLOCK values and browser round-trip cases.
After this migration, `encode_roundtrip.rs` is the original product's sole
remaining consumer, in the web domain. Deletion of that 4,467-line product is
**DEFERRED until this reader migrates to runtime Native**. The replay README must record this remaining consumer and
deletion dependency. Neither remaining product is a replay fallback after this
migration. No WASM or KSR consumer is migrated or disabled in this slice.

## Runtime contract

Before creating a genuine-reference replay fixture, call existing
`OrcaRunner::from_env`. Required mode fails on missing dependencies before
cache access; optional mode emits shared visible SKIPPED and returns before
creating replay inputs, reference streams or comparison artifacts.
Write unchanged input bytes into the runner work directory with their source
filename, then pass an `ExportedCase` through existing `slice_case`.
The existing producer/resource identity, input SHA, command recipe, receipt,
lock, staging, replay validation and publication remain authoritative.
Copy the returned raw Native reference into the temporary replay layout
`ender3.3mf` / `ender3/plate_1.gcode`; invoke the actual replay entrypoint.
No second cache, prewarming, Ares expectation or historical Native fallback.

Transport controls that do not require a genuine reference stay runnable
without live slicing: empty/missing inventory, missing reference, invalid
artifact destination and missing artifact destination. Preserve their error
assertions and independent malformed paths. Required-mode replay entrypoint
gating already exists and is not a newly discovered failure.
The explicitly offline child retains only its existing required-env removal.

Preserve all eleven original names and semantics:

- `explicit_empty_inventory_fails`
- `explicit_missing_reference_fails`
- `explicit_unwritable_artifact_root_fails`
- `explicit_divergent_orca_output_fails`
- `unchanged_classic_replays_byte_identical_with_complete_paired_bytes_and_hashes`
- `original_orca_project_matches_after_classic_convergence`
- `explicit_unreadable_project_fails_and_preserves_reference`
- `explicit_unwritable_report_fails_after_saving_paired_bytes`
- `explicit_missing_inventory_fails`
- `explicit_request_without_artifact_destination_fails`
- `unset_replay_is_explicitly_offline_not_a_comparison`

Retain the deliberate single `M104 S` to `M104 S9` mutation, unreadable project,
saved paired raw bytes and hashes, complete manifest/status/report assertions,
actual public-core output assertion and validated generator-only comparator.
The legacy replay manifest still honestly reports producer provenance unknown;
the producer-bound runtime cache receipt is separate evidence, not a fabricated
legacy manifest attestation. Do not normalize timing, M73, CONFIG_BLOCK or footer.

## Migration evidence and acceptance

RED is a controlled optional child running the existing classic positive test
without Native dependencies. Its policy assertion requires visible SKIPPED and
no fabricated fixture/reference/comparison artifacts. Current frozen-copy
setup instead executes a comparison and saves paired streams; this is a real
runtime policy failure, not a compiler failure or a claim that required gating
was absent. Use an external artifact root to retain this evidence.

GREEN migrates both copy sites. Required cold/hot executions retain all eleven
contracts; validate both inputs' cache receipts, raw Native bytes and miss/hit
behavior. Missing-dependency required controls fail before a warm cache can
mask the dependency; optional genuine-reference tests visibly skip without
fabrication, while transport and offline controls retain their assertions.
Capture fresh paired seeded/official results and shared Ares raw hashes,
then workspace, clippy `-D warnings`, fmt and wasm checks.
Any current Native divergence is reported with complete evidence; comparator, assertions and hashes are not weakened.

Runtime Native output and diagnostics are not committed. Deferred behavior:
KSR/classic product removal, WASM original product consumer migration, production
replay provenance/schema changes, broader inventory and all-domain parity.
