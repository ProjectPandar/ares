# DynamicPrintConfig diff child-config direct set branch Spec

## Goal
Port OrcaSlicer's direct-set branch from `DynamicPrintConfig::update_diff_values_to_child_config(...)` into `ares-core` as a private helper for later full diff update assembly.

## Rewrite gate mapping
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream boundary:
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10024-10037`: iterate child keys, skip extruder id/variant keys, require source and target options, skip equal values, and directly set scalar or non-restore vector keys.

Context only:
- `PrintConfig.cpp:9972-10022`: already-ported M254 variant-index setup.
- `PrintConfig.cpp:10038-10045`: deferred vector `set_only_diff` branch.
- `PrintConfig.hpp:668`: owning function declaration.

## Requirements
- Add a private helper in `crates/ares-core/src/options/update_diff_values_to_child_config.rs`.
- The helper accepts mutable current/base `SliceOptions`, target/child `SliceOptions`, ordered child keys, `extruder_id_name`, `extruder_variant_name`, `key_set1`, and `key_set2`.
- Iterate the provided target/child keys in order.
- Skip keys equal to `extruder_id_name` or `extruder_variant_name`.
- Mutate only when both current and target contain the key and values differ.
- Copy target value into current when the target value is scalar/non-array.
- Copy target vector values into current when the key is absent from `key_set1` and either `key_set2` is empty or the key is absent from `key_set2`.
- Leave current unchanged for changed vector keys that are in `key_set1` or `key_set2`; those are deferred to `set_only_diff`.
- Add focused tests for scalar direct set, vector direct set outside key sets, extruder metadata skip, missing/equal value skip, and vector restore-needed no-op.

## Non-goals
- Do not implement vector `set_only_diff`, stride selection, nil handling, full diff update assembly, profile loading, UI, slicing, extrusion, G-code, new crates, new dependencies, or independent Ares pipeline behavior.
