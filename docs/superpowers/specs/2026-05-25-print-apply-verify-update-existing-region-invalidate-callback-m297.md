# Spec: M297 PrintApply verify-update existing region invalidate callback

## Goal

Port the callback-invalidation call used by OrcaSlicer's existing-region update-in-place branch into `ares-core` as private staged event state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:802`: `callback_invalidate(region.region->config(), cfg, diff);`

Required context:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:734`: comment states `callback_invalidate()` is called before region configs are updated to possibly stop background processing.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:741`: callback signature accepts old/current `PrintRegionConfig`, new/derived `PrintRegionConfig`, and `t_config_option_keys` diff keys.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:801`: M296 staged diff keys are computed immediately before the callback.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:803`: `config_apply_only(...)` happens after callback and remains deferred.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M293/M294 `StagedPrintRegionConfigKey`, M295 `StagedExistingRegionUpdateAction`, and M296 `StagedExistingRegionConfigDiff`.
- Add a private staged invalidation event carrying current config, derived config, and diff keys.
- Add a helper that accepts the update action, current config, derived config, and staged diff keys.
- If action is `UpdateInPlace`, return one event preserving current config first, derived config second, and the diff key order unchanged.
- If action is `Unchanged` or `RequiresReslice`, return no event.
- Do not call a real callback, stop background processing, mutate config, or expose public API.
- Defer config apply, real callback invocation, real configs/regions, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Update-in-place action emits one event with current config first and derived config second.
- Diff keys are preserved in order in the emitted event.
- Update-in-place action with an empty diff still emits an event.
- Unchanged action emits no event.
- Requires-reslice action emits no event.

## Migration note

This milestone is a staged compatibility shell around `PrintApply.cpp:802`. It records the callback event shape and ordering without invoking runtime callbacks or mutating config; later milestones must port `PrintApply.cpp:803` as a source-cited slice.
