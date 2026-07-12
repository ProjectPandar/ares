# M195: PrintConfig set_num_filaments vector resizing API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::set_num_filaments` filament-option resize loop into Ares as an explicit `SliceOptions::set_num_filaments(num_filaments)` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8612-8627`, using the already-ported M184 `print_config_def.filament_option_keys()` list, `OrcaSlicer/src/libslic3r/Config.hpp:635-663` vector resize semantics, and `OrcaSlicer/src/libslic3r/Config.cpp:295-315` / option-definition defaults for default fill values. It covers only `set_num_filaments` default-filament-profile skip behavior and vector resizing to `num_filaments`. No validation, preset/model loading, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::set_num_filaments(num_filaments)` iterates every registered filament option key except `default_filament_profile`.
- Existing non-empty arrays resize to `num_filaments`, extending by cloning the first element and truncating extras.
- Existing empty arrays extend from source-cited registry default values.
- Missing filament arrays are materialized from source-cited registry defaults for Ares' sparse `SliceOptions` boundary.
- `default_filament_profile` remains untouched even when absent or present.
- `num_filaments = 0` produces empty non-skipped filament arrays.
- Invalid present non-array filament option values return `SliceError::InvalidInput`.
- Existing M194 `set_num_extruders`, M193 extruder-variant, and M192 parameter-size behavior remains intact.
- `PrintConfig.cpp:8629+` validation behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
