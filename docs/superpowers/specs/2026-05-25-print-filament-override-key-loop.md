# Print filament override key-loop assembly Spec

## Goal
Port OrcaSlicer's per-key filament override loop into `ares-core` as a private JSON-map helper that composes the already staged `compute_filament_override_value(...)` behavior.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/Print.cpp:2976-2988`: `extruder_retract_keys`, `filament_` prefix lookup, and per-key `compute_filament_override_value(...)` call inside `Print::update_filament_maps_to_config(...)`.

Context only:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:220-244`: equivalent diff loop that calls `compute_filament_override_value(...)` only for keys with present filament override values.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10051-10082` and `PrintConfig.hpp:690-691`: M258-M260 staged `compute_filament_override_value(...)` behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7195` and `PrintConfig.hpp:569-574`: sorted `extruder_retract_keys()` key-list context, already exposed by `ares-core` registry APIs.

## Approval gate
Do not begin Task 1, tests, implementation, or any code changes for M261 until this M261 plan/spec review returns `APPROVE`.

## Requirements
- Add a private helper in `crates/ares-core/src/options/filament_override.rs` or a split child module if needed for LOC.
- Suggested signature: `fn collect_filament_override_updates(old_machine_values: &serde_json::Map<String, Value>, new_machine_values: &serde_json::Map<String, Value>, new_full_config_values: &serde_json::Map<String, Value>, enable_long_retraction_when_cut: Option<&Value>, default_index: &[isize], nullable_override: bool, diff_keys: &mut Vec<String>, filament_overrides: &mut serde_json::Map<String, Value>) -> Result<(), SliceError>`.
- Iterate `crate::options::registry::extruder_retract_keys()` in source order.
- For each `key`, look up `new_full_config_values[format!("filament_{key}")]`; if absent, skip the key without requiring machine values.
- For present filament values, look up `old_machine_values[key]` and `new_machine_values[key]` and return `SliceError::InvalidInput` if either is absent at this private helper boundary.
- Call the existing M260 `compute_filament_override_value(...)` with the unprefixed key, old machine value, new machine value, filament value, `enable_long_retraction_when_cut`, `default_index`, `nullable_override`, `diff_keys`, and `filament_overrides`.
- Preserve M260 changed/unchanged behavior: changed calls append/insert, unchanged leaves outputs untouched.
- Do not implement full `Print::update_filament_maps_to_config`, config apply, placeholder parser apply, full `PrintApply::print_config_diffs`, scalar/non-filament diffing, profile loading, public APIs, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
- Add tests for:
  - present `filament_retraction_length` changes append `retraction_length` and insert the computed unprefixed override value;
  - missing `filament_` prefixed override skips the key and does not require old/new machine values;
  - multiple present keys produce `diff_keys` in `extruder_retract_keys()` source order;
  - unchanged computed override suppresses output.

## Non-goals
- Do not implement `m_config.filament_map` updates, `m_ori_full_print_config` mutation, multiple-filament printer-extruder expansion, `apply_only`, placeholder parser updates, invalidation, full print diff computation, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
