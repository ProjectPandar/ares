# M311: PrintApply update_volume_bboxes multi-layer expanded ranges

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the multi-layer range setup in `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:919-927`: create the per-layer bbox/range work vectors, reserve range capacity to match `layer_ranges.size()`, copy each `layer_range.layer_height_range`, expand the lower bound by subtracting `EPSILON`, expand the upper bound by adding `EPSILON`, and append the expanded range in layer order. Required context comes from M310's multi-layer old-extents setup at `PrintApply.cpp:908-917` and existing staged multi-layer volume-cache state. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned range, bbox, or slicing pipeline.

## Exit criteria

- Reuse the private staged layer-height range data and add a helper for the `PrintApply.cpp:919-927` multi-layer range-expansion setup.
- Preserve that every input layer height range is copied and expanded by `[-EPSILON, +EPSILON]`.
- Preserve input layer order in the returned expanded ranges.
- Preserve empty layer-range input behavior.
- Preserve exact supplied epsilon behavior without introducing an Ares-specific tolerance policy.
- Keep the helper private to staged `ares-core` PrintApply modules; do not add public APIs.
- Add tests for single range expansion, multiple range order preservation, empty input, zero epsilon, and negative/lower-bound ranges.
- Defer cached multi-layer extent reuse from `PrintApply.cpp:928-936`, uncached bbox generation/insertion from `PrintApply.cpp:937-941`, real `BoundingBox` vector population, real meshes/transforms/bounding boxes, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
