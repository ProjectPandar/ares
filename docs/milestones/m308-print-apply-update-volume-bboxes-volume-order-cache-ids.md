# M308: PrintApply update_volume_bboxes volume order/cache ids

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the model-volume ordering and cached-volume-id refresh in `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`: the function first sorts `model_volumes` by `ObjectID` via `model_volumes_sort_by_id(model_volumes)`, processes only `model_volume_solid_or_modifier(...)` volumes, and finally clears and rebuilds `cached_volume_ids` from the sorted solid-or-modifier volumes. Required context comes from `model_volume_solid_or_modifier(...)` behavior already staged from Orca model-volume type handling and from existing staged cache invalidation behavior in `volume_cache_state.rs`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned volume-cache or slicing pipeline.

## Exit criteria

- Add private staged update-volume-bboxes input ordering/cache-id refresh for `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`.
- Preserve sorting model volumes by id before cache processing.
- Preserve filtering to solid-or-modifier volumes only.
- Preserve refreshed cached-volume ids in sorted model-volume order.
- Preserve duplicate ids in source behavior order after sorting rather than inventing deduplication.
- Preserve that unsupported/support-only/invalid model volume types do not appear in refreshed cache ids.
- Add tests in a focused update-volume-bboxes test module for unsorted input sorting, filtering non-solid-or-modifier volumes, empty input, already sorted input, duplicate ids, and replacing stale cached ids.
- Defer single-layer and multi-layer bounding-box reuse/computation from `PrintApply.cpp:895-941`, real meshes/transforms/bounding boxes, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
