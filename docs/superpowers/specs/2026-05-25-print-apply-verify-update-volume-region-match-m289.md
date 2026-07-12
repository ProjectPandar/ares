# PrintApply verify-update volume-region matching Spec

## Goal

Port the first `volume_regions` scan prefix of OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged matching state for later region-validation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:755-766`: initialize the last-visited modifier iterator, iterate `layer_range.volume_regions`, process only model-part or modifier regions, lower-bound lookup the referenced model volume in sorted model volumes, assert it exists, and detect first visits for modifier model volumes.

Required context:
- `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`: model volumes are sorted by id before this scan.
- `OrcaSlicer/src/libslic3r/Model.hpp:905-907`: `is_model_part()` and `is_modifier()` determine eligible volume types.
- `OrcaSlicer/src/libslic3r/Print.hpp:229-240`: `VolumeRegion` stores the referenced `ModelVolume` and parent/region fields outside this slice.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-282`: `LayerRangeRegions::volume_regions` preserves source ModelVolume order.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse or extend the M288 staged model-volume sort state where practical.
- Add private staged volume-region input carrying at least `volume_id` and `volume_type`.
- Add private staged match output carrying `region_id`, `volume_id`, and `first_modifier_visit`.
- The helper must iterate staged volume regions in source order and skip any region whose type is not `ModelPart` or `ParameterModifier`.
- For every eligible region, the helper must lower-bound search sorted model volumes by `volume_id` and panic if no exact match exists.
- The helper must append one output record per eligible region in source order.
- For `ParameterModifier` regions, the helper must set `first_modifier_visit` only when the matched model-volume id differs from the last visited modifier id.
- For later consecutive eligible modifier regions with the same last-visited modifier id, `first_modifier_visit` must be false until another modifier id is visited.
- For `ModelPart` regions, `first_modifier_visit` must always be false and must not change the last visited modifier id.
- Tests must prove skipping ineligible types, exact sorted lookup, missing volume panic, modifier first-visit deduplication, model-part non-first behavior, and source order preservation.
- Keep all new types/functions private or `pub(super)` only for tests.
- Do not implement modifier parent-region creation checks, `next_region_id`, bbox lookup/intersection, region config derivation, config diff/apply, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice return decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
