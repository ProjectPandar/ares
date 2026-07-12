# M243 Spec: DynamicPrintConfig non-diff different-key no-op classification

## Goal

Port OrcaSlicer's `different_keys` no-op condition from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal classification helper, without implementing the later vector `set_with_restore` branches or designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9905-9909`: `different_keys` branch and no-op condition using `opt_target->is_scalar()`, `key_set1`, and `key_set2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9896-9904`: M242 key-loop and direct-inheritance context, including the `opt_src && opt_target` presence gate.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9910-9964`: deferred vector restore context.
- `OrcaSlicer/src/libslic3r/Config.hpp`: scalar/vector option kind context for representing `ConfigOption::is_scalar()`.

## Deferred behavior

- `PrintConfig.cpp:9910-9964`: child-greater-than-parent guard, stride selection, expected-size calculation, stride-2 float type check, normalization, vector resizing, and `set_with_restore` mutation.
- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)`.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M243 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for the no-op classification.
- Split `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs` into private test submodules before adding M243 tests, because it is already close to the 400 LOC limit.
- Add focused M243 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/different_key_noop.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m243-print-config-non-diff-different-key-noop.md`, create the matching implementation plan, and append one M243 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to the Orca no-op condition for a changed key already known to be in `different_keys`.
2. The helper inputs must include the key, target/base `SliceOptions`, `key_set1`, and `key_set2`, so missing `opt_target` can be represented.
3. Return `true` when the target value is missing, matching the upstream outer presence gate by keeping the current value for this slice.
4. Return `true` when the target option is scalar, matching `opt_target->is_scalar()`.
5. Return `true` when the key is absent from `key_set1` and `key_set2` is empty.
6. Return `true` when the key is absent from `key_set1` and `key_set2` is non-empty but does not contain the key.
7. Return `false` when the target option is vector-like and the key is present in `key_set1`.
8. Return `false` when the target option is vector-like, absent from `key_set1`, and present in `key_set2`.
9. Represent scalar/vector classification using the current Ares JSON value shape: JSON arrays are vector-like, all other JSON values are scalar for this helper.
10. Unknown target keys with scalar JSON values must be classified as scalar/no-op; unknown target keys with array JSON values must be classified as vector-like and follow key-set membership.
11. Do not mutate current or target configs.
12. Do not call this helper from the M242 direct-inheritance helper yet; full key-loop assembly remains deferred.
13. Do not implement vector restore, `set_with_restore`, stride selection, normalization call sites, logging, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Missing target values return no-op.
- Scalar target values return no-op even when the key appears in `key_set1` or `key_set2`.
- Vector target values absent from both key sets return no-op when `key_set2` is empty.
- Vector target values absent from both key sets return no-op when `key_set2` is non-empty and lacks the key.
- Vector target values present in `key_set1` return restore-needed (`false`).
- Vector target values present in `key_set2` return restore-needed (`false`) even when absent from `key_set1`.
- Unknown scalar JSON keys are treated as scalar no-op without registry lookup.
- Unknown array JSON keys are treated as vector-like and follow key-set membership.
- The predicate does not mutate the target config.
