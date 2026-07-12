# M249 Spec: DynamicPrintConfig non-diff stride-2 set_with_restore mapping

## Goal

Port OrcaSlicer's stride-2 `set_with_restore(...)` mutation from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper for normalized float vectors, without assembling the full non-diff restore branch.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9942`: `src_f->set_with_restore(&rhs_tmp, variant_index, stride)` inside the stride-2 branch.
- `OrcaSlicer/src/libslic3r/Config.hpp:488-504`: `ConfigOptionVector<T>::set_with_restore(...)` behavior: backup original values, replace with RHS values, require RHS size equals `restore_index.size() * stride`, then restore each non-`-1` indexed stride segment from backup.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9930-9941`: M248 normalized source and target temporary context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`: M246 stride-2 float-vector type-check context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9933-9937`: M247 size mismatch context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: `ConfigOptionFloats` vector storage context.

## Deferred behavior

- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)` implementation and call wiring.
- `PrintConfig.cpp:9943-9963`: stride-1 vector restore branch.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M249 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for normalized stride-2 float `set_with_restore` behavior.
- Add focused M249 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/stride2_set_with_restore.rs`.
- Register that test submodule from `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m249-print-config-non-diff-stride2-set-with-restore.md`, create the matching implementation plan, and append one M249 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to `ConfigOptionVector<T>::set_with_restore(...)` for normalized stride-2 float vectors.
2. The helper inputs must include mutable source float values, normalized target temporary float values, and `restore_index` values.
3. Use fixed stride `2` for this M249 helper because it ports the stride-2 branch only.
4. Backup the original source values before mutation.
5. Replace source values with target temporary values before applying restores.
6. After backing up source and replacing source with target temporary values, reject target temporary size when it is not `restore_index.len() * 2`, matching upstream mutation-before-throw ordering.
7. For each restore index equal to `-1`, leave the target temporary pair in place.
8. For each non-negative restore index, restore the two-value pair from the backed-up source at `restore_index * 2`.
9. Preserve target temporary ordering for un-restored pairs.
10. Preserve duplicate restore-index behavior by restoring the same source pair into multiple target positions.
11. Return `SliceError::InvalidInput` with a message containing `set_with_restore` and `invalid restore_index size` for invalid target size.
12. Do not inspect key sets, option definitions, variants, JSON values, or logging state in this helper.
13. Do not call this helper from M242-M248 helpers yet; full restore branch assembly remains deferred.
14. Do not implement logging, stride-1 behavior, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- All `-1` restore indexes replace source with target temporary unchanged.
- Non-negative restore indexes restore the corresponding source stride-2 pairs into target positions.
- Mixed `-1` and non-negative indexes preserve target pairs where restore is skipped.
- Duplicate restore indexes restore the same source pair into multiple target positions.
- Invalid target temporary size returns `SliceError::InvalidInput` with the expected message fragments after source has been replaced by the target temporary values, matching `Config.hpp:488-504` mutation ordering.
