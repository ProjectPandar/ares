# Spec: M316 PrintApply generate_print_object_regions region-set helper

## Goal

Port OrcaSlicer's `generate_print_object_regions(...)` local `region_set` / `get_create_region(...)` helper into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:998-1010`: create `std::vector<PrintRegion*> region_set`; define `get_create_region` that computes `config.hash()`, lower-bounds by `config_hash()` and config ordering, reuses an equal existing region, otherwise appends `PrintRegion(std::move(config), hash, int(all_regions.size()))` to `all_regions`, inserts the new region pointer into `region_set`, and returns it.

Required context:
- M314 stages the `PrintObjectRegions` shell with `all_regions` and layer ranges.
- M315 stages the immediately preceding `update_volume_bboxes(...)` call boundary.
- M316 models `PrintRegionConfig` as a private comparable staged key with an explicit hash value; it does not port real config storage or comparison yet.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a lightweight staged region config key carrying a hash and comparable ordinal/value sufficient to model `config.hash()` and `config() < config` ordering.
- Add a lightweight staged print region record carrying id, config hash, and config key.
- Add a staged region set helper that owns the sorted lookup vector and can mutate a staged shell's `all_regions` through a `get_create_region`-style method/function.
- Reuse an existing region when both hash and config key are equal.
- Create a new region when either hash differs or hash collides but config key differs.
- Assign each new region id from the current `all_regions.len()` before append.
- Preserve `all_regions` append order separately from sorted lookup order.
- Keep all M316 symbols private to the staged `print_apply` module.
- Do not perform volume-region construction, painted/fuzzy construction, real `PrintRegionConfig`, real `PrintRegion`, config diffing, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, or Ares-owned pipeline behavior.

## Tests

- Empty helper insertion creates region id `0`, appends it to `all_regions`, and returns id `0`.
- Repeating an equal hash/config key reuses the existing id and does not append another region.
- A hash collision with a distinct config key creates a distinct region.
- Creation order that is not sorted by `(hash, config)` still leaves lookup order sorted while preserving `all_regions` append order.
- Starting from a shell with existing `all_regions` assigns the next new id from the existing length.

## Migration note

This milestone stages only `PrintApply.cpp:998-1010`. Later milestones must continue with volume-region construction at `PrintApply.cpp:1012-1054` and painted/fuzzy construction at `PrintApply.cpp:1056-1101` as separate source-cited rewrite slices.
