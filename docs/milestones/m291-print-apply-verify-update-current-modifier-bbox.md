# M291: PrintApply verify-update current modifier bbox lookup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the current-modifier bbox lookup/assert inside the first-modifier branch of `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:767-768`, with M285 `find_volume_extents(...)` context from `PrintApply.cpp:697-703`, `VolumeExtents` context from `OrcaSlicer/src/libslic3r/Print.hpp:224-228`, and `LayerRangeRegions::volumes` context from `Print.hpp:271-278`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to `const BoundingBox *bbox = find_volume_extents(layer_range, *region.model_volume); assert(bbox);` for the current first-visited modifier region.
- Preserve exact volume-id lookup through the M285 staged lower-bound semantics.
- Preserve returning the referenced bbox when the current modifier volume id exists.
- Preserve assert/panic behavior when the current modifier volume id is missing from sorted volume extents.
- Add tests for returning the exact current modifier bbox, panicking on missing current modifier extents, and using exact id lookup rather than nearest lower-bound neighbors.
- Defer parent bbox lookup/intersection from `PrintApply.cpp:783`, `find_modifier_volume_extents(...)` integration into verify-update, config derivation/comparison/application, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
