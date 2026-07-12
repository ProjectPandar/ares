# Spec: M303 PrintApply painted region ref increment

## Goal

Port OrcaSlicer's color-painted region ref-count increment from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:834`: `print_region_ref_inc(*region.region);` after the painted-region update/reslice branch.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:821-831`: M301 gates changed painted-region configs; changed referenced regions return `false` before line 834.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:826-828`: M302 stages diff/invalidate/apply for changed zero-ref update-in-place painted regions before line 834.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729`: `print_region_ref_inc(...)` increments `PrintRegion::m_ref_cnt`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:809`: existing-region branch increments after its config update branch with the same sequencing shape.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` owns `m_ref_cnt` and grants helper access.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse `StagedExistingRegionUpdateAction`, `StagedExistingRegionConfigApply`, and `StagedExistingRegionRefIncrement` where practical.
- Add a painted-region ref-increment helper that takes the M301 action, optional M302 apply result, and mutable staged ref-count region.
- Increment for `Unchanged` actions.
- Increment for `UpdateInPlace` actions only when the staged apply result exists.
- Return no increment for `UpdateInPlace` without apply.
- Return no increment for `RequiresReslice` actions.
- Preserve accumulated counts by mutating the provided `StagedPrintRegionRefCount` through the existing helper.
- Do not model fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, loop integration, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Unchanged painted-region action increments a zero-ref region.
- Update-in-place painted-region action increments only when apply state exists.
- Update-in-place painted-region action without apply does not increment.
- Requires-reslice painted-region action does not increment.
- Repeated increment calls accumulate the staged ref count.

## Migration note

This milestone stages `PrintApply.cpp:834` only. Later milestones must continue with fuzzy painted regions at `PrintApply.cpp:837-856` as source-cited rewrite slices.
