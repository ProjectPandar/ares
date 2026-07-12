# Spec: M299 PrintApply verify-update existing region ref increment

## Goal

Port the existing-region ref-count increment at the end of OrcaSlicer's `verify_update_print_object_regions(...)` existing volume-region branch into `ares-core` as private staged sequencing state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:809`: `print_region_ref_inc(*region.region);`

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729`: `print_region_ref_inc(PrintRegion &region)` increments `m_ref_cnt`.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` owns `m_ref_cnt`, and `print_region_ref_inc(...)` is a friend helper.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:796-806`: changed configs either update in place for zero-ref regions or return `false` for referenced regions before the increment line.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:803`: update-in-place config application happens before the increment.
- M295/M298 staged state: `StagedExistingRegionUpdateAction` and `StagedExistingRegionConfigApply` already represent the branch decision and staged apply result.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M287 `StagedPrintRegionRefCount` and `staged_print_region_ref_inc(...)` for mutation.
- Reuse M295 `StagedExistingRegionUpdateAction` and M298 `StagedExistingRegionConfigApply` for sequencing.
- Add a private staged increment result carrying the ref count after increment.
- Add a helper that accepts the update action, optional staged apply result, and a mutable staged region ref count.
- For `Unchanged`, increment immediately and return the post-increment result.
- For `UpdateInPlace`, increment only when the staged config apply result is present, then return the post-increment result.
- For `UpdateInPlace` without a staged apply result, return no increment result and leave the count unchanged.
- For `RequiresReslice`, return no increment result and leave the count unchanged.
- Preserve accumulation by incrementing from the current staged count rather than replacing it.
- Defer real `PrintRegion`, real `PrintObjectRegions`, branch loop integration, missing-override creation, painted/fuzzy painted regions, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Unchanged action increments a zero-ref existing region to one.
- Unchanged action increments an already referenced existing region by one.
- Update-in-place action increments only when a staged config apply result is present.
- Update-in-place action without staged config apply result returns no increment and leaves count unchanged.
- Requires-reslice action returns no increment and leaves count unchanged.
- Increment result records the post-increment count.

## Migration note

This milestone stages `PrintApply.cpp:809` sequencing only. Later milestones must integrate this staged helper into a real `verify_update_print_object_regions(...)` port and then continue to painted/fuzzy painted region handling as source-cited slices.
