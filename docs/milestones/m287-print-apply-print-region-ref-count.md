# M287: PrintApply print region ref count helpers

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `print_region_ref_inc(...)`, `print_region_ref_reset(...)`, and `print_region_ref_cnt(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`, with `PrintRegion` friend/helper context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149` and `verify_update_print_object_regions(...)` reset context from `PrintApply.cpp:746-747`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add private staged `PrintRegion` ref-count state in `ares-core` for the three helper operations.
- Preserve `print_region_ref_inc(PrintRegion &r)` semantics: increment the region ref count by one.
- Preserve `print_region_ref_reset(PrintRegion &r)` semantics: reset the region ref count to zero.
- Preserve `print_region_ref_cnt(const PrintRegion &r)` semantics: return the current count without mutation.
- Preserve signed `int`-like count behavior with `i32` staged state.
- Add tests for default zero count, increment accumulation, reset after increments, count read without mutation, and reset idempotence.
- Defer real `PrintRegion`, `PrintRegionConfig`, config hash/equality, `PrintObjectRegions::all_regions`, region validation, merging/splitting, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
