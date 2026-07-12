# M286: PrintApply find modifier volume extents

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `find_modifier_volume_extents(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:705-725`, with predecessor `find_volume_extents(...)` context from `PrintApply.cpp:697-703`, `PrintObjectRegions::VolumeRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, `PrintObjectRegions::LayerRangeRegions::volumes` / `volume_regions` context from `Print.hpp:271-282`, `ModelVolume::is_model_part()` context from `OrcaSlicer/src/libslic3r/Model.hpp:901-907`, and bounding-box context from `Print.hpp:216-223`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to `find_modifier_volume_extents(...)` over staged volume regions and staged volume extents.
- Preserve lookup of the current region by `this_region_id` and lookup of its own volume extents via M285 `find_volume_extents` semantics.
- Preserve panic/assert behavior when current or parent volume extents are missing, or when a modifier parent id is invalid.
- Preserve starting output bbox as the current region's bbox.
- Preserve no parent traversal when the current region's model volume is a model part.
- Preserve parent traversal for modifier/non-model-part regions: extend output with each parent region bbox until a model-part parent is reached.
- Preserve updating `parent_region_id = parent_region.parent` after each non-model-part parent.
- Add tests for model-part direct return, single parent extension, multi-level parent extension, missing current extent panic, missing parent extent panic, and invalid parent panic.
- Defer real `PrintObjectRegions::LayerRangeRegions`, real `VolumeRegion` pointers, `ModelVolume` pointers, `PrintRegion`, painted/fuzzy regions, region config merging, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
