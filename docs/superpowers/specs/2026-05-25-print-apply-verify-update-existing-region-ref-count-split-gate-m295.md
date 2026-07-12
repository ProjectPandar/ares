# Spec: M295 PrintApply verify-update existing region ref-count split gate

## Goal

Port the changed existing-region ref-count decision from OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:798-806`: after `cfg != region.region->config()`, update in place only when `print_region_ref_cnt(*region.region) == 0`; otherwise return false because the region would be split.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:796`: M294 staged config-change predicate.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`: M287 staged `print_region_ref_cnt(...)` helper.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion::m_ref_cnt` and config mutation context.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M294 `StagedExistingRegionConfigChange` and M287 `StagedPrintRegionRefCount` / `staged_print_region_ref_cnt`.
- Add a private staged action enum with exactly these states: unchanged, update in place, requires reslice.
- Add a helper that accepts the M294 config-change result and the current staged print-region ref count.
- If the config-change result is unchanged, return unchanged regardless of ref count.
- If the config changed and ref count is zero, return update in place.
- If the config changed and ref count is nonzero, return requires reslice.
- Defer config diff key collection, callback invalidation, config apply, ref-count increment, derived config source selection, real config merge internals, real `PrintRegion`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Equal current/derived staged configs with zero ref count return unchanged.
- Differing current/derived staged configs with zero ref count return update in place.
- Differing current/derived staged configs with positive ref count return requires reslice.
- Equal current/derived staged configs with positive ref count still return unchanged.

## Migration note

This milestone is a staged compatibility shell around the existing-region ref-count branch in `PrintApply.cpp:798-806`. It does not compute config diffs, call invalidation callbacks, mutate configs, or create an Ares-owned pipeline; later milestones must port `PrintApply.cpp:801-803` as source-cited slices.
