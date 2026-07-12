# M254 Spec: DynamicPrintConfig diff child-config variant index setup

## Goal

Port OrcaSlicer's `variant_index` setup prefix from `DynamicPrintConfig::update_diff_values_to_child_config(...)` into `ares-core` as a small internal helper, without implementing the key loop or mutation behavior.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9972-10022`: function entry, optional id/variant vector loading, current and target variant counts, initial `variant_index` sizing, missing target behavior, id-length mismatch behavior, and nested variant/id matching.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:667-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10024-10103`: deferred key-loop and mutation context.
- `OrcaSlicer/src/libslic3r/Config.hpp`: `ConfigOptionInts` and `ConfigOptionStrings` vector storage context.

## Deferred behavior

- `PrintConfig.cpp:10024-10103`: key iteration, scalar direct set, vector diff-only behavior, nil inheritance behavior, and type-specific branches.
- Full assembly of `update_diff_values_to_child_config(...)`.
- Any changes to `update_non_diff_values_to_base_config(...)`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Create `crates/ares-core/src/options/update_diff_values_to_child_config.rs` with one internal helper for diff child-config variant-index setup plus private JSON vector readers.
- Register the module from `crates/ares-core/src/options.rs`.
- Add focused M254 tests inside `crates/ares-core/src/options/update_diff_values_to_child_config.rs` under `#[cfg(test)]`.
- Create this spec, create `docs/milestones/m254-print-config-diff-child-variant-index.md`, create the matching implementation plan, and append one M254 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to Orca's diff child-config variant-index setup prefix.
2. The helper inputs must include current/base `SliceOptions`, target/child `SliceOptions`, `extruder_id_name`, and `extruder_variant_name`.
3. If `extruder_id_name` is empty, ignore current and target id vectors.
4. If `extruder_id_name` is non-empty, load current and target integer id vectors only when present; missing vectors are empty.
5. Load current and target string variant vectors only when present; missing vectors are empty.
6. Initialize `variant_index` to `vec![-1; current_variant_count]` when current variants are present.
7. Initialize `variant_index` to `vec![0]` when current variants are absent.
8. If target variants are absent, set `variant_index[0] = 0` and return the vector.
9. If current ids are present and current variant count does not equal current id count, return the initialized vector without matching.
10. If target ids are present and target variant count does not equal target id count, return the initialized vector without matching.
11. Otherwise, for each current variant, set that current index to the first target index with the same variant string and, when current ids are present, the same id.
12. Preserve unmatched current variants as `-1`.
13. Reject malformed present id vectors as `SliceError::InvalidInput`.
14. Reject malformed present variant vectors as `SliceError::InvalidInput`.
15. Do not inspect key sets, option definitions, scalar/vector option values, nil values, or logging state in this helper.
16. Do not mutate current/base or target/child configs.
17. Do not implement the key loop, direct set, `set_only_diff`, `set_with_nil`, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Matching current variants and ids map current/base indexes to target/child indexes in current order.
- Empty id name matches by variant only.
- Missing current variants returns `[0]`.
- Missing target variants sets the first current entry to `0` and leaves later current entries at `-1`.
- Current id length mismatch returns the initialized vector without matching.
- Target id length mismatch returns the initialized vector without matching.
- Unmatched current variants remain `-1`.
- Malformed present id or variant vectors return `SliceError::InvalidInput`.
