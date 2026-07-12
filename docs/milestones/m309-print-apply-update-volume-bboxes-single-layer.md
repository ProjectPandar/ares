# M309: PrintApply update_volume_bboxes single-layer extents

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the single-layer branch of `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:895-907`: move the existing `layer_range.volumes` into `volumes_old`, reserve output capacity, iterate sorted `model_volumes`, process only `model_volume_solid_or_modifier(...)` volumes, reuse an old extent only when `cached_volume_ids` contains the volume id and `volumes_old` contains that id, and otherwise add a newly computed bbox only when the id is not cached. Required context comes from M308's model-volume ordering/cache-id refresh at `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, existing staged `StagedVolumeExtents` / `staged_find_volume_extents(...)`, and staged model-volume eligibility. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned volume-cache or bbox-computation pipeline.

## Exit criteria

- Add private staged single-layer update-volume-bboxes extents behavior for `PrintApply.cpp:895-907`.
- Preserve processing only solid-or-modifier model volumes.
- Preserve old extent reuse when the volume id is cached and an old extent exists.
- Preserve new extent insertion when the volume id is not cached.
- Preserve upstream skip behavior when the id is cached but the old extent is missing.
- Preserve output order following the input model-volume order established by M308.
- Preserve duplicate model-volume ids by processing each staged input independently.
- Add tests for cached reuse, uncached insertion, cached-missing-old skip, non-eligible filtering, source-order output, duplicate processing, and empty input.
- Defer actual `transformed_its_bbox2d(...)`, real meshes/transforms/bounding boxes, multi-layer branch behavior from `PrintApply.cpp:908-941`, final cache-id refresh already staged in M308, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
