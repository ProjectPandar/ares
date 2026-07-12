# Spec: M304 PrintApply fuzzy painted region config

## Goal

Port OrcaSlicer's fuzzy-skin painted-region config derivation prefix from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:837-842`: iterate fuzzy-skin painted regions, resolve the parent print region, copy parent config, and set fuzzy skin to `All` unless the parent config is `Disabled_fuzzy`.

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:255-266`: `FuzzySkinPaintedRegion` carries `ParentType`, parent index, region pointer, and parent lookup helpers.
- `OrcaSlicer/src/libslic3r/Print.hpp:271-283`: `LayerRangeRegions` owns `volume_regions`, `painted_regions`, and `fuzzy_skin_painted_regions`.
- `OrcaSlicer/src/libslic3r/Print.cpp:4932-4947`: parent lookup chooses `painted_regions[parent].region` for painted parents and `volume_regions[parent].region` for volume parents.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-57`: `FuzzySkinType` enum variants include `All` and `Disabled_fuzzy`.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add staged fuzzy skin type values matching the upstream variants needed for this milestone.
- Add staged fuzzy-painted parent references for `VolumeRegion` and `PaintedRegion` parents.
- Add staged parent config records carrying a parent print-region id and fuzzy-skin value.
- Add staged fuzzy-painted region inputs carrying fuzzy region id, parent reference, and destination painted print-region id.
- Add a helper that resolves parent config from either parent collection, copies it, normalizes fuzzy skin to `All` for any non-`DisabledFuzzy` value, and returns source-order derivations.
- Preserve `DisabledFuzzy` without changing it to `All`.
- Preserve source-order derivation output and region/parent metadata.
- Do not perform config comparison/update/reslice handling, ref-count increment, real `PrintRegionConfig`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Fuzzy-painted config derives from a volume-region parent.
- Fuzzy-painted config derives from a painted-region parent.
- Non-disabled parent fuzzy-skin values normalize to `All`.
- Disabled parent fuzzy-skin values remain disabled.
- Multiple fuzzy-painted regions are derived in source order from their referenced parents.
- Parent config collections are not mutated.

## Migration note

This milestone stages `PrintApply.cpp:837-842` only. Later milestones must continue with fuzzy-painted config comparison/update/apply at `PrintApply.cpp:843-853` and fuzzy-painted ref increment at `PrintApply.cpp:856` as separate source-cited rewrite slices.
