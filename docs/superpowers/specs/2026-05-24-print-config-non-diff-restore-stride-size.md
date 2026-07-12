# M245 Spec: DynamicPrintConfig non-diff restore stride and expected size

## Goal

Port OrcaSlicer's restore-branch stride selection and expected-size calculation from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper, without implementing the stride-specific vector restore branches.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9923`: `stride = 1`, `key_set2` membership sets stride 2, `restore_n = variant_index.size()`, and `expected_size = restore_n * stride`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9910-9916`: M244 child-greater-than-parent guard context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9942`: deferred stride-2 float branch context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9943-9963`: deferred stride-1 vector branch context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9844-9894`: variant-index setup context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.

## Deferred behavior

- `PrintConfig.cpp:9925-9942`: stride-2 float type check, legacy-size logging, M240 normalization calls, and `set_with_restore` mutation.
- `PrintConfig.cpp:9943-9963`: stride-1 vector resize/clone checks and `set_with_restore` mutation.
- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)`.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M245 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for restore stride and expected-size calculation.
- Add focused M245 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/restore_stride_size.rs`.
- Register that test submodule from `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m245-print-config-non-diff-restore-stride-size.md`, create the matching implementation plan, and append one M245 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to Orca's restore stride and expected-size calculation.
2. The helper inputs must include the option key, `key_set2`, and `restore_n`.
3. Return stride `2` when the key is present in `key_set2`.
4. Return stride `1` when the key is absent from `key_set2`.
5. Return stride `1` when `key_set2` is empty.
6. Return `expected_size = restore_n * stride`.
7. Preserve zero restore count semantics: expected size is zero for stride 1 and stride 2.
8. Preserve duplicate `key_set2` input semantics by treating membership as true if any entry equals the key.
9. Do not inspect key sets other than `key_set2`, option values, variant-index contents, or option definitions in this helper.
10. Do not mutate current or target configs.
11. Do not call this helper from M242-M244 helpers yet; full restore branch assembly remains deferred.
12. Do not implement stride-2 type checks, vector restore, `set_with_restore`, normalization call sites, logging, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Key absent from non-empty `key_set2` returns stride 1 and expected size `restore_n`.
- Empty `key_set2` returns stride 1 and expected size `restore_n`.
- Key present in `key_set2` returns stride 2 and expected size `restore_n * 2`.
- Duplicate key entries in `key_set2` still return stride 2.
- Zero restore count returns expected size zero for stride 1 and stride 2.
