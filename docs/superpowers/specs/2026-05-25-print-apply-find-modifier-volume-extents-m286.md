# PrintApply find modifier volume extents Spec

## Goal

Port OrcaSlicer's private `find_modifier_volume_extents(...)` helper into `ares-core` as a staged private helper for later modifier clipping and print-object-region milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:705-725`: load the current volume region, find its extents, copy them to output, and for modifier volumes extend through parent volume extents until reaching a model-part parent.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:697-703`: `find_volume_extents(...)` exact-id lookup staged by M285.
- `OrcaSlicer/src/libslic3r/Print.hpp:229-240`: `VolumeRegion` stores `model_volume`, `parent`, `region`, `bbox`, and previous-region link.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-282`: `LayerRangeRegions` stores sorted `volumes` plus source-ordered `volume_regions`.
- `OrcaSlicer/src/libslic3r/Model.hpp:901-907`: `ModelVolume::is_model_part()` checks `ModelVolumeType::MODEL_PART`.
- `OrcaSlicer/src/libslic3r/Print.hpp:216-223`: `PrintObjectRegions::BoundingBox` is an extendable f32 3D bbox.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M285 staged volume extents and bbox lookup where practical.
- Add private staged `VolumeRegion` state with at least `volume_id`, `is_model_part`, and `parent` fields.
- Add a private helper equivalent to `find_modifier_volume_extents(layer_range, this_region_id)` over staged volume regions and extents.
- The helper must load the current region by `this_region_id` and panic if the index is invalid.
- The helper must find the current region's extents by volume id and panic if missing.
- The helper must initialize the output bbox from the current region's bbox.
- If the current region is a model part, the helper must return that bbox without parent traversal.
- If the current region is not a model part, the helper must traverse parent region indices, asserting each parent index is non-negative and valid.
- For each parent, the helper must find parent extents by parent volume id, panic if missing, and extend the output bbox by the parent bbox.
- Traversal must stop immediately after extending a parent whose volume is a model part.
- If a parent is also not a model part, traversal must continue through that parent's `parent` field.
- Add unit tests for model-part direct return, one modifier parent extension, multi-level modifier parent chain extension, missing current extents panic, missing parent extents panic, and invalid parent panic.
- Do not implement real `PrintObjectRegions::LayerRangeRegions`, real `VolumeRegion` pointers, `ModelVolume` pointers, `PrintRegion`, painted/fuzzy regions, region config merging, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
