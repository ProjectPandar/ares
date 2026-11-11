# CLI invalid-values exit: preserve the reached upward-machine snapshot

## Boundary and authority

This is the source-cited `OrcaSlicer.cpp` CLI adapter metadata port assigned by
the coordinator, not a new Ares slicing pipeline. Base is
`de07bfa9e4f17806398b7f5152cad35321b21cb7`; the unrelated nozzle branch stays
unchanged at `e87c09966642bb0c3c31c2813a1d0d59d2681795`.

Authority: OrcaSlicer v2.5.0-beta source
`345efec64c7b95a05ce5cf96df0dde66a158835c`, with the frozen official binary
and separately approved seeded replay evaluated independently.

- `src/OrcaSlicer.cpp:2820–2851` fetches the original system machine's
  `upward_compatible_machine`, retaining the existing source-selection rules.
- `:2980–3004` may reject process/machine compatibility before report commit;
  `:3005` assigns the selected ordered vector to `sliced_info.upward_machines`.
- `:4272–4278` calls `m_print_config.validate(true)` and writes/exits
  `CLI_INVALID_VALUES_IN_3MF` (`-18`) when the validity map is nonempty.
  The owning libslic3r boundary is `PrintConfig.cpp`'s
  `DynamicPrintConfig::validate`; its validation algorithm is not changed here.
- `record_exit_reson`, `:535–586`, particularly `:550–551`, writes the
  nonempty upward vector to `j["upward_compatible_machine"]`. Empty vectors
  remain omitted. The remaining result fields and failure channels are intact.

Collection, snapshot commit and validation are distinct stages. The selected
original-machine vector has already reached the report at the `-18` exit;
reading final printer settings after failure is not an equivalent operation.

## Rust destination and non-negotiable decisions

`crates/ares-core/src/project_slice/state.rs::prepare_project_source` is the
bounded destination: the existing `CliUpwardMachine` report assignment must
run after existing post-process/process-compatibility rejection and before
`ProjectSettings::validate_loaded_config` can return `InvalidValuesInProject`.
Reuse the existing `UpwardMachineSource::select` and supplied byte-only source
input. Do not implement another selector, resource lookup, report structure,
validation algorithm, exporter, profile-refresh controller or error family.

The CLI's existing `compatible_machine.rs` resource loading,
`failure.rs::CliFailure::with_upward_report`, `failure/report.rs` and
`result_json.rs` already transport and conditionally serialize this field.
Keep those mechanisms and their platform boundaries; move only the existing
snapshot publication to the correct upstream stage. This corrects the prior
compatibility shell's assumption that all loaded-config failures precede the
snapshot. It does not replace source ordering with an Ares-owned pipeline.

Core remains platform-neutral on browser WASM, Windows, macOS and Linux:
no filesystem, environment, terminal, OpenGL, new dependencies, unsafe code,
new public identity/report API or mutable global state. Existing Linux-only
result-file emission remains in the CLI. Preserve vector order/multiplicity,
nonempty-only emission and all existing source-selection behavior.

## Native-first and test-first plan

1. Construct a retained input from the main stock H2D project
   `tests/parity/master-extruder-id/stock-right-only.3mf`, changing only the
   existing `use_firmware_retraction` option to `"1"` while its wipe vector
   remains enabled. Record archive and ZIP-member payload provenance.
2. Run original bytes through both frozen Native modes before authoring the
   regression. Require actual `-18`, nonempty upward metadata, complete
   stdout/stderr/result.json and no final G-code. If admission or another
   premise fails, retain it separately and stop calling that attempt evidence
   of the missing field. Native output is authority, not a hand-derived JSON.
3. Write a public CLI E2E at the agreed result.json seam, then execute it on
   the unchanged implementation. Exercise ordinary and `--dump-moves` entries
   through their existing common report-aware path. Compare entire result.json
   bytes and all fields, including zero timing fields on this early exit;
   do not use the wall-clock adaptation helper, delete fields or normalize
   machine arrays. Independently check exit/stdout/stderr and absent final
   G-code/move CSV. Keep a repeatable, verifiable artifact per invocation.
4. Only after the real failure, relocate the existing report assignment;
   rerun the exact test and existing early/late failure controls in both modes.
   Run fmt, locked workspace/all-target Clippy with warnings denied, fresh
   core+WASM checks and changed Rust LOC checks, then submit one separate RR.

Failure model: missing nonempty header; wrong machine source or array order;
report leakage before post-process/compatibility rejection; changed error
code/stdout/stderr/other JSON fields; empty-key emission; hidden timing/field
filters; missing Native execution mislabeled as parity; changed `-100` behavior.
No implementation-following unit tests substitute for public CLI E2E evidence.

## The reported -100 analogue: already covered, not implementation scope

Coordinator clarification: the earlier W3 classification note has no separate
input/evidence and refers to main's existing
`cli_upward_machine_result::cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`
and existing writer. `docs/architecture/cli-upward-machine-result-green-candidate.md`
records its prior official/seeded coverage. That late custom-G-code `-100`
path already receives the snapshot; the newly reported `-18` defect is the
loaded-config validation ordering ahead of publication. They share transport
but are not an additional unported `-100` writer defect. Reuse that existing
E2E as a regression control, without altering its assertions or implementing
another `-100` route.

Deferred: broader CLI result metadata, downward discovery/checking, new error
families, load-settings/profile-controller sequencing, machine-switch/update
orchestration, validation behavior, grouping/G-code changes and cross-platform
runtime certification. This slice does not certify all CLI entry variants or
workspace-wide Native parity. The nozzle branch and its pending B54 rebase
remain a separate workstream.
