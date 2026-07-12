# M193: PrintConfig extend_extruder_variant API

## Goal
Port OrcaSlicer's `extend_extruder_variant` helper into Ares as an explicit `SliceOptions::extend_extruder_variant(num_extruders)` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8558-8591` plus call-site context in `PrintConfig.cpp:8593-8596`, option-definition anchors in `PrintConfig.cpp:5239-5264`, and declaration context in `PrintConfig.hpp:634`. It covers only defaulting/resizing `extruder_variant_list` and rebuilding `printer_extruder_id` / `printer_extruder_variant`. No `PrintConfig.cpp:8597+` generic option-vector resizing, `set_num_filaments`, validation, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- Missing `extruder_variant_list` creates `num_extruders` default `"Direct Drive Standard"` entries.
- Existing `extruder_variant_list` resizes to `num_extruders`, extending with the first entry and truncating extras.
- Generated `printer_extruder_id` and `printer_extruder_variant` arrays are cleared/rebuilt from comma-separated variant entries.
- Generated ids are 1-based extruder ids repeated once per generated variant.
- `num_extruders = 0` produces empty arrays.
- Invalid present `extruder_variant_list` values return `SliceError::InvalidInput`.
- Existing M192 parameter-size API behavior remains intact.
- `PrintConfig.cpp:8597+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
