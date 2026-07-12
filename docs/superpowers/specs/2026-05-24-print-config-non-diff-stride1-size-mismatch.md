# M250 Spec: DynamicPrintConfig non-diff stride-1 vector size mismatch detection

## Goal

Port OrcaSlicer's non-stride-2 restore branch vector size capture and legacy-size-log predicate from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper, without implementing resize, target cloning, or restore mutation.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9943-9950`: non-stride-2 branch entry, `ConfigOptionVectorBase* opt_vec_src`, `src_size`, `dest_size`, and `src_size != expected_size || dest_size != expected_size` predicate before `log_normalize_legacy_vector_size(...)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9923`: M245 stride and expected-size calculation context proving this branch is the default stride-1 path when the key is not in `key_set2`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9930-9942`: M248-M249 stride-2 sibling branch context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9952-9963`: deferred vector resize, target clone normalization, and `set_with_restore(...)` context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:341-360`: `ConfigOptionVectorBase` size/restore interface context.
- `OrcaSlicer/src/libslic3r/Config.hpp:488-504`: later `ConfigOptionVector<T>::set_with_restore(...)` semantics context.

## Deferred behavior

- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)` implementation and call wiring.
- `PrintConfig.cpp:9952-9953`: source vector resize to `expected_size`.
- `PrintConfig.cpp:9955-9961`: target option clone and target vector resize to `expected_size`.
- `PrintConfig.cpp:9963`: stride-1/general vector `set_with_restore(...)` mutation.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M250 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for stride-1/general vector size mismatch detection.
- Add focused M250 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/stride1_size_mismatch.rs`.
- Register that test submodule from `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m250-print-config-non-diff-stride1-size-mismatch.md`, create the matching implementation plan, and append one M250 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to Orca's non-stride-2 branch source/target vector size capture and mismatch predicate.
2. The helper inputs must include source vector values, target vector values, and `expected_size`.
3. Return the source length and target length unchanged.
4. Return `false` for mismatch when both lengths equal `expected_size`.
5. Return `true` when only source length differs from `expected_size`.
6. Return `true` when only target length differs from `expected_size`.
7. Return `true` when both lengths differ from `expected_size`.
8. Preserve zero expected-size behavior: two empty vectors do not mismatch; any non-empty side mismatches.
9. Be generic over vector element types so the helper represents `ConfigOptionVectorBase` size behavior rather than float-only stride-2 behavior.
10. Do not inspect key sets, option definitions, variants, JSON values, or logging state in this helper.
11. Do not mutate current or target configs.
12. Do not call this helper from M242-M249 helpers yet; full restore branch assembly remains deferred.
13. Do not implement logging, resize, target clone normalization, `set_with_restore`, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Equal source and target lengths matching expected size return no mismatch.
- Source-only mismatch returns mismatch.
- Target-only mismatch returns mismatch.
- Both-source-and-target mismatch returns mismatch.
- Zero expected size returns no mismatch for two empty vectors.
- Zero expected size returns mismatch when either side is non-empty.
- Non-float vector element types are accepted without value inspection.
