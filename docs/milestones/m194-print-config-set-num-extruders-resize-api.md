# M194: PrintConfig set_num_extruders vector resizing API

## Goal
Port OrcaSlicer's `DynamicPrintConfig::set_num_extruders` generic extruder-option resize loop into Ares as an explicit `SliceOptions::set_num_extruders(num_extruders)` API for UI/config consumers.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8593-8610`, using the already-ported M193 `extend_extruder_variant` helper from `PrintConfig.cpp:8558-8591`, the already-ported M192 `get_parameter_size` helper from `PrintConfig.cpp:8529-8556`, the already-ported M184 `print_config_def.extruder_option_keys()` list, `OrcaSlicer/src/libslic3r/Config.hpp:635-663` vector resize semantics, and `OrcaSlicer/src/libslic3r/Config.cpp:295-315` / option-definition defaults for default fill values. It covers only `set_num_extruders` ordering, default-filament-profile skip behavior, per-key parameter sizing, and vector resizing. No `set_num_filaments`, validation, preset/model loading, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior is added.

## Exit checklist
- `SliceOptions::set_num_extruders(num_extruders)` first applies the M193 extruder-variant extension.
- Every registered extruder option key except `default_filament_profile` is resized to `parameter_size(key, num_extruders)`.
- Existing non-empty arrays extend by cloning their first element and truncate extras, matching `ConfigOptionVector::resize`.
- Existing empty arrays extend from the source-cited registry default value.
- Missing extruder arrays are materialized from source-cited registry defaults for Ares' sparse `SliceOptions` boundary.
- `default_filament_profile` remains untouched even when absent or present.
- Invalid present non-array extruder option values return `SliceError::InvalidInput`.
- Existing M192 parameter-size and M193 extruder-variant behavior remains intact.
- `PrintConfig.cpp:8612+` behavior remains unchanged/deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
