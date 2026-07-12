# Spec: M314 PrintApply generate_print_object_regions layer-range shell

## Goal

Port OrcaSlicer's `generate_print_object_regions(...)` reuse/new layer-range shell into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:953-993`: object reuse/new shell, `all_regions.clear()`, old layer-range reuse detection, reused layer-range assertions/config refresh/region-list clearing, and fresh `trafo_bboxes` plus layer-range creation.

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:271-296`: `LayerRangeRegions` stores `layer_height_range`, `config`, `volumes`, `volume_regions`, `painted_regions`, `fuzzy_skin_painted_regions`, while `PrintObjectRegions` stores `all_regions`, `layer_ranges`, `trafo_bboxes`, and `cached_volume_ids`.
- `crates/ares-core/src/print_apply/volume_cache_state.rs` is near the 400 LOC split threshold, so this milestone must use a focused new private module instead of expanding that file.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a focused private `generate_regions_state.rs` module for this staged shell.
- Add lightweight staged records for model layer-range input, layer-range regions, and print-object-regions shell using ids for configs, transforms, all regions, cached ids, volumes, volume regions, painted regions, and fuzzy painted regions.
- Add a helper that consumes optional old state plus model layer ranges and input transform id, then returns the initialized staged state.
- If old state exists and has non-empty layer ranges, assert equal layer-range count and exact layer-height-range equality, refresh each range config from the model range, clear each range's volume/painted/fuzzy region lists, preserve each range's existing volumes, preserve cached volume ids, preserve old `trafo_bboxes`, and clear `all_regions`.
- If no old state exists or old state has empty layer ranges, set `trafo_bboxes` from the input transform id, create fresh layer ranges from model ranges, and clear `all_regions`.
- Do not perform `is_mm_painted`, `update_volume_bboxes`, region creation/deduplication, volume-region construction, painted/fuzzy construction, real configs/transforms, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Fresh creation stores the input transform id and model layer ranges.
- Old object with empty layer ranges follows the fresh path and uses the new transform id.
- Reuse clears `all_regions`, `volume_regions`, `painted_regions`, and `fuzzy_skin_painted_regions`.
- Reuse refreshes configs from model layer ranges and preserves existing volume extents/cached volume ids/old transform id.
- Reuse panics on layer-range count mismatch.
- Reuse panics on layer-height-range mismatch.

## Migration note

This milestone stages only the prefix shell of `generate_print_object_regions(...)`. Later milestones must continue with `is_mm_painted` / `update_volume_bboxes(...)` at `PrintApply.cpp:995-996`, then `get_create_region` and region construction as separate source-cited slices.
