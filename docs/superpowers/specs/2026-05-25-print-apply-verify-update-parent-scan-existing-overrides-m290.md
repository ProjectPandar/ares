# PrintApply verify-update parent scan existing overrides Spec

## Goal

Port the first-modifier parent scan prefix after the current-modifier bbox lookup of OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged state for later modifier override and reslice-decision milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:766 and PrintApply.cpp:769-782`: initialize `next_region_id`, scan prior parent regions in descending order, assert parent candidates do not reference the same model volume, filter to model-part/modifier parents, assert existing generated-region ordering, and increment `next_region_id` when a parent override already exists.

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:229-240`: `VolumeRegion` stores `model_volume` and `parent`.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-282`: `LayerRangeRegions::volume_regions` preserves source ModelVolume order.
- `OrcaSlicer/src/libslic3r/Model.hpp:905-907`: `is_model_part()` and `is_modifier()` determine eligible parent candidates.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse or extend M289 staged volume-region state where practical.
- Add private staged parent-scan output carrying final `next_region_id`, scanned eligible parent ids, and existing override parent ids.
- The helper is called only for first visits of `ParameterModifier` regions identified by the M289 matching stage, after the current-modifier bbox lookup/assert that this milestone defers. The helper must initialize `next_region_id` to `current_region_id`.
- The helper must scan parent candidate ids from `current_region_id - 1` down to `0`.
- The helper must panic if any scanned candidate references the same model volume id as the current modifier region.
- The helper must ignore candidates whose type is neither `ModelPart` nor `ParameterModifier`.
- For eligible candidates, the helper must enforce the upstream ordering assertion: `next_region_id == volume_regions.len()` OR `volume_regions[next_region_id].volume_id != current_modifier_volume_id` OR `volume_regions[next_region_id].parent <= parent_region_id`.
- The helper must record each eligible parent id in descending scan order.
- The helper must detect an existing override when `next_region_id < volume_regions.len()`, the region at `next_region_id` references the current modifier volume id, and its parent equals the current parent candidate id; then it must record that parent id and increment `next_region_id`.
- If no existing override matches a candidate, the helper must leave `next_region_id` unchanged for that candidate.
- Tests must prove descending scan order, ineligible parent skipping, same-volume panic, ordering assertion panic, existing override increment, no-override unchanged behavior, and multiple sequential existing overrides.
- Keep all new types/functions private or `pub(super)` only for tests.
- Do not implement `find_modifier_volume_extents`, bbox intersection, config derivation/comparison/application, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice return decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
