# PrintApply keep reusable cached volume ids Spec

## Goal

Port the private cache-retention control flow from OrcaSlicer's `print_objects_regions_invalidate_keep_some_volumes(...)` into `ares-core` as a staged private helper for later print-object-region invalidation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:664-695`: clear `all_regions`, sort old/new volumes by id, iterate solid/modifier new volumes, match old volumes by id, reuse only transform-equivalent volumes, compact retained cached volume ids, and erase the tail.

Required context:
- `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`: `model_volumes_sort_by_id(...)` sorts model-volume pointers by `id()`.
- `OrcaSlicer/src/libslic3r/ObjectID.hpp:20-37`: `ObjectID` wraps ordered `size_t` ids.
- `OrcaSlicer/src/libslic3r/Print.hpp:291-296`: `PrintObjectRegions` owns `all_regions` and `cached_volume_ids`.
- `OrcaSlicer/src/libslic3r/Model.hpp:340-348`: `ModelVolumeType` variants.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:542-546`: `model_volume_solid_or_modifier(...)` accepts only model part, negative volume, and parameter modifier.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add private staged state for the subset needed by this source boundary: cached volume ids, an all-regions count/list that can be cleared, and old/new staged volume records.
- Each staged volume record must include id, `StagedModelVolumeType`, and a transform compatibility marker sufficient to model `old.get_matrix().isApprox(new.get_matrix())`.
- Add a private helper equivalent to `print_objects_regions_invalidate_keep_some_volumes(regions, old_volumes, new_volumes)` for staged records.
- The helper must clear staged all-regions state before matching volumes.
- The helper must sort old and new volume records by id before matching.
- The helper must ignore new volumes whose type is not accepted by `staged_model_volume_solid_or_modifier`.
- The helper must preserve upstream monotonic old-volume scan: do not restart scanning old volumes for each new volume.
- The helper must keep a cached id only when an old volume with the same id exists and the old/new transform compatibility marker matches.
- When keeping a volume, the helper must scan cached ids forward until the matched id and panic/assert if the matched id is absent.
- The helper must compact kept cached ids at the front in discovered order and truncate the rest after processing.
- The helper must skip unmatched ids, transform-changed ids, and non-solid/modifier new volumes.
- Add unit tests for clearing all-regions, sorted input matching, non-solid filtering, transform-changed skip, missing cached id panic, compact/truncate behavior, and preservation of duplicate unrelated cached ids only when explicitly kept by the upstream scan.
- Do not implement real `PrintObjectRegions`, `PrintRegion`, `ModelVolume` pointers, Eigen transforms or `isApprox`, bbox recomputation, layer-range region rebuilding, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
