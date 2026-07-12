# M289: PrintApply verify-update volume-region matching

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the first `volume_regions` scan inside `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:755-766`, with sorted model-volume lookup context from `Model.hpp:1227-1230`, `ModelVolume::is_model_part()` / `is_modifier()` context from `Model.hpp:905-907`, `PrintObjectRegions::LayerRangeRegions::volume_regions` context from `Print.hpp:271-282`, and `VolumeRegion::model_volume` context from `Print.hpp:229-240`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to the `volume_regions` eligibility and model-volume matching prefix of `verify_update_print_object_regions(...)`.
- Preserve processing only regions whose model volume is a model part or parameter modifier.
- Preserve lower-bound lookup against already sorted model volumes by volume id, with panic/assert behavior when the referenced volume id is missing.
- Preserve returning matched region ids in source `volume_regions` order.
- Preserve last-visited modifier detection for modifier model volumes: a modifier region is marked as a first modifier visit when its modifier volume id differs from the last visited modifier id, while consecutive eligible modifier regions with the same last-visited modifier id are not.
- Preserve model-part regions as eligible but never first modifier visits.
- Add tests for skipping non-model-part/non-modifier regions, matching sorted model volumes by id, missing model volume panic, modifier first-visit deduplication, model-part non-first-modifier behavior, and preserving source region order.
- Defer modifier parent-region creation checks, `next_region_id`, bbox lookup/intersection, region config derivation, config diff/apply, callback invalidation, ref-count increment, painted/fuzzy painted regions, return-value reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
