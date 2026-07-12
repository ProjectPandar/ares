# M312: PrintApply update_volume_bboxes multi-layer cached reuse

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the cached-volume reuse branch inside the multi-layer `update_volume_bboxes(...)` loop at `OrcaSlicer/src/libslic3r/PrintApply.cpp:928-936`: iterate sorted `model_volumes`, process only `model_volume_solid_or_modifier(...)` volumes, when `cached_volume_ids` contains the current volume id iterate every layer range, pick that layer's old extents from `volumes_old` by layer index, lower-bound search by volume id, and append the old extent only when present. Required context comes from M310 old-extents setup at `PrintApply.cpp:908-917`, M311 expanded-range setup at `PrintApply.cpp:919-927`, M308 ordering/cache-id context at `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, and existing staged `StagedMultiLayerVolumeCacheLayer`, `StagedVolumeExtents`, and `staged_find_volume_extents(...)`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned cache or slicing pipeline.

## Exit criteria

- Add private staged multi-layer cached extent reuse for `PrintApply.cpp:928-936`.
- Preserve processing only solid-or-modifier model volumes.
- Preserve reusing old per-layer extents only when the volume id is cached and the old extent exists in that layer.
- Preserve skipping cached ids that are missing from a specific layer's old extents.
- Preserve no action for uncached ids; uncached bbox generation/insertion remains deferred.
- Preserve model-volume loop order and per-layer append order.
- Preserve duplicate cached model-volume ids by processing each staged input independently.
- Add tests for per-layer cached reuse, missing old extent skip, non-eligible filtering, uncached no-op, model-volume order, duplicate cached inputs, and empty layer/old-extents input.
- Defer uncached bbox generation/insertion from `PrintApply.cpp:937-941`, real `transformed_its_bboxes_in_z_ranges(...)`, real bbox vector population, real meshes/transforms/bounding boxes, final cache-id refresh already staged in M308, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
