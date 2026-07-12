# M301: PrintApply painted region update gate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the color-painted region config comparison and ref-count update/reslice decision in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:821-831`: compare the derived painted config against `region.region->config()`, update in place only when `print_region_ref_cnt(*region.region) == 0`, and require reslice for changed configs on already-referenced regions. Required context comes from the painted-region config derivation prefix at `PrintApply.cpp:813-820`, the earlier existing-region update gate at `PrintApply.cpp:796-806`, `print_region_ref_cnt(...)` at `PrintApply.cpp:729-731`, `PrintObjectRegions::PaintedRegion` in `OrcaSlicer/src/libslic3r/Print.hpp:243-252`, and `PrintRegion::m_ref_cnt` / config mutation context in `Print.hpp:104-149`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned painted-region or slicing pipeline.

## Exit criteria

- Add a private staged painted-region update gate for comparing current painted-region config against the M300-derived painted config.
- Preserve unchanged config behavior: no update branch and no reslice requirement.
- Preserve changed zero-ref behavior: update in place is allowed.
- Preserve changed nonzero-ref behavior: reslice is required.
- Preserve that the painted region id and both current/derived configs are carried in the staged comparison result for later diff/callback/apply wiring.
- Reuse the existing private `StagedExistingRegionUpdateAction` vocabulary instead of introducing an Ares-specific pipeline state.
- Add tests for unchanged zero-ref, unchanged referenced, changed zero-ref, changed referenced, and comparison-result payload preservation.
- Defer concrete diff-key collection from `PrintApply.cpp:826`, invalidate callback from `PrintApply.cpp:827`, config apply-only from `PrintApply.cpp:828`, painted-region ref increment from `PrintApply.cpp:834`, fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
