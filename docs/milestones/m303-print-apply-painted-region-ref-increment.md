# M303: PrintApply painted region ref increment

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the color-painted region reference-count increment in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:834`: after the painted-region config comparison/update branch completes without returning `false`, call `print_region_ref_inc(*region.region)`. Required context comes from the painted-region update gate at `PrintApply.cpp:821-831`, the painted-region diff/invalidate/apply sequence at `PrintApply.cpp:826-828`, `print_region_ref_inc(...)` helper context at `PrintApply.cpp:729`, the existing-region increment at `PrintApply.cpp:809`, and `PrintRegion::m_ref_cnt` / helper access context in `OrcaSlicer/src/libslic3r/Print.hpp:104-149`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned painted-region or slicing pipeline.

## Exit criteria

- Add private staged painted-region ref-increment sequencing for line `PrintApply.cpp:834`.
- Preserve unchanged painted-region behavior: increment ref count.
- Preserve changed zero-ref update-in-place behavior: increment only after staged config apply exists.
- Preserve changed nonzero-ref/reslice behavior: no increment because upstream returns `false` before line 834.
- Preserve accumulated ref count mutation through the existing staged `PrintRegion` ref-count helper.
- Reuse existing private `StagedExistingRegionRefIncrement` and `StagedExistingRegionUpdateAction` vocabulary instead of introducing an Ares-specific pipeline state.
- Add tests for unchanged increment, update-in-place increment after apply, update-in-place no increment without apply, requires-reslice no increment, and accumulated count behavior.
- Defer fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegion`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
