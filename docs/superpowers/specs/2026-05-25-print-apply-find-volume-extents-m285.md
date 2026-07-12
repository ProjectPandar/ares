# PrintApply find volume extents lookup Spec

## Goal

Port OrcaSlicer's private `find_volume_extents(...)` lookup into `ares-core` as a staged private helper for later modifier-volume and print-object-region milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:697-703`: lower-bound lookup in `layer_range.volumes` by `volume.id()` and return bbox pointer only for an exact id match.

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:224-228`: `PrintObjectRegions::VolumeExtents` stores `volume_id` and `bbox`.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-278`: `LayerRangeRegions::volumes` is sorted by `ModelVolume::id()`.
- `OrcaSlicer/src/libslic3r/ObjectID.hpp:20-37`: `ObjectID` wraps ordered `size_t` ids.
- `OrcaSlicer/src/libslic3r/libslic3r.h:230-247`: `lower_bound_by_predicate(...)` finds the first element whose predicate is false.
- `OrcaSlicer/src/libslic3r/Print.hpp:216-223`: `PrintObjectRegions::BoundingBox` is a 3D f32 bbox.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse the staged f32 bounding box shape from M282/M283 where practical, or add an equivalent private bbox record inside the same private staging boundary.
- Add a private staged `VolumeExtents` equivalent with `volume_id: u64` and bbox payload.
- Add a private helper equivalent to `find_volume_extents(layer_range, volume_id)` over a sorted extent slice.
- The helper must use lower-bound semantics: find the first record whose `volume_id >= requested_id`.
- The helper must return `Some(&bbox)` only if the found record exists and has exactly the requested id.
- The helper must return `None` for empty extents, requested ids below a different first id, between existing ids, or above the last id.
- For duplicate ids, the helper must return the first equal record, matching lower-bound behavior.
- Add unit tests for exact match, empty extents, below-first mismatch, between-id mismatch, above-last miss, and duplicate first-equal behavior.
- Do not implement real `PrintObjectRegions::LayerRangeRegions`, `ModelVolume`, `ObjectID` wrapper types, bbox clipping/recalculation, modifier extents, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
