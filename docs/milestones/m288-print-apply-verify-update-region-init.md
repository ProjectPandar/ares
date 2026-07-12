# M288: PrintApply verify-update region initialization

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the initialization prefix of `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:743-747`, with `model_volumes_sort_by_id(...)` context from `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`, `PrintObjectRegions::all_regions` context from `OrcaSlicer/src/libslic3r/Print.hpp:291-296`, and M287 `print_region_ref_reset(...)` context from `PrintApply.cpp:729-731`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to the initialization prefix of `verify_update_print_object_regions(...)`.
- Preserve in-place sorting of model volumes by ascending volume id.
- Preserve resetting every existing print-region ref count after sorting.
- Preserve duplicate-id ordering as unspecified beyond Rust stable sort behavior not being relied on by tests.
- Preserve empty input behavior for both model volumes and all regions.
- Add tests for unsorted model volume ordering, already-sorted preservation by id, duplicate id grouping, resetting multiple nonzero print-region ref counts, and accepting empty inputs.
- Defer the `layer_ranges` loop, model-part/modifier filtering, model-volume lower-bound lookup, modifier override detection, config diff/apply, callback invalidation, painted/fuzzy painted regions, return-value reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
