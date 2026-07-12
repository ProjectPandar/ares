# M279: PrintApply model-volume solid-or-modifier predicate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `model_volume_solid_or_modifier(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:542-546`, with `ModelVolumeType` declaration context from `OrcaSlicer/src/libslic3r/Model.hpp:340-348`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private staged `ModelVolumeType` vocabulary for the upstream variants `Invalid`, `ModelPart`, `NegativeVolume`, `ParameterModifier`, `SupportBlocker`, and `SupportEnforcer`.
- Preserve upstream discriminant order from `Model.hpp:341-348`, including `Invalid = -1` and `ModelPart = 0`.
- Add a private staged predicate equivalent to `model_volume_solid_or_modifier(...)` that returns `true` only for `ModelPart`, `NegativeVolume`, and `ParameterModifier`.
- Add tests proving all included and excluded variants.
- Defer real `ModelVolume`, mesh data, transformation math, bounding boxes, print-object-region invalidation, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
