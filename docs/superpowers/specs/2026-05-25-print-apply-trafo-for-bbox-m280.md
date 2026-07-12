# PrintApply bbox transform composition Spec

## Goal

Port OrcaSlicer's private `trafo_for_bbox(...)` helper into `ares-core` as a staged private transform helper for later print-object-region bounding-box milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:548-554`: `trafo_for_bbox(...)` multiplies object and volume transforms, zeros composed X/Y translation, and casts to `Transform3f`.

Required context:
- `OrcaSlicer/src/libslic3r/Point.hpp:79-85`: `Transform3f` and `Transform3d` are Eigen 3D affine transform aliases with `float` / `double` scalar types.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:582-606`: downstream bounding-box transform use.

## Requirements

- Extend private `ares-core` PrintApply staged implementation; do not add public APIs.
- Add staged 4x4 affine transform storage for f64 input and f32 output, limited to this source boundary.
- Add a private helper equivalent to `trafo_for_bbox(object_trafo, volume_trafo)`.
- The helper must multiply `object_trafo * volume_trafo` in that order.
- The helper must set the composed translation X and Y to `0.0` after multiplication.
- The helper must preserve composed translation Z.
- The helper must preserve linear transform terms.
- The helper must return f32 values equivalent to Orca's `m.cast<float>()` destination.
- Add unit tests for composition order, X/Y translation zeroing, Z preservation, linear term preservation, and f32 cast output.
- Do not implement full Eigen-compatible transforms, inverse transforms, rotation/mirroring comparison, mesh vertex transformation, bounding boxes, print-object-region invalidation, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
