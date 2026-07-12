# M310: PrintApply update_volume_bboxes multi-layer old extents

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the multi-layer old-volume setup in `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:908-917`: create `volumes_old`, clear each layer range's current volumes when `cached_volume_ids` is empty, or reserve/capture each layer range's existing volumes into `volumes_old` when cached ids are present. Required context comes from M308's sorted eligible volume/cache-id shell at `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, M309's single-layer old-extents reuse at `PrintApply.cpp:895-907`, and existing staged `StagedVolumeExtents`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned volume-cache or bbox pipeline.

## Exit criteria

- Add private staged multi-layer old-extents setup for `PrintApply.cpp:908-917`.
- Preserve that an empty cached-id list clears every layer range's current volumes and produces no reusable old extents.
- Preserve that a non-empty cached-id list captures every layer range's existing volumes into a parallel old-extents list in layer order.
- Preserve that captured layer ranges are emptied for subsequent output population.
- Preserve empty layer-range input behavior.
- Preserve layer order and per-layer volume order when capturing old extents.
- Add tests for empty cached ids clearing all layers, non-empty cached ids capturing and clearing, empty layer ranges, per-layer empty volume lists, and order preservation.
- Defer layer-height range expansion from `PrintApply.cpp:919-927`, cached multi-layer extent reuse from `PrintApply.cpp:928-936`, uncached bbox generation/insertion from `PrintApply.cpp:937-941`, final cache-id refresh already staged in M308, real meshes/transforms/bounding boxes, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
