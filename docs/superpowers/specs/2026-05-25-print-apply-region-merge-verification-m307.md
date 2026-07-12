# Spec: M307 PrintApply region merge verification

## Goal

Port OrcaSlicer's final region-merge verification block from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:860-875`: collect all regions, assert positive refs, sort by config hash, compare configs only inside equal-hash groups, and require reslice if two regions have equal configs.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:809`, `PrintApply.cpp:834`, and `PrintApply.cpp:856`: volume, color-painted, and fuzzy-painted verification paths increment region refs before final merge verification.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`: helper functions increment, reset, and read `PrintRegion::m_ref_cnt`.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` exposes `config()`, `config_hash()`, and owns `m_ref_cnt`.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a private staged region record carrying a region id, config fingerprint, config hash, and ref count.
- Add a private staged merge-verification result that distinguishes valid/no-reslice from merged/requires-reslice.
- Add a helper that panics/asserts when any staged region has a non-positive ref count, matching upstream `assert(print_region_ref_cnt(*region) > 0)`.
- Sort staged regions by config hash before scanning.
- Compare config equality only within equal-hash runs.
- Return requires-reslice if any two regions in one hash run have equal config fingerprints.
- Return valid/no-reslice for empty input, one region, unique configs, same-hash unequal configs, and equal configs with different hashes.
- Do not perform real `PrintRegion`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Empty all-regions list is valid.
- Unique referenced regions are valid.
- Equal config fingerprints with the same config hash require reslice.
- Same hash with unequal config fingerprints is valid.
- Equal config fingerprints with different hashes are valid because upstream only compares same-hash groups.
- Unsorted input is sorted before same-hash comparison.
- Zero-ref or negative-ref region panics before merge verification.

## Migration note

This milestone stages `PrintApply.cpp:860-875` only. Later milestones must continue with the next source-cited `PrintApply` or `libslic3r` boundary rather than designing independent Ares merge/deduplication behavior.
