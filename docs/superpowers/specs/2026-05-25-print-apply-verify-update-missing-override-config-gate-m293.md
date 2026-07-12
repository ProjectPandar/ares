# Spec: M293 PrintApply verify-update missing override config gate

## Goal

Port the missing-modifier-override config comparison/reslice gate from OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:786-789`: after parent bbox intersection, derive a `PrintRegionConfig` for the current modifier over the parent region config; if the derived config differs from `parent_region.region->config()`, return `false` because the object needs reslicing.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:727`: `region_config_from_model_volume(...)` declaration.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3430-3460`: `region_config_from_model_volume(...)` implementation context; this milestone does not yet port the merge internals.
- M292 staged parent bbox gate provides the preceding `parent_bbox.intersects(*bbox)` condition.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a compact private staged config value representing `PrintRegionConfig` equality for this gate.
- Add a private staged result carrying `parent_region_id`, `parent_config`, `derived_config`, and `requires_reslice`.
- Add a helper that accepts the parent region id, parent config, and already-derived config, then returns `requires_reslice = derived_config != parent_config`.
- Equal configs must not require reslice.
- Different configs must require reslice.
- Parent id must be recorded but must not affect config equality.
- Defer actual `region_config_from_model_volume(...)` merging, `apply_to_print_region_config(...)`, extruder clamping, sparse infill/fuzzy-skin normalization, callbacks, ref-counts, painted/fuzzy regions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Equal parent/derived staged configs return `requires_reslice = false`.
- Differing staged configs return `requires_reslice = true`.
- Result preserves `parent_region_id`, `parent_config`, and `derived_config`.
- Distinct parent ids with identical config values do not affect the equality decision.

## Migration note

This milestone is a staged compatibility shell around the `PrintRegionConfig` comparison in `PrintApply.cpp:786-789`. It intentionally does not create an Ares-owned pipeline or full config merge system; later milestones must port `region_config_from_model_volume(...)` internals from `PrintObject.cpp:3430-3460`.
