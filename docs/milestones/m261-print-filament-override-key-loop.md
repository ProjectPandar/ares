# M261: Print filament override key-loop assembly

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the per-key filament override loop in `OrcaSlicer/src/libslic3r/Print.cpp:2976-2988`, with equivalent `PrintApply.cpp:220-244` diff-loop context, `PrintConfig.cpp:10051-10082` / `PrintConfig.hpp:690-691` `compute_filament_override_value(...)` context from M258-M260, and `PrintConfig.cpp:7164-7195` / `PrintConfig.hpp:569-574` extruder retract key-list context from M153. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add an internal `ares-core` helper that stages the Orca per-key filament override loop over JSON option maps.
- Preserve upstream loop behavior: iterate `extruder_retract_keys` in source order, construct `filament_` + key lookup, skip keys when the filament-prefixed option is missing, read old machine and new machine values by the unprefixed key, and call the existing M260 staged `compute_filament_override_value(...)` helper for present filament override values.
- Preserve existing `diff_keys` and `filament_overrides` mutation behavior from M260.
- Add focused tests for changed override collection, missing filament-prefixed skip, multiple-key source-order output, and unchanged override suppression.
- Do not implement `Print::update_filament_maps_to_config` state mutation, `m_full_print_config.update_values_to_printer_extruders_for_multiple_filaments`, `m_config.apply_only`, placeholder parser updates, `PrintApply::print_config_diffs` scalar/non-filament diff behavior, public API wiring, profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
