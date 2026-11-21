# Arranged STL plate slicing

## Upstream boundary

Port the arranged STL execution of the FFF plate loop in OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`). In `src/libslic3r` the boundary
is `Model::update_print_volume_state` (`Model.cpp:839–852`), `Print::apply`,
`Print::validate` and `Print::empty`, followed by `Print::process` and
`Print::get_conflict_string` (`Print.hpp:1325`). The action sets the
`PrintBase::set_no_check_flag` state (`PrintBase.hpp:528–529`); `Print::process`
uses it to skip the conflict checker (`Print.cpp:3946–3964`). Keep the existing
geometry and conflict checker; this slice changes where the CLI invokes them.

`src/OrcaSlicer.cpp:6426–6427,6441–6445,6514–6575,6986–7056,7289–7292`
prechecks the selected plates in order for `--slice 0` when there is more than
one plate. This is fail-fast, not error collection: a precheck failure exits
with that plate's one-based index before processing any plate. With a nonzero
selection the loop skips the other plates. In the processing pass the CLI
checks `get_conflict_string()` before consuming slicing warnings, records
`CLI_GCODE_PATH_CONFLICTS` (`-101`) with `index + 1`, and exits on a conflict
(`7134–7138`). It exports each successful plate before advancing (`7184–7213`),
so an earlier plate's output survives a later processing failure.

## Rust destination

For the direct STL CLI with loaded presets, replace the arranged branch's
per-plate `slice_source` calls in
`ares-core/src/profiles/entry/load_settings.rs::slice_stl_models_with_load_settings`
with the existing `project_slice::state::apply_plate` / `AppliedPlate` boundary
for precheck, then reuse `project_slice::slice_plate` for processing. Complete
prechecks before processing; use the same action `SlicerContext` throughout
processing, instead of rebuilding it with only the triangle limit and custom
G-code. Preserve STL plate membership, custom G-code targeting and the existing
triangle limit. The current combined `ProjectInput::ArrangedStl` load/apply/process
path is a temporary compatibility shell for the upstream separation; do not leave
a second path doing the same work.

`ares-cli/src/direct_stl.rs` enables the existing CLI conflict check for the
arranged branch. It consumes completed plates in order, checks the existing
warning policy and writes each plate before processing the next. Route errors
through the existing CLI failure and `result.json` handlers with the actual
plate ID and, when available, the report from that plate. An error before any
plate is selected is not a plate-failure report. In particular, don't attach
plate 1's report to an error on plate 2, or fabricate a completed plate to
report an error.

## Verification

Use the public CLI with synthetic STLs and loaded presets, running Ares and
Native with identical argv. Establish fixture behavior with Native first:

- One plate passes and a later plate fails a real precheck (use `--mtcpp` with
  unequal triangle counts); compare the indexed exit report and absence of
  G-code. Verify the good plate on its own by selecting it from the same
  arrangement.
- Where a reproducible arranged STL fixture triggers a processing-time
  path conflict on a later plate, compare the exit code, failing plate,
  earlier output retained and conflict precedence. With `--no-check` before
  `--slice`, the same fixture must avoid the conflict; moving `--no-check`
  after `--slice` must not avoid it. A failure on plate 1 or a preset-load
  rejection cannot stand in for the later-plate case.

Use the existing report comparison and output inventory; retain both runs'
argv, reports and logs. Run the affected cases in both Native reference modes,
then formatting, workspace Clippy on all targets and the core/WASM check.

## Deferred

- **Saved-project `--enable-timelapse`.** `OrcaSlicer.cpp:1593–1595,4302–4320`
  reads the action flag once, before plate selection. With smooth timelapse it
  keeps the shared prime-tower setting when the single-filament mapping logic
  would otherwise disable it. The settings feed all plates, not just the
  first. `Print::enable_timelapse_print` (`Print.cpp:5235–5238`) reads the
  resolved mode, not the action flag. In Ares the flag belongs in the saved
  project's profile preparation before per-plate apply
  (`project_slice/cli_plates.rs`), not in the arranged STL processing loop.
- **`--convert-unit` with `--slice >1`.** Native resets the early non-project
  plate selection to all (`OrcaSlicer.cpp:2042–2045`), performs `convert_unit`
  over the loaded models (`4901–4909,4961–4972`, `Model.cpp:976–1040`), then
  arranges, and rereads the requested selection for the slice action
  (`6441–6445,6522–6527`). The current STL CLI's imported-model conversion
  branch is bypassed when it takes the multi-model path, including the
  single-STL `--slice >1` route with loaded presets. Fix conversion at the
  import/transform boundary separately; do not move it into plate execution
  or make `--slice >1` limit which models are converted or arranged.
- No new arrangement algorithms, on-disk slice caches, warning types or
  validation rules. Changes to import, printer or profile admission are
  outside this slice.
