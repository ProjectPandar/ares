# Spec: M298 PrintApply verify-update existing region config apply-only

## Goal

Port the config-application call used by OrcaSlicer's existing-region update-in-place branch into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:803`: `region.region->config_apply_only(cfg, diff, false);`

Required context:
- `OrcaSlicer/src/libslic3r/Print.hpp:136-139`: `PrintRegion::config_apply_only(...)` calls `m_config.apply_only(...)` and refreshes `m_config_hash`.
- `OrcaSlicer/src/libslic3r/Config.cpp:461-500`: `ConfigBase::apply_only(...)` loops through keys and sets destination options from source options when source contains the key; missing source keys are not initialized.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:801`: M296 staged diff keys.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:802`: M297 staged invalidate callback event happens before apply.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Reuse M296 `StagedConfigValue` / `StagedExistingRegionConfigDiff` and M297 `StagedExistingRegionInvalidateEvent`.
- Add a private staged apply result carrying the updated staged config values, `ignore_nonexistent`, and `hash_refreshed`.
- Add a helper that accepts an optional invalidate event, current staged config values, derived staged config values, and staged diff keys.
- If the invalidate event is absent, return no apply result.
- If the invalidate event is present, iterate diff keys in order and update matching current config entries from matching derived config values.
- If a diff key is absent from derived config, leave current config unchanged for that key.
- Preserve duplicate diff-key behavior by processing each key in order.
- Record `ignore_nonexistent = false` for this source line.
- Record `hash_refreshed = true` for the `PrintRegion::config_apply_only(...)` hash-refresh context.
- Defer vector `#` option handling, unknown-option exceptions, real option value typing, real hashing, real region mutation, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Present invalidate event applies changed derived values into current config.
- Duplicate diff keys are processed in order and converge to the derived value without changing key order.
- Diff keys missing from derived config leave current config unchanged.
- Missing invalidate event returns no apply result.
- Apply result records `ignore_nonexistent = false`.
- Apply result records `hash_refreshed = true`.

## Migration note

This milestone is a staged compatibility shell around `PrintApply.cpp:803`. It does not perform real `ConfigBase` mutation or hash calculation; later milestones must port `PrintApply.cpp:809` ref-count increment and broader real-region integration as source-cited slices.
