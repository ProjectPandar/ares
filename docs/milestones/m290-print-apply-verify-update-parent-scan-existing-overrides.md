# M290: PrintApply verify-update parent scan existing overrides

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the first-modifier parent scan prefix after the current-modifier bbox lookup in `OrcaSlicer/src/libslic3r/PrintApply.cpp:766 and PrintApply.cpp:769-782`, with `VolumeRegion::model_volume` / `parent` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, `LayerRangeRegions::volume_regions` source-order context from `Print.hpp:271-282`, and model-part/modifier predicate context from `OrcaSlicer/src/libslic3r/Model.hpp:905-907`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to the parent-region scan prefix used on first modifier visits, with the current-modifier bbox lookup/assert explicitly deferred.
- Preserve `next_region_id` initialization to the current modifier region id.
- Preserve descending parent scan from `next_region_id - 1` down to zero.
- Preserve assertion that a parent scan candidate cannot reference the same model volume as the current modifier region.
- Preserve filtering parent scan candidates to model-part or modifier model volumes only.
- Preserve the generated-region ordering assertion: either `next_region_id` is at the end, or the next region is for a different model volume, or its parent id is less than or equal to the current parent scan id.
- Preserve detecting an already existing override when the next region is in bounds, references the current modifier model volume, and has parent equal to the current parent scan id; in that case increment `next_region_id`.
- Return staged parent-scan events sufficient for later bbox/config/reslice milestones: scanned eligible parent ids and existing-override parent ids, plus final `next_region_id`.
- Add tests for descending scan order, skipping ineligible parent candidates, same-volume assertion, ordering assertion, existing override increments, no override leaves `next_region_id` unchanged, and multiple existing overrides incrementing sequentially.
- Defer `find_volume_extents(...)` / current-modifier bbox assertion from `PrintApply.cpp:767-768`, `find_modifier_volume_extents(...)`, bbox intersection, `region_config_from_model_volume(...)`, config comparison, callback invalidation, ref-count increment, painted/fuzzy painted regions, return-value reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
