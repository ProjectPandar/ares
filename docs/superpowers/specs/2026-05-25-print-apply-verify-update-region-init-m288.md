# PrintApply verify-update region initialization Spec

## Goal

Port the initialization prefix of OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as a private staged helper for later region-validation milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:743-747`: sort incoming model volumes by id, then reset every existing `PrintRegion` ref count in `print_object_regions.all_regions`.

Required context:
- `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`: `model_volumes_sort_by_id(...)` sorts `ModelVolumePtrs` by `ModelVolume::id()`.
- `OrcaSlicer/src/libslic3r/Print.hpp:291-296`: `PrintObjectRegions` owns `all_regions` and `cached_volume_ids` state.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`: M287 staged `print_region_ref_reset(...)` sets `m_ref_cnt` to zero.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add private staged model-volume state carrying a volume id for the sort prefix.
- Add a private staged helper that accepts mutable model volumes and mutable print-region ref-count state.
- The helper must sort model volumes in-place by ascending id before resetting regions.
- The helper must reset every existing staged print-region ref count to zero using the M287 reset helper.
- The helper must accept empty model-volume and region slices without panicking.
- Tests must prove unsorted model volumes become sorted by id, already sorted ids remain sorted, duplicate ids are grouped by id without asserting same-id order, multiple nonzero region counts reset to zero, and empty inputs are accepted.
- Keep all new types/functions private or `pub(super)` only for tests.
- Do not implement the layer-range loop, model-part/modifier filtering, lower-bound model-volume lookup, modifier override detection, config diff/apply, callback invalidation, painted/fuzzy painted regions, reslice return decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
