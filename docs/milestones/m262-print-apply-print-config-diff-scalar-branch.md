# M262: PrintApply print-config diff scalar branch

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the scalar/non-filament diff branch inside `print_config_diffs(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:232-260`, with return context from `PrintApply.cpp:262-264`, function-local setup context from `PrintApply.cpp:220-231`, M261 filament override call-loop context from `PrintApply.cpp:240-244` / `Print.cpp:2976-2988`, M258-M260 `compute_filament_override_value(...)` context from `PrintConfig.cpp:10051-10082` / `PrintConfig.hpp:690-691`, and wipe tower option-definition context from `PrintConfig.cpp:6694-6708`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add an internal `ares-core` helper that stages the `PrintApply::print_config_diffs(...)` key loop over JSON option maps.
- Preserve upstream loop behavior: iterate current config keys in caller-provided order, skip keys missing from the new full config, call the existing M261 filament override helper when an extruder retract key has a present `filament_` option, otherwise append changed non-filament keys.
- Preserve `wipe_tower_x` and `wipe_tower_y` special handling: when both old and new values contain the requested `plate_index`, compare only that indexed value; when only one side contains the index, emit the key; when neither side contains the index, suppress the key.
- Preserve existing `diff_keys` and `filament_overrides` mutation behavior from M260/M261 for filament override keys.
- Add focused tests for missing-new skip, scalar changed-key insertion, unchanged suppression, wipe-tower indexed comparison, wipe-tower one-sided index presence, and filament override delegation taking precedence over scalar changed-key insertion.
- Do not implement public `PrintApply::print_config_diffs` wiring, `full_print_config_diffs`, `Print::update_filament_maps_to_config` state mutation, `m_config.apply_only`, placeholder parser updates, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
