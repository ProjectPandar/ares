# M292: PrintApply verify-update parent bbox intersection gate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the parent bbox intersection gate in `OrcaSlicer/src/libslic3r/PrintApply.cpp:783-789`, with current-modifier bbox lookup context from `PrintApply.cpp:767-768`, `find_modifier_volume_extents(...)` context from `PrintApply.cpp:705-725`, `PrintObjectRegions::BoundingBox` / `VolumeExtents` / `VolumeRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:216-240`, and `LayerRangeRegions::volumes` / `volume_regions` context from `Print.hpp:271-282`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to the bbox-intersection decision prefix for a parent region that does not already have an override for the current modifier.
- Preserve using `find_modifier_volume_extents(...)` for the candidate parent region id.
- Preserve intersecting that parent bbox with the current modifier bbox from `find_volume_extents(...)` / `bbox` context.
- Return staged state sufficient for the later config-comparison milestone: parent id, parent bbox, current modifier bbox, and whether the bboxes intersect.
- Add tests for intersecting boxes, disjoint boxes, touching boundary intersection, model-part parent bbox direct lookup, modifier parent-chain bbox extension through the existing helper, and missing parent extents panic through the existing helper.
- Defer `region_config_from_model_volume(...)`, config comparison, `return false` reslice decision, callback invalidation, ref-count increment, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
