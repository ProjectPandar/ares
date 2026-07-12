# Spec: M312 PrintApply update_volume_bboxes multi-layer cached reuse

## Goal

Port OrcaSlicer's cached-volume reuse branch in multi-layer `update_volume_bboxes(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:928-936`: in the multi-layer branch, iterate model volumes, filter to solid-or-modifier volumes, for cached ids iterate each layer range, select the matching layer's `volumes_old`, lower-bound search by volume id, and append the old extent when found.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:908-917`: M310 staged old extents are captured in layer order.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:919-927`: M311 staged expanded ranges are prepared before this loop but not consumed by cached reuse.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`: M308 sorted model volumes and refreshed cache ids bound loop order/cache context.
- Existing staged `StagedMultiLayerVolumeCacheLayer`, `StagedVolumeExtents`, and `staged_find_volume_extents(...)` model layer outputs and lower-bound old extent lookup.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a private staged multi-layer cached-reuse volume input carrying id and staged model-volume type.
- Add a helper that mutates staged multi-layer layers by appending reused old extents for cached eligible model volumes.
- For each cached eligible model volume, scan every layer in order and append the matching old extent for that layer only when present.
- Skip cached ids missing from a layer's old extents; do not compute a fallback bbox.
- Do nothing for uncached ids in this milestone; uncached bbox generation/insertion is deferred.
- Preserve model-volume input order, layer order, existing per-layer output prefix, and duplicate model-volume processing.
- Do not perform uncached bbox generation/insertion, real `transformed_its_bboxes_in_z_ranges(...)`, real bbox vector population, real meshes, transforms, real `LayerRangeRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Cached eligible volume reuses matching old extent across multiple layers.
- Missing old extent in one layer is skipped while other layers can append.
- Non-solid-or-modifier volumes are filtered out.
- Uncached eligible volumes do not append anything.
- Output append order follows model-volume input order per layer.
- Duplicate cached model-volume ids are processed independently.
- Empty layer and old-extents input is accepted.

## Migration note

This milestone stages only `PrintApply.cpp:928-936`. Later milestones must continue with uncached bbox generation/insertion at `PrintApply.cpp:937-941` as a separate source-cited rewrite slice.
