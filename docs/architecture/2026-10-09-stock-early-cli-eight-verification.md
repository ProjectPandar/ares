# #30: eight runtime stock early CLI rejection cases

## ARD: phase-owned error classification

This is the remaining eight-case slice of residual spec `02ffadb9c`
and [the preparation spec](../superpowers/specs/2026-10-09-stock-early-cli-eight.md),
not an Ares-owned pipeline feature.

- `OrcaSlicer.cpp:5710–5724` rejects zero eligible instances with
  `CLI_NO_SUITABLE_OBJECTS (-50)` before `Print::apply`.
  `Model.cpp:3423–3469::ModelInstance::calc_print_volume_state`,
  `Model::update_print_volume_state/count_inside_instances` and `BuildVolume`
  remain the owning eligibility boundary.
- Rust destination: the existing `project_slice/state.rs` producer in
  `prepare_loaded_project_slice_with_validation`. Change only its existing
  `count_inside_instances == 0` error variant from `InvalidInput` to the
  existing `NoSuitableObjects`. Condition, error text, phase, geometry and
  placement are unchanged. No new Core process/filesystem contract is added.
- Existing `ares-cli/src/failure.rs` already transports `NoSuitableObjects` as
  exit `206`, upstream code `-50`, empty stderr and the upstream result bytes.
  Do not add CLI string matching or duplicate transport.
- `OrcaSlicer.cpp:1682–1690` owns loaded embedded-postprocess rejection `-19`
  before slicing. Four Prusa cases match the existing implementation without
  a production change.

Non-negotiable: unchanged runtime stock selection/export/cache; identical
runtime-exported input bytes for both consumers; no committed Native input or
output products; full exit/stdout/stderr/result comparison. Only the shared
`native_wall_clock::result_comparison_bytes` exempts `prepare_time` and
`export_time`. G-code comparison exempts only the entire generator line,
never result clocks. No repair, placement rewrite, guard weakening, retry to
PASS or expected-output change. `plate_selection.rs` origin logic and
`cache/cli_result.rs` recipe are untouched. Twelve Native crashes, SLA/timelapse,
profile admission, general slicing and adjacent geometry behavior are deferred.

## Cases and unchanged RED → GREEN

Anycubic Kobra 3 `0.2/0.4/0.6/0.8 nozzle`: eligibility `206/-50`.
Prusa MK3S `0.25/0.4/0.6/0.8 nozzle`: embedded-postprocess `237/-19`.
Own tests: `orca_parity/stock_early_cli.rs`, selector table `stock_early_cli/cases.rs`.
The existing `VendorProfiles/select_printer`, flattened presets, empty overrides,
`cube10.stl` and `OrcaRunner::export_case` produce the original stock inputs at
runtime in external artifact/cache directories. `early_cli_reference` is called
twice with unchanged `EmptyPlate`/`Postprocess` recipes; cached directory,
stdout/stderr/result/G-code inventory must match. Real Native exit comes from
the Native run, not an invented expected process status. Optional Native prints
visible SKIPPED before artifact-root access; required absence fails closed.

Unchanged RED `34b08e089`:
4 MK3S PASS, 4 Kobra FAIL, exit100. Native Kobra exits206, stdout contains the
`-50` exit line, stderr is empty, result.json is complete. Ares exits1, stdout
is empty, stderr wraps the same text in `InvalidInput`, and result.json is
absent. The test fails reading the missing file; recorded real channels and
Native outputs establish the production mismatch. It is not an input mismatch.
No failing input or assertion was changed to obtain GREEN.

GREEN actual `bfcbbd6732fbdd6074063f21272f20511a79a38f`: 10 PASS, exit0, comprising eight new
CLI cases plus two existing empty-selected-plate controls. The populated plate2
control retains complete Native G-code equality with generator-only exemption.

Exactly one existing Native-derived variant assertion changed, before the
production commit:

- `empty_selected_plate_controls::empty_selected_plate_control_nonempty_outside_volume_keeps_later_error`:
  `Err(InvalidInput(ref text)) if text == OUTSIDE` →
  `Err(NoSuitableObjects(ref text)) if text == OUTSIDE`. Native exit206,
  empty stderr, return_code/error_string and exact payload assertions remain.

No assertions were deleted. The four volume-rejection tests assert Display
text and are unchanged. Other `InvalidInput` checks concern invalid options,
geometry or missing plate identities and are unchanged.
