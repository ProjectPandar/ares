# M306: PrintApply fuzzy painted region ref increment

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the fuzzy-skin painted-region reference increment in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:856`: after a fuzzy-painted region is unchanged or updated in place, call `print_region_ref_inc(*region.region)`. Required context comes from M304's fuzzy-painted config derivation at `PrintApply.cpp:837-842`, M305's fuzzy-painted comparison/update/apply block at `PrintApply.cpp:843-853`, the shared `print_region_ref_inc(...)` helper in `PrintApply.cpp:729`, the color-painted ref increment at `PrintApply.cpp:834`, the existing-region ref increment at `PrintApply.cpp:809`, and `PrintRegion::m_ref_cnt` / friend helper access in `OrcaSlicer/src/libslic3r/Print.hpp:104-149`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned fuzzy-skin or slicing pipeline.

## Exit criteria

- Add private staged fuzzy-painted ref increment sequencing for `PrintApply.cpp:856`.
- Preserve that unchanged fuzzy-painted regions increment the destination region ref count.
- Preserve that changed zero-ref regions increment only after a staged update-in-place apply exists.
- Preserve that changed referenced regions require reslice and do not increment because upstream returns before line 856.
- Preserve accumulated ref-count mutation through the existing staged ref-count helper.
- Move fuzzy-painted staged state out of `painted_region_state.rs` into a separate private module because the current file is at the 400 LOC split threshold; keep behavior unchanged during the move.
- Add focused tests for unchanged increment, update-in-place increment with apply, update-in-place skip without apply, requires-reslice skip, accumulated increments, and continued M304/M305 behavior after module split.
- Defer the region merge verification block after `PrintApply.cpp:860`, real `PrintRegion`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
