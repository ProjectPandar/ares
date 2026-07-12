# PrintApply model-volume solid-or-modifier predicate Spec

## Goal

Port OrcaSlicer's private `model_volume_solid_or_modifier(...)` predicate into `ares-core` as a staged private helper for later print-object-region invalidation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:542-546`: `model_volume_solid_or_modifier(...)` local predicate.

Required context:
- `OrcaSlicer/src/libslic3r/Model.hpp:340-348`: `ModelVolumeType` enum declaration and discriminant order.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:667-695`: first downstream use context for filtering new volumes before cached-volume reuse.

## Requirements

- Extend private `ares-core` PrintApply staged implementation; do not add public APIs.
- Add staged `ModelVolumeType` variants matching upstream vocabulary:
  - `Invalid = -1`
  - `ModelPart = 0`
  - `NegativeVolume = 1`
  - `ParameterModifier = 2`
  - `SupportBlocker = 3`
  - `SupportEnforcer = 4`
- Add a private helper equivalent to `model_volume_solid_or_modifier(...)`.
- The helper must return `true` for `ModelPart`, `NegativeVolume`, and `ParameterModifier`.
- The helper must return `false` for `Invalid`, `SupportBlocker`, and `SupportEnforcer`.
- Add unit tests for discriminants and predicate behavior over every variant.
- Do not implement real `ModelVolume`, mesh ownership, volume sorting, matrix comparison, cached-volume id mutation, bounding-box math, print-object-region invalidation, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
