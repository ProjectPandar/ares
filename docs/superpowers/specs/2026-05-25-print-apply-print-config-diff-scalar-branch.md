# PrintApply print-config diff scalar branch Spec

## Goal
Port OrcaSlicer's `print_config_diffs(...)` key-loop scalar/non-filament diff behavior into `ares-core` as a private JSON-map helper that composes the already staged filament override behavior.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:232-260`: current-config key iteration, old/new lookup, missing-new skip, filament override branch, scalar changed-key insertion, and `wipe_tower_x` / `wipe_tower_y` plate-index special case.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:262-264`: return of collected `print_diff`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:220-231`: function-local `filament_overrides`, `plate_index`, `filament_maps`, `extruder_retract_keys`, `filament_prefix`, and `print_diff` context.
- `OrcaSlicer/src/libslic3r/Print.cpp:2976-2988`: M261 equivalent filament override call-loop context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10051-10082` and `PrintConfig.hpp:690-691`: M258-M260 staged `compute_filament_override_value(...)` behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694-6708`: `wipe_tower_x` / `wipe_tower_y` option definitions as float vectors.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M262 until this M262 plan/spec review returns `APPROVE`.

## Requirements
- Add a private helper in `crates/ares-core/src/options/filament_override.rs` or a split child module if needed for LOC.
- Suggested signature: `fn collect_print_config_diff_updates(current_config_values: &serde_json::Map<String, Value>, new_full_config_values: &serde_json::Map<String, Value>, current_keys: &[String], plate_index: usize, enable_long_retraction_when_cut: Option<&Value>, default_index: &[isize], nullable_override: bool, diff_keys: &mut Vec<String>, filament_overrides: &mut serde_json::Map<String, Value>) -> Result<(), SliceError>`.
- Iterate `current_keys` in order to mirror `current_config.keys()`.
- For each key, require an old value from `current_config_values[key]`; this private helper may return `SliceError::InvalidInput` if a provided key is absent from the current map.
- If `new_full_config_values[key]` is absent, skip the key.
- If the key is in `crate::options::registry::extruder_retract_keys()` and `new_full_config_values[format!("filament_{key}")]` is present, delegate to the existing M260/M261 filament override path for that key and do not also append a scalar changed key.
- Otherwise, if old and new values are equal, leave outputs untouched.
- Otherwise, for `wipe_tower_x` and `wipe_tower_y`, treat both values as arrays and apply Orca's plate-index logic:
  - if both arrays have `plate_index`, append the key only when indexed values differ;
  - if exactly one array has `plate_index`, append the key;
  - if neither array has `plate_index`, do not append the key.
- For all other changed keys, append the unprefixed key to `diff_keys`.
- Preserve existing `diff_keys` and `filament_overrides` mutation behavior from M260/M261.
- Do not implement public `PrintApply::print_config_diffs` wiring, `full_print_config_diffs`, print config mutation, config apply/apply_only, placeholder parser updates, profile loading, public APIs, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.

## Non-goals
- Do not implement `full_print_config_diffs`, `Print::update_filament_maps_to_config`, placeholder parser updates, invalidation, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
