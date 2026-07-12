# Spec: M311 PrintApply update_volume_bboxes multi-layer expanded ranges

## Goal

Port OrcaSlicer's multi-layer `update_volume_bboxes(...)` range-expansion setup into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:919-927`: in the multi-layer branch, create `bboxes` and `ranges`, reserve range capacity to `layer_ranges.size()`, copy each `layer_range.layer_height_range`, subtract `EPSILON` from `first`, add `EPSILON` to `second`, and append the expanded range.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:908-917`: M310 stages multi-layer old-volume setup before this range setup.
- Existing staged `StagedMultiLayerVolumeCacheLayer` models the private multi-layer volume-cache branch, but M311 remains focused on layer height ranges only.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse the private staged layer-height range record with lower/upper values matching `t_layer_height_range` semantics for this slice; add accessors only as needed for the helper.
- Add a helper that returns expanded ranges in input order by subtracting the supplied epsilon from `first` and adding it to `second`.
- Preserve empty input as empty output.
- Preserve zero epsilon as an exact copy.
- Do not invent an Ares-specific epsilon constant or tolerance policy in this milestone; pass epsilon explicitly so the slice mirrors the upstream `EPSILON` operation without owning global numeric policy.
- Do not perform cached multi-layer reuse, uncached bbox generation/insertion, real bbox vector population, real meshes, transforms, real `LayerRangeRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- A single layer height range expands lower/upper bounds by the supplied epsilon.
- Multiple input ranges preserve input order.
- Empty input returns empty output.
- Zero epsilon returns exact copied ranges.
- Negative/lower-bound ranges are expanded arithmetically without clamping.

## Migration note

This milestone stages only `PrintApply.cpp:919-927`. Later milestones must continue with cached multi-layer extent reuse at `PrintApply.cpp:928-936` and uncached bbox generation/insertion at `PrintApply.cpp:937-941` as separate source-cited rewrite slices.
