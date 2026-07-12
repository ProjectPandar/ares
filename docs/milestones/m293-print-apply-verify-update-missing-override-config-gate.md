# M293: PrintApply verify-update missing override config gate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the config-difference reslice gate for a missing modifier override in `OrcaSlicer/src/libslic3r/PrintApply.cpp:786-789`, with `region_config_from_model_volume(...)` declaration context from `PrintApply.cpp:727` and implementation context from `OrcaSlicer/src/libslic3r/PrintObject.cpp:3430-3460`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned region pipeline.

## Exit criteria

- Add private staged `PrintRegionConfig` equality state sufficient to represent the comparison `config != parent_region.region->config()`.
- Add a private staged helper equivalent to the reslice decision made when a missing override's derived config differs from its parent config.
- Preserve that equal derived/parent configs do not require reslice.
- Preserve that differing derived/parent configs require reslice for a newly needed missing override.
- Preserve carrying the parent region id and both config values for later wiring to the parent bbox gate and real config derivation.
- Add tests for equal configs, differing configs, parent id preservation, and distinct parent ids with identical config values.
- Defer `region_config_from_model_volume(...)` merge internals, `apply_to_print_region_config(...)`, extruder clamping, sparse infill/fuzzy-skin normalization, callback invalidation, ref-count increment, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and independent Ares pipeline behavior.
