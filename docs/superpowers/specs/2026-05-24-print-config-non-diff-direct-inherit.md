# M242 Spec: DynamicPrintConfig non-diff base-config direct inheritance

## Goal

Port OrcaSlicer's key-loop entry and non-`different_keys` direct inheritance branch of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal mutation helper, without implementing the later `different_keys` vector restore branches or designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9896-9904`: key iteration, `opt_src` / `opt_target` presence check, inequality check, and direct `opt_src->set(opt_target)` branch when `opt` is not in `different_keys`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9844-9894`: M241 setup and `variant_index` context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9905-9964`: deferred `different_keys` scalar/vector restore context.

## Deferred behavior

- `PrintConfig.cpp:9905-9964`: scalar no-op branch for `different_keys`, key-set membership checks, child-greater-than-parent guard, stride selection, normalization, and `set_with_restore` mutation.
- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)`.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for the direct-inheritance branch.
- Move tests from the inline `#[cfg(test)] mod tests` to `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs` if required to keep files at or below 400 LOC.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m242-print-config-non-diff-direct-inherit.md`, create the matching implementation plan, and append M242 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to the direct-inheritance part of `update_non_diff_values_to_base_config` for a current config, target/base config, ordered key list, and `different_keys` set.
2. Iterate keys in the caller-provided order; do not sort or deduplicate.
3. For each key, read the source value from current config and the target value from target config.
4. If either source or target is missing, leave current config unchanged for that key.
5. If source and target values are equal, leave current config unchanged for that key.
6. If the key is present in `different_keys`, leave current config unchanged for this milestone.
7. If source and target exist, differ, and the key is absent from `different_keys`, clone the target value into current config.
8. Repeated keys are allowed and should be processed in order; repeated direct-inherit keys remain idempotent because the first copy makes later iterations equal.
9. Preserve unknown JSON values already present in `SliceOptions`; this helper should not require option registry lookup for the direct branch.
10. Do not call M241 variant-index setup yet; this M242 slice ports only the key-loop direct branch and leaves assembly of the full function to later milestones.
11. Do not implement scalar `different_keys` no-op handling beyond skipping all `different_keys`, vector restore, `set_with_restore`, logging, normalization call sites, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Non-different keys with existing unequal values copy target values into current config.
- Missing source keys, missing target keys, and equal values leave current config unchanged.
- Keys listed in `different_keys` are skipped while other keys in the same call copy.
- Caller key order is honored and repeated keys are idempotent.
- Unknown-but-present JSON keys copy without registry lookup.
- The helper does not remove unrelated existing keys.
