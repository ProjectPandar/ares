# PrintApply print region ref count helpers Spec

## Goal

Port OrcaSlicer's private `PrintRegion` ref-count helper functions into `ares-core` as staged private state used by later `verify_update_print_object_regions(...)` milestones.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`: `print_region_ref_inc`, `print_region_ref_reset`, and `print_region_ref_cnt` directly mutate or read `PrintRegion::m_ref_cnt`.

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` owns private `int m_ref_cnt { 0 }` and declares the three helper functions as friends.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:746-747`: `verify_update_print_object_regions(...)` resets all existing print-region ref counts before validating/updating regions.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add private staged `PrintRegion` ref-count state with an initial count of zero.
- Use signed `i32` for the staged count to match Orca's `int m_ref_cnt` role.
- Add `staged_print_region_ref_inc(&mut region)` that increments the count by one.
- Add `staged_print_region_ref_reset(&mut region)` that sets the count to zero.
- Add `staged_print_region_ref_cnt(&region) -> i32` that returns the current count without mutating it.
- Tests must prove default zero count, repeated increments, reset after increments, count read without mutation, and repeated reset staying zero.
- Keep all new types/functions private or `pub(super)` only for tests.
- Do not implement real `PrintRegion`, `PrintRegionConfig`, config hash/equality, region validation, region merging/splitting, public APIs, profile loading, UI runtime behavior, slicing, extrusion, G-code generation, new crates, new dependencies, or independent Ares pipeline behavior.
