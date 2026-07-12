# Spec: M296 PrintApply verify-update existing region config diff keys

## Goal

Port the config diff-key collection used by OrcaSlicer's existing-region update-in-place branch into `ares-core` as private staged state.

## Rewrite gate mapping

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:801`: `t_config_option_keys diff = region.region->config().diff(cfg);`

Required context:
- `OrcaSlicer/src/libslic3r/Config.cpp:518-528`: `ConfigBase::diff(...)` iterates `this->keys()`, compares options present in both configs, and appends keys whose values differ.
- `OrcaSlicer/src/libslic3r/Config.hpp:73-75`: `t_config_option_key` is `std::string`; `t_config_option_keys` is `std::vector<std::string>`.
- `OrcaSlicer/src/libslic3r/PrintApply.cpp:798-803`: diff keys are computed only inside the changed-region zero-ref update-in-place branch.
- M295 `StagedExistingRegionUpdateAction` identifies update-in-place versus unchanged/reslice actions.

## Requirements

- Extend only private `ares-core` PrintApply staged implementation; do not add public APIs or pipeline wiring.
- Add a private staged config-diff input type representing key/value pairs in current-key order. A simple `Vec<(String, u64)>` or equivalent staged value fingerprint is sufficient for this slice.
- Add a private staged diff result carrying `Vec<String>` in upstream order.
- Add a helper that accepts the M295 action, current staged config values, and derived staged config values.
- If the action is not `UpdateInPlace`, return no diff keys.
- If the action is `UpdateInPlace`, iterate current config keys in order and append a key only when both configs contain the key and their staged values differ.
- Preserve duplicate current-key behavior by evaluating each current entry in order; do not sort or deduplicate.
- Defer callback invalidation, config apply, real option value types, real `ConfigBase`, real `PrintRegionConfig`, hashing, ref-count increment, public APIs, UI, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

## Tests

- Update-in-place action returns changed keys in current config order.
- Keys absent from derived config are ignored.
- Keys absent from current config are ignored even when present in derived config.
- Equal values are suppressed.
- Duplicate current keys are preserved in current order without sorting or deduplication.
- Unchanged action returns no diff keys even if provided staged values differ.
- Requires-reslice action returns no diff keys even if provided staged values differ.

## Migration note

This milestone is a staged compatibility shell around `PrintApply.cpp:801` and `ConfigBase::diff(...)`. It does not invoke invalidation callbacks or mutate region config; later milestones must port `PrintApply.cpp:802-803` as source-cited slices.
