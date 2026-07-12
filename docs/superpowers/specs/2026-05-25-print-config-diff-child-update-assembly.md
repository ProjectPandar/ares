# DynamicPrintConfig diff child-config update assembly Spec

## Goal
Port the full staged `DynamicPrintConfig::update_diff_values_to_child_config(...)` body that combines the M254-M256 diff child-config helpers into one internal `ares-core` update pass.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9972-10048`: complete `DynamicPrintConfig::update_diff_values_to_child_config(...)` body.

Included upstream slices already staged in Rust:
- `PrintConfig.cpp:9972-10022`: diff child `variant_index` setup from M254.
- `PrintConfig.cpp:10024-10037`: direct-set key branch from M255.
- `PrintConfig.cpp:10038-10045`: vector branch stride selection and `set_only_diff` call from M256.
- `OrcaSlicer/src/libslic3r/Config.hpp:561-580`: vector `set_only_diff` mutation semantics.

Context only:
- `PrintConfig.hpp:667-668`: owning function declaration.
- `Config.hpp`: scalar/vector option shape context.

## Requirements
- Add private assembly helpers in `crates/ares-core/src/options/update_diff_values_to_child_config.rs`.
- Compute `variant_index` by calling the M254 helper with current/source options, target/child options, `extruder_id_name`, and `extruder_variant_name`.
- Iterate target/child keys in the order provided to the helper.
- Skip keys equal to `extruder_id_name` or `extruder_variant_name`.
- Skip keys missing from either current/source or target/child options.
- Skip keys whose current/source value equals the target/child value.
- For scalar target values, copy the target/child value into current/source.
- For array target values whose key is absent from both `key_set1` and `key_set2`, copy the target/child value into current/source.
- For array target values whose key is present in `key_set1` or `key_set2`, apply M256 `set_only_diff` semantics using the computed `variant_index` and stride selected from `key_set2` membership.
- Treat JSON `null` in target arrays as a staged nil marker for the existing `Option<Value>` representation; a null at the selected target stride's first slot skips the whole copied segment.
- Return the existing `SliceError::InvalidInput` error from `apply_diff_set_only_diff` when current/source vector length is invalid for the computed variant index and stride.
- Keep the helper private and disconnected from public slicing/profile APIs.
- Add focused tests for full assembly direct scalar copy, non-restore array copy, stride-1 restore vector mapping, stride-2 restore vector mapping, metadata/missing/equal skips, and null target skip behavior.

## Non-goals
- Do not implement public `SliceOptions` API wiring, concrete Orca `ConfigOption` class hierarchy, exhaustive JSON option type dispatch, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
