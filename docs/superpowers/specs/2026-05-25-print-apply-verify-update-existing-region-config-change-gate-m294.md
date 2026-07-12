# Spec: M294 PrintApply verify-update existing region config change gate

## Goal

Port the existing volume-region config change predicate from OrcaSlicer's `verify_update_print_object_regions(...)` into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:796`: after deriving `cfg`, compare it against `region.region->config()` and enter the changed-region branch only when they differ.

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:793-795`: `cfg` is derived from default/layer config or parent region config depending on `region.parent`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:798-806`: later changed-region branch either updates in place or returns false based on ref count; deferred here.
- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3430-3460`: `region_config_from_model_volume(...)` implementation context; merge internals remain deferred.
- M293 staged `StagedPrintRegionConfigKey` provides equality-only config state.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse `StagedPrintRegionConfigKey` for equality-only config comparison.
- Add a private staged result carrying `volume_region_id`, `current_config`, `derived_config`, and `config_changed`.
- Add a helper that accepts the volume region id, current config, and already-derived config, then returns `config_changed = derived_config != current_config`.
- Equal configs must not be marked changed.
- Different configs must be marked changed.
- Region id must be recorded but must not affect config equality.
- Defer derived config source selection, real `region_config_from_model_volume(...)` merging, ref-count update/split behavior, callback invalidation, config diff/apply, painted/fuzzy regions, real `PrintRegion`, real `PrintObjectRegions`, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Equal current/derived staged configs return `config_changed = false`.
- Differing staged configs return `config_changed = true`.
- Result preserves `volume_region_id`, `current_config`, and `derived_config`.
- Distinct volume region ids with identical config values do not affect the equality decision.

## Migration note

This milestone is a staged compatibility shell around the existing-region config predicate in `PrintApply.cpp:796`. It does not create an Ares-owned pipeline or full config system; later milestones must port `PrintApply.cpp:793-795` derivation selection and `PrintApply.cpp:798-806` update/split behavior as source-cited slices.
