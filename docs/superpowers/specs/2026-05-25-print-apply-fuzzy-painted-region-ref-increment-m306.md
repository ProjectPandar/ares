# Spec: M306 PrintApply fuzzy painted region ref increment

## Goal

Port OrcaSlicer's fuzzy-skin painted-region ref-count increment from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:856`: call `print_region_ref_inc(*region.region)` after fuzzy-painted config derivation and update/apply handling completes without requiring reslice.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:837-842`: M304 fuzzy-painted config derivation.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:843-853`: M305 fuzzy-painted update/apply block returns before the increment when changed referenced regions require reslice.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729`: `print_region_ref_inc(PrintRegion &r)` increments `m_ref_cnt`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:809` and `PrintApply.cpp:834`: existing-region and color-painted region loops increment after successful verification/update.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` ref-count storage and helper friend access.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Move existing fuzzy-painted staged types/helpers from `painted_region_state.rs` into a new private `fuzzy_painted_region_state.rs` module so both files stay below the 400 LOC split threshold.
- Update tests to import fuzzy-painted staged items from the new module.
- Add a fuzzy-painted `staged_fuzzy_painted_region_ref_inc(action, apply, region)` helper that delegates to the existing staged ref-increment behavior.
- Preserve upstream sequencing:
  - `Unchanged` increments the destination region ref count without an apply record.
  - `UpdateInPlace` increments only when an apply record exists.
  - `UpdateInPlace` without apply does not increment.
  - `RequiresReslice` does not increment because upstream returns before `PrintApply.cpp:856`.
- Preserve accumulated ref-count mutation across multiple successful fuzzy-painted increments.
- Do not perform the region merge verification block after `PrintApply.cpp:860`, real `PrintRegion`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Existing M304 fuzzy-painted derivation tests still pass after moving state to the new private module.
- Existing M305 fuzzy-painted update/apply tests still pass after moving state to the new private module.
- Fuzzy-painted ref increment increments unchanged zero-ref region.
- Fuzzy-painted ref increment updates in place when apply exists.
- Fuzzy-painted ref increment skips update-in-place without apply.
- Fuzzy-painted ref increment skips requires-reslice action.
- Fuzzy-painted ref increment accumulates successful increments.

## Migration note

This milestone stages `PrintApply.cpp:856` only. Later milestones must continue with the region merge verification block after `PrintApply.cpp:860` as a separate source-cited rewrite slice.
