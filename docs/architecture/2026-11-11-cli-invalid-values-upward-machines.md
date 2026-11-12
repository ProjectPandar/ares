# CLI invalid-values exit: preserve the reached upward-machine snapshot

## Boundary and authority

This ports the `OrcaSlicer.cpp` CLI adapter's upward-machine report ordering.
It does not introduce an Ares-owned slicing pipeline.

Authority: OrcaSlicer v2.5.0-beta source
`345efec64c7b95a05ce5cf96df0dde66a158835c`. The official binary and seeded
replay are evaluated independently.

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

## Fixture and public CLI coverage

The fixture derives from the stock H2D project
`tests/parity/master-extruder-id/stock-right-only.3mf`. Only the existing
`use_firmware_retraction` option changes to `"1"` while the wipe vector remains
enabled. Archive and ZIP-member payload provenance identifies the input.

Both Native modes return `-18` with nonempty upward metadata, complete
stdout/stderr/result.json and no final G-code. Native output defines the
comparison contract.

The public CLI E2E exercises ordinary and `--dump-moves` entries through their
existing common report-aware path. It compares entire result.json bytes and
all fields with the existing normalization of only `prepare_time` and
`export_time`, because those upstream members are uninitialized at this exit.
It asserts Ares retains deterministic zeros and the helper changes no other
field. Machine arrays are not normalized, and no other mask applies. Exit,
stdout/stderr and absent final G-code/move CSV are checked independently. Each
invocation retains a repeatable, verifiable artifact.

Failure model: missing nonempty header; wrong machine source or array order;
report leakage before post-process/compatibility rejection; changed error
code/stdout/stderr/other JSON fields; empty-key emission; hidden timing/field
filters; missing Native execution mislabeled as parity; changed `-100` behavior.
No implementation-following unit tests substitute for public CLI E2E coverage.

## Upstream uninitialized early-exit timings

Both Native modes return `-18` with
`["Bambu Lab H2D Pro 0.4 nozzle"]`. Observed official timings were0/0, while
seeded `prepare_time` was140737174491600 and `export_time` was140737488334736.
The timing fields cannot be assumed to contain zero.

Source: `OrcaSlicer.cpp:258–259` declares the two `size_t` members without
initializers; `:1387` default-initializes stack `sliced_info_t sliced_info;`;
`:555–556` serializes them on `-18` before the prepare-time assignment at
`:6294`, after the validation exit at `:4277`. These observed values are an
upstream uninitialized-read defect, not useful durations to port. Ares keeps
its existing deterministic zero values for this early failure.

The upstream uninitialized reads require the existing
`native_wall_clock::result_comparison_bytes` exception for **only**
`prepare_time` and `export_time`. The E2E names these two fields and must
assert its normalized Native JSON equals the original JSON with exactly those
two values set to0, and compare the remaining complete serialized bytes
strictly against Ares. This checks no extra helper mask can silently apply.
All fields remain present and recorded in raw artifacts; the upward array,
return code, messages, configuration metadata, warnings, strict flag, and
formatting retain the full comparison contract. No Native patch, Ares
uninitialized-value emulation, extra normalization or changed error behavior.

The input SHA256 is
`5f7adc3f35fdb9a06ee98964178990cd41906ee1610ef7e4e0114f716daced29`.

## Existing -100 coverage

`cli_upward_machine_result::cli_custom_gcode_failure_preserves_nonempty_upward_machine_header`
already covers the existing writer in both Native modes. That late custom-G-code
`-100` path receives the snapshot. The `-18` defect is loaded-config validation
preceding publication. The two exits share transport. Keep the existing E2E's
assertions as a regression control without implementing another `-100` route.

Deferred: broader CLI result metadata, downward discovery/checking, new error
families, load-settings/profile-controller sequencing, machine-switch/update
orchestration, validation behavior, grouping/G-code changes and cross-platform
runtime certification. This slice does not certify all CLI entry variants or
workspace-wide Native parity.
