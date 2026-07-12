# M307: PrintApply region merge verification

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the final region-merge verification block in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:860-875`: collect all `print_object_regions.all_regions`, assert every region has a positive `print_region_ref_cnt`, sort regions by `config_hash()`, scan adjacent equal-hash groups, and return `false` when any pair in a same-hash group has equal `config()`. Required context comes from the preceding region ref-count increments at `PrintApply.cpp:809`, `PrintApply.cpp:834`, and `PrintApply.cpp:856`, the ref-count helpers at `PrintApply.cpp:729-731`, and `PrintRegion::config()`, `config_hash()`, and `m_ref_cnt` in `OrcaSlicer/src/libslic3r/Print.hpp:104-149`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region-deduplication or slicing pipeline.

## Exit criteria

- Add private staged region merge verification for `PrintApply.cpp:860-875`.
- Preserve that every all-regions entry must have a positive ref count before merge verification.
- Preserve sorting by config hash before comparing config equality.
- Preserve that equal configs within the same config-hash group require reslice.
- Preserve that hash collisions with unequal configs do not require reslice.
- Preserve that equal configs with different hashes are not compared by this upstream block.
- Preserve success for empty or uniquely configured all-regions lists.
- Add tests for empty input, unique regions, equal config same hash requiring reslice, hash collision unequal config no reslice, equal config different hash no reslice, sorting before comparison, and zero-ref assertion/panic.
- Defer real `PrintRegion`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
