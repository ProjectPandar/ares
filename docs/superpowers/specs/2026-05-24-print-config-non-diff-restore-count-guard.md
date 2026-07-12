# M244 Spec: DynamicPrintConfig non-diff restore count guard

## Goal

Port OrcaSlicer's child-greater-than-parent guard from the vector restore branch of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal predicate, without implementing stride selection or `set_with_restore`.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9910-9916`: restore-branch entry and `if (cur_variant_count > target_variant_count) continue;` guard.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9905-9909`: M243 no-op predicate context that decides whether this restore branch is reached.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9964`: deferred stride selection, expected-size calculation, normalization, and `set_with_restore` context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9844-9864`: current and target variant-count setup context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.

## Deferred behavior

- `PrintConfig.cpp:9918-9964`: stride selection, expected-size calculation, stride-2 float type check, normalization, vector resizing, temporary target cloning, and `set_with_restore` mutation.
- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)`.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M244 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal predicate for the restore count guard.
- Add focused M244 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/restore_count_guard.rs`.
- Register that test submodule from `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m244-print-config-non-diff-restore-count-guard.md`, create the matching implementation plan, and append one M244 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal predicate equivalent to Orca's `cur_variant_count > target_variant_count` guard for keys that already reached the vector restore branch.
2. The helper inputs must include the current variant count and target variant count.
3. Return `true` when `cur_variant_count > target_variant_count`, meaning the later restore merge must be skipped.
4. Return `false` when the counts are equal.
5. Return `false` when the current variant count is less than the target variant count.
6. Preserve zero-count semantics: `0 > 0` and `0 > n` return `false`, while `n > 0` returns `true`.
7. Do not inspect key sets, values, variant-index contents, or option definitions in this helper.
8. Do not mutate current or target configs.
9. Do not call this helper from M242/M243 helpers yet; full key-loop assembly remains deferred.
10. Do not implement stride selection, expected-size calculation, vector restore, `set_with_restore`, normalization call sites, logging, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Current count greater than target count returns skip/`true`.
- Equal counts return no-skip/`false`.
- Current count less than target count returns no-skip/`false`.
- Zero/zero and zero/current-less-than-target cases return no-skip/`false`.
- Nonzero current with zero target returns skip/`true`.
