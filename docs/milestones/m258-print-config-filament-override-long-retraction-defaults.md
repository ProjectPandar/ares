# M258: compute_filament_override_value long-retraction override defaults

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the long-retraction special-case prefix of `compute_filament_override_value(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10051-10071`, with declaration context from `PrintConfig.hpp:690-691`, enum context from `PrintConfig.hpp:183-188`, option-definition context from `PrintConfig.cpp:5077-5090`, and later `apply_override`/changed-key context from `PrintConfig.cpp:10073-10082`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned filament override pipeline.

## Exit criteria

- Add an internal `ares-core` helper that prepares the staged filament override input vector for `long_retractions_when_cut` and `retraction_distances_when_cut` before a later `apply_override` milestone.
- Preserve upstream behavior: when `enable_long_retraction_when_cut` is not `LongRectrationLevel::EnableFilament` (`2`), replace the filament-provided values for `long_retractions_when_cut` or `retraction_distances_when_cut` with nil/default entries of the same length, treating the `retraction_distances_when_cut` push into `opt_long_retraction_default` at `PrintConfig.cpp:10069` as an upstream typo because `opt_retraction_distance_default` is assigned to `opt_new_filament` at `PrintConfig.cpp:10070`; when it is `2`, preserve the filament values unchanged; other keys are unchanged.
- Use existing JSON option staging conventions: string `"nil"` is the nullable nil marker for bool/float vectors in Ares tests.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement `ConfigOptionVector::apply_override`, diff-key insertion, `filament_overrides` config mutation, full `compute_filament_override_value`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
