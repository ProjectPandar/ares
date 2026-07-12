# M241 Spec: DynamicPrintConfig non-diff base-config variant index setup

## Goal

Port OrcaSlicer's setup and `variant_index` calculation prefix of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper, without implementing the later key mutation loop or designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9844-9894`: setup and `variant_index` calculation prefix of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9963`: downstream `stride`, `restore_n`, `expected_size`, and `set_with_restore` use context for the computed `variant_index`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9789-9830`: M240 `normalize_stride2_floats(...)` helper context for later stride-2 consumers.
- `OrcaSlicer/src/libslic3r/Config.hpp`: `ConfigOptionInts` and `ConfigOptionStrings` vector storage context.

## Deferred behavior

- `PrintConfig.cpp:9896-9970`: key iteration, option equality checks, `different_keys` handling, scalar/vector branching, stride calculation, logging, normalization call sites, and `set_with_restore` mutation.
- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)`.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for variant-index setup.
- Keep the helper private to `ares-core` options code, with no public API export.
- Keep `crates/ares-core/src/options.rs` at or below 400 LOC and avoid new module registration unless required.
- Add focused unit tests in `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs`.
- Create this spec, create `docs/milestones/m241-print-config-non-diff-variant-index.md`, create the matching implementation plan, and append M241 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to Orca's `variant_index` setup for current `self` config and target/base `new_config`.
2. If `extruder_id_name` is empty, do not load current or target extruder-id vectors.
3. If `extruder_id_name` is non-empty and an id option exists, load it as an integer vector for the matching config; if missing, leave that side's id vector empty.
4. If the variant option exists in current or target config, load it as a string vector for that config; if missing, leave that side's variant vector empty.
5. Initialize the returned `variant_index` to `-1` with length equal to the target variant count.
6. If the current variant count is zero and the target variant count is nonzero, set only the first target entry to `0` and leave remaining entries as `-1`.
7. If current ids are present and current variant count differs from current id count, return the initialized all-`-1` vector.
8. If target ids are present and target variant count differs from target id count, return the initialized all-`-1` vector.
9. Otherwise, for each target variant in order, find the first current variant with matching variant string and a matching id when target ids are present.
10. When target ids are absent, match only by variant string, preserving Orca's `target_extruder_ids.empty()` branch.
11. Preserve unmatched target entries as `-1`.
12. Return signed indices (`isize` or equivalent) so later milestones can preserve Orca's `-1` sentinel for `set_with_restore`.
13. Reject malformed JSON values for present id or variant vectors with `SliceError::InvalidInput` and do not mutate either config.
14. Do not implement the key loop, `different_keys`, `set_with_restore`, logging, normalization call sites, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Matching target variants with matching ids produce source indices in target order.
- Empty `extruder_id_name` ignores id vectors and matches by variant string only.
- Missing current variants with non-empty target variants returns `[0, -1, ...]`.
- Current id length mismatch returns an all-`-1` vector sized to the target variant list.
- Target id length mismatch returns an all-`-1` vector sized to the target variant list.
- Unmatched target variants remain `-1` while matched variants keep their source index.
- Missing target variants returns an empty vector.
- Malformed present id or variant vectors return `SliceError::InvalidInput`.
