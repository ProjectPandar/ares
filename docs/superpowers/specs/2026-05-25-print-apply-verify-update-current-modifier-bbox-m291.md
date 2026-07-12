# PrintApply verify-update current modifier bbox lookup Spec

## Goal

Port the current first-visited modifier bbox lookup/assert from OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged state for later parent-bbox intersection milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:767-768`: call `find_volume_extents(layer_range, *region.model_volume)` for the current modifier and assert the result exists.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:697-703`: `find_volume_extents(...)` lower-bound exact-id lookup, staged in M285.
- `OrcaSlicer/src/libslic3r/Print.hpp:224-228`: `VolumeExtents` stores the bbox payload.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-278`: `LayerRangeRegions::volumes` stores sorted volume extents.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:760-766` and M289/M290: this lookup happens only inside first modifier visits before parent scanning/intersection.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M285 staged `StagedVolumeExtents`, `StagedExtentBox`, and `staged_find_volume_extents` where practical.
- Add a private staged helper that accepts sorted staged volume extents and the current modifier volume id.
- The helper must use exact lower-bound lookup semantics and panic if no extents exist for the current modifier volume id.
- The helper must return the current modifier bbox value when found.
- Tests must prove exact bbox return, missing-current-extents panic, and no nearest-neighbor fallback for ids between sorted extents.
- Keep all new types/functions private or `pub(super)` only for tests.
- Do not implement parent bbox lookup/intersection, `find_modifier_volume_extents(...)` verify-update integration, config derivation/comparison/application, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
