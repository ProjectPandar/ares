# Spec: M309 PrintApply update_volume_bboxes single-layer extents

## Goal

Port OrcaSlicer's single-layer `update_volume_bboxes(...)` bbox reuse/insertion branch into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:895-907`: for one `LayerRangeRegions`, move old volume extents aside, reserve output capacity, iterate sorted model volumes, filter to solid-or-modifier volumes, reuse old extents for cached ids present in old extents, and insert newly computed extents for uncached ids.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893`: M308 sorts model volumes before this branch.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:946-950`: M308 refreshes cached ids after bbox processing.
- Existing staged `StagedVolumeExtents` and `staged_find_volume_extents(...)` model sorted old extent lookup.
- Existing staged model-volume eligibility maps `model_volume_solid_or_modifier(...)`.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a private staged single-layer model-volume input carrying id, staged model-volume type, and a precomputed new bbox/extents value supplied by tests/later callers.
- Add a helper that returns the new single-layer `volume_regions.volumes` equivalent from cached ids, old extents, and staged model volumes.
- Reuse old extents only when `cached_volume_ids` contains the volume id and `volumes_old` contains that volume id.
- Skip cached ids whose old extent is missing, matching upstream's lack of fallback computation in that branch.
- Insert the supplied new extents when the volume id is not cached.
- Filter out non-solid-or-modifier volume types.
- Preserve input model-volume order and duplicate processing; do not sort or deduplicate in this milestone because M308 owns ordering.
- Do not perform actual `transformed_its_bbox2d(...)`, real mesh/transform/bbox computation, multi-layer behavior, cache-id refresh, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Cached eligible volume reuses the matching old extent.
- Uncached eligible volume inserts the supplied new extent.
- Cached eligible volume with missing old extent is skipped.
- Non-solid-or-modifier volumes are filtered out.
- Output follows input model-volume order.
- Duplicate ids are processed independently.
- Empty input returns empty output.

## Migration note

This milestone stages only `PrintApply.cpp:895-907`. Later milestones must continue with the multi-layer branch at `PrintApply.cpp:908-941` as source-cited rewrite slices.
