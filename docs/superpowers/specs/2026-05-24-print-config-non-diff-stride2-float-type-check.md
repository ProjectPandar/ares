# M246 Spec: DynamicPrintConfig non-diff stride-2 float type check

## Goal

Port OrcaSlicer's stride-2 restore branch float-vector type check from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper, without implementing normalization or restore mutation.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`: the `if (stride == 2)` branch and `coFloats` type check for both `opt_src` and `opt_target`, throwing `ConfigurationError` when either side is not `ConfigOptionFloats`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9923`: M245 stride and expected-size context that determines when this branch is reached.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9930-9942`: deferred stride-2 float restore branch context that consumes validated float vectors.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: `ConfigOptionFloats` vector storage context.

## Deferred behavior

- `PrintConfig.cpp:9930-9942`: source cast, target temporary clone, source/target size capture, `log_normalize_legacy_vector_size(...)`, `normalize_stride2_floats(...)`, and `set_with_restore(...)` mutation.
- `PrintConfig.cpp:9943-9963`: stride-1 vector restore branch.
- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)`.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M246 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for stride-2 float-vector validation.
- Add focused M246 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/stride2_float_type_check.rs`.
- Register that test submodule from `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m246-print-config-non-diff-stride2-float-type-check.md`, create the matching implementation plan, and append one M246 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to Orca's stride-2 `ConfigOptionFloats` type check.
2. The helper inputs must include the option key, current/source JSON value, and target/base JSON value.
3. Accept source and target values only when both are JSON arrays containing numeric float-compatible entries.
4. Accept empty arrays, matching `ConfigOptionFloats` empty-vector shape.
5. Reject a non-array source value.
6. Reject a non-array target value.
7. Reject arrays containing non-numeric entries.
8. Return `SliceError::InvalidInput` with a message naming the key and `ConfigOptionFloats for stride=2` when validation fails.
9. Do not inspect `key_set1`, `key_set2`, `variant_index`, expected size, or option definitions in this helper.
10. Do not mutate current or target configs.
11. Do not call this helper from M242-M245 helpers yet; full restore branch assembly remains deferred.
12. Do not implement source/target cloning, size checks, logging, normalization call sites, `set_with_restore`, stride-1 behavior, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Numeric source and numeric target arrays are accepted.
- Empty numeric-vector arrays are accepted.
- Non-array source values are rejected.
- Non-array target values are rejected.
- Source arrays containing non-numeric entries are rejected.
- Target arrays containing non-numeric entries are rejected.
- Rejection messages include the key and `ConfigOptionFloats for stride=2`.
