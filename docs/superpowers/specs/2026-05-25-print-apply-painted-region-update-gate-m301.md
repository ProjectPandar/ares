# Spec: M301 PrintApply painted region update gate

## Goal

Port OrcaSlicer's color-painted region config comparison and ref-count update/reslice gate from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:821-831`: `if (cfg != region.region->config())`, then update in place only when `print_region_ref_cnt(*region.region) == 0`; otherwise return `false` for reslice.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:813-820`: M300 derives the painted config from parent config and painted extruder id before this comparison.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:796-806`: existing-region branch uses the same changed/zero-ref/update-in-place vs changed/referenced/reslice decision shape.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`: `print_region_ref_cnt(...)` reads `PrintRegion::m_ref_cnt`.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` stores config and ref count.
- `OrcaSlicer/src/libslic3r/Print.hpp:243-252`: `PaintedRegion` points to the painted `PrintRegion` being compared/updated.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Use the existing M300 `StagedPaintedRegionConfig` as both current and derived painted-region config shape; keep the M300 parent-indexed derivation helper intact.
- Add a staged comparison result carrying `painted_region_id`, `current_config`, `derived_config`, and `config_changed`.
- Add a helper that compares current and derived painted-region configs.
- Add a helper that maps the comparison result plus `StagedPrintRegionRefCount` to `StagedExistingRegionUpdateAction`.
- Return `Unchanged` when configs are equal regardless of ref count.
- Return `UpdateInPlace` when configs differ and ref count is zero.
- Return `RequiresReslice` when configs differ and ref count is nonzero.
- Do not perform diff-key collection, callback invalidation, config apply-only, ref-count increment, fuzzy painted-region handling, real `PrintRegionConfig`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Equal current/derived painted configs yield `Unchanged` for zero-ref regions.
- Equal current/derived painted configs yield `Unchanged` for referenced regions.
- Different current/derived painted configs yield `UpdateInPlace` for zero-ref regions.
- Different current/derived painted configs yield `RequiresReslice` for referenced regions.
- Comparison result preserves painted region id plus current and derived configs.

## Migration note

This milestone stages the decision portion of `PrintApply.cpp:821-831` only. Later milestones must wire the painted-region diff/callback/apply sequence from `PrintApply.cpp:826-828`, the painted-region ref increment at `PrintApply.cpp:834`, and fuzzy painted regions at `PrintApply.cpp:837-856` as separate source-cited rewrite slices.
