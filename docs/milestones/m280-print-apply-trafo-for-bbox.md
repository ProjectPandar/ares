# M280: PrintApply bbox transform composition

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `trafo_for_bbox(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:548-554`, with transform type alias context from `OrcaSlicer/src/libslic3r/Point.hpp:79-85` and downstream bounding-box context from `PrintApply.cpp:582-606`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned print pipeline.

## Exit criteria

- Add private staged 3D affine transform data sufficient for this PrintApply boundary.
- Preserve `object_trafo * volume_trafo` composition order.
- Preserve zeroing the composed translation's X and Y components while keeping Z and the linear transform terms.
- Preserve the staged float-cast destination semantics by returning `f32` matrix values for the bbox transform.
- Add tests for identity, XY translation zeroing, Z preservation, non-commutative multiplication order, linear term preservation, and f32 cast output.
- Defer full Eigen-compatible transform API, inverse/rotation comparison, mesh vertex transformation, bounding boxes, print-object-region invalidation, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
