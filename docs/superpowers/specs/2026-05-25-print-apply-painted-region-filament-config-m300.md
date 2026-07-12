# Spec: M300 PrintApply painted region extruder config

## Goal

Port OrcaSlicer's color-painted region extruder config derivation prefix from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:813-820`: painted-region loop prefix copies the parent region config and overwrites wall, solid infill, and sparse infill filament fields with `region.extruder_id`.

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:243-252`: `PrintObjectRegions::PaintedRegion` carries `parent` and `extruder_id`.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-283`: `LayerRangeRegions` owns `painted_regions`.
- `OrcaSlicer/src/libslic3r/Print.hpp:229-240`: `VolumeRegion` references the parent `PrintRegion`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1121,1154,1161`: `wall_filament`, `solid_infill_filament`, and `sparse_infill_filament` fields.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add private staged parent config, painted-region input, and derivation records carrying the parent index, painted-region id, and three filament fields plus a marker for preservation tests.
- Add a private staged helper that accepts parent configs and painted-region inputs.
- Return source-order derivations that copy each referenced parent config and overwrite `wall_filament`, `solid_infill_filament`, and `sparse_infill_filament` by the painted-region extruder id.
- Preserve unrelated parent config fields and the painted-region/parent ids.
- Do not model comparison/update/reslice handling, ref-count increment, fuzzy painted regions, real `PrintRegionConfig`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Painted-region derivation overwrites `wall_filament`.
- Painted-region derivation overwrites `solid_infill_filament`.
- Painted-region derivation overwrites `sparse_infill_filament`.
- Painted-region derivation preserves an unrelated marker field from the parent config.
- Multiple painted regions produce source-order derived configs from their referenced parents.

## Migration note

This milestone stages `PrintApply.cpp:813-820` only. Later milestones must continue with the comparison/update/reslice branch at `PrintApply.cpp:821-831`, the painted-region ref increment at `PrintApply.cpp:834`, and fuzzy painted regions at `PrintApply.cpp:837-856` as separate source-cited rewrite slices.
