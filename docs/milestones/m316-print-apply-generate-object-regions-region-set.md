# M316: PrintApply generate_print_object_regions region-set helper

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintApply.cpp:998-1010`: after `update_volume_bboxes(...)`, `generate_print_object_regions(...)` creates a local `region_set` sorted by `PrintRegion::config_hash()` and `PrintRegion::config()`, then defines `get_create_region(PrintRegionConfig &&config)` to reuse an equal existing `PrintRegion` or append a new `PrintRegion` to `all_regions` with id `int(all_regions.size())` and insert its pointer into the sorted `region_set`. Required context comes from M314-M315's staged print-object-region shell and call-boundary state. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region or slicing pipeline.

## Exit criteria

- Add private staged behavior for `PrintApply.cpp:998-1010`.
- Preserve lookup ordering by `(config_hash, config ordering)` using a staged config key.
- Preserve reuse when both hash and config are equal.
- Preserve new-region creation id as the previous `all_regions` length.
- Preserve append order in `all_regions` while keeping the local staged `region_set` sorted for lookup.
- Add tests for empty insertion, equal-config reuse, hash collision with distinct configs, sorted insertion independent of creation order, and all-region id/order preservation.
- Defer volume-region construction from `PrintApply.cpp:1012-1054`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `PrintRegionConfig`, real `PrintRegion`, real config diffing, public API wiring, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
