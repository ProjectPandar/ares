# M294: PrintApply verify-update existing region config change gate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the existing volume-region config change predicate in `OrcaSlicer/src/libslic3r/PrintApply.cpp:796`, with derived config construction context from `PrintApply.cpp:793-795`, missing-override config comparison context from `PrintApply.cpp:786-789`, and `PrintRegionConfig` derivation context from `OrcaSlicer/src/libslic3r/PrintObject.cpp:3430-3460`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add a private staged helper equivalent to deciding whether an existing volume region's newly derived config differs from its current region config.
- Preserve that equal derived/current configs do not enter the update/split branch.
- Preserve that differing derived/current configs enter the update/split branch.
- Preserve carrying the volume region id and both config values for later ref-count update/split milestones.
- Add tests for equal configs, differing configs, region id preservation, and distinct region ids with identical config values.
- Defer `region_config_from_model_volume(...)` base/parent selection from `PrintApply.cpp:793-795`, real config merge internals, `print_region_ref_cnt(...)` update/split handling from `PrintApply.cpp:798-806`, `print_region_ref_inc(...)` from `PrintApply.cpp:809`, callback invalidation, config diff/apply, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
