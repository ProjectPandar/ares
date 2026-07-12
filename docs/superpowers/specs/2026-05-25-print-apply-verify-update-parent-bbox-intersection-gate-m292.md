# Spec: M292 PrintApply verify-update parent bbox intersection gate

## Goal

Port the parent bbox intersection gate for missing modifier overrides from OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged state for the later config-comparison/reslice milestone.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:783-789`: when no existing override is found for the current modifier and scanned parent, compute `find_modifier_volume_extents(layer_range, parent_region_id)` and gate later config comparison on `parent_bbox.intersects(*bbox)`.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:767-768`: current modifier bbox comes from `find_volume_extents(layer_range, *region.model_volume)` and is asserted present before the parent scan.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:705-725`: `find_modifier_volume_extents(...)` computes parent bbox, including modifier parent-chain extension.
- `OrcaSlicer/src/libslic3r/Print.hpp:216-240`: `BoundingBox`, `VolumeExtents`, and `VolumeRegion` fields used by the bbox lookup/intersection.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-282`: `LayerRangeRegions::volumes` and `volume_regions` ownership/order context.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M285/M286 `StagedExtentBox`, `StagedVolumeExtents`, `StagedVolumeRegion`, and `staged_find_modifier_volume_extents(...)` rather than inventing another bbox representation.
- Add an intersection helper or method matching Eigen `AlignedBox::intersects` semantics for closed boxes: boxes that overlap or touch on every axis intersect; separated boxes on any axis do not.
- Add a staged parent-bbox gate result carrying `parent_region_id`, `parent_bbox`, `current_modifier_bbox`, and `intersects`.
- Add a private helper that takes `volume_regions`, `volume_extents`, `current_modifier_bbox`, and `parent_region_id`, computes the parent bbox using `staged_find_modifier_volume_extents(...)`, and records whether it intersects the current bbox.
- Defer `region_config_from_model_volume(...)`, config comparison, returning `false`, callback invalidation, ref-count increment, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Intersecting parent/current bboxes return `intersects = true` and preserve both boxes in the result.
- Disjoint parent/current bboxes return `intersects = false`.
- Boxes touching at a boundary count as intersecting.
- A model-part parent region uses direct bbox lookup through `staged_find_modifier_volume_extents(...)`.
- A modifier parent region extends through its parent chain via the existing helper before intersection.
- Missing parent extents panic via the existing helper.

## Migration note

This milestone is a staged continuation of M285/M286/M290. It does not replace the Ares scaffold or create an Ares-owned pipeline; it adds the next source-cited private state transition needed to model Orca `verify_update_print_object_regions(...)` behavior.
