# Spec: M305 PrintApply fuzzy painted region update/apply

## Goal

Port OrcaSlicer's fuzzy-skin painted-region config comparison and update/apply block from `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:843-853`: compare derived fuzzy-painted config with `region.region->config()`, update in place only for zero-ref destination regions, run diff/invalidate/apply sequencing, and require reslice when the destination region is already referenced.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:837-842`: M304 derives the fuzzy-painted config by resolving the parent and normalizing fuzzy skin.
- `OrcaSlicer/src/libslic3r/Print.hpp:255-266`: `FuzzySkinPaintedRegion` carries parent metadata and destination `PrintRegion *region`.
- `OrcaSlicer/src/libslic3r/Print.hpp:104-149`: `PrintRegion` exposes config access, config apply, and ref-count storage used by `print_region_ref_cnt(...)`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:786-800` and `PrintApply.cpp:821-833`: existing-region and color-painted region update blocks use the same diff, invalidation, apply-only, and reslice gate pattern.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Put M305 tests in a separate focused test module so existing Rust files stay below the 400 LOC split threshold.
- Reuse M304 staged fuzzy-painted config derivations as the derived config input.
- Add staged fuzzy-painted config change records that preserve fuzzy painted region id, parent reference, destination region id, current config, derived config, and whether the config changed.
- Add a helper that compares current destination config with derived config.
- Add a helper that maps config-change plus destination ref count to the shared `StagedExistingRegionUpdateAction` values: `Unchanged`, `UpdateInPlace`, or `RequiresReslice`.
- Add fuzzy-painted wrappers around the existing staged config diff, invalidation event, and config apply helpers.
- For unchanged and requires-reslice actions, diff/event/apply helpers must produce no update payload, matching the upstream branch that only updates inside the zero-ref changed case.
- For update-in-place actions, preserve diff key order from the current config values, preserve callback payload current/derived config keys, and preserve apply-only derived values for diff keys.
- Do not perform ref-count increment from `PrintApply.cpp:856`, real `PrintRegionConfig`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Fuzzy-painted update gate keeps unchanged zero-ref destination regions unchanged.
- Fuzzy-painted update gate keeps unchanged referenced destination regions unchanged.
- Fuzzy-painted update gate updates changed zero-ref destination regions in place.
- Fuzzy-painted update gate requires reslice for changed referenced destination regions.
- Fuzzy-painted config change records preserve fuzzy region id, parent reference, destination region id, current config, derived config, and changed flag.
- Fuzzy-painted config diff preserves current key order for update-in-place.
- Fuzzy-painted config apply skips unchanged actions.
- Fuzzy-painted config apply skips requires-reslice actions.
- Fuzzy-painted invalidation event preserves callback payload before apply.
- Fuzzy-painted config apply requires invalidation event and records apply-only state.

## Migration note

This milestone stages `PrintApply.cpp:843-853` only. Later milestones must continue with fuzzy-painted `print_region_ref_inc(*region.region)` at `PrintApply.cpp:856`, then the region merge verification block after `PrintApply.cpp:860`, as separate source-cited rewrite slices.
