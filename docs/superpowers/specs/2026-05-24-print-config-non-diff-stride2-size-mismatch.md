# M247 Spec: DynamicPrintConfig non-diff stride-2 size mismatch detection

## Goal

Port OrcaSlicer's stride-2 restore branch source/target size capture and legacy-size-log predicate from `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` into `ares-core` as a small internal helper, without implementing logging, normalization, or restore mutation.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9933-9937`: `src_size`, `dest_size`, and `src_size != expected_size || dest_size != expected_size` predicate before `log_normalize_legacy_vector_size(...)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`: M246 stride-2 float-vector type-check context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9939-9942`: deferred normalization and `set_with_restore(...)` context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9832-9841`: deferred logging helper context.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:666-668`: function declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: `ConfigOptionFloats` vector storage context.

## Deferred behavior

- `PrintConfig.cpp:9832-9841`: `log_normalize_legacy_vector_size(...)` implementation and call wiring.
- `PrintConfig.cpp:9939-9942`: `normalize_stride2_floats(...)` calls and `set_with_restore(...)` mutation.
- `PrintConfig.cpp:9943-9963`: stride-1 vector restore branch.
- `PrintConfig.cpp:9972+`: `update_diff_values_to_child_config(...)`.
- Full assembly of `update_non_diff_values_to_base_config(...)` that combines M241-M247 helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_non_diff_values_to_base_config.rs` with one internal helper for stride-2 size mismatch detection.
- Add focused M247 tests under `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests/stride2_size_mismatch.rs`.
- Register that test submodule from `crates/ares-core/src/options/update_non_diff_values_to_base_config/tests.rs`.
- Keep the helper private to `ares-core` options code, with no public API export.
- Create this spec, create `docs/milestones/m247-print-config-non-diff-stride2-size-mismatch.md`, create the matching implementation plan, and append one M247 entry to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Provide an internal helper equivalent to Orca's stride-2 source/target size capture and mismatch predicate.
2. The helper inputs must include source float values, target float values, and `expected_size`.
3. Return the source length and target length unchanged.
4. Return `false` for mismatch when both lengths equal `expected_size`.
5. Return `true` when only source length differs from `expected_size`.
6. Return `true` when only target length differs from `expected_size`.
7. Return `true` when both lengths differ from `expected_size`.
8. Preserve zero expected-size behavior: two empty vectors do not mismatch; any non-empty side mismatches.
9. Do not inspect key sets, option definitions, variants, or JSON values in this helper.
10. Do not mutate current or target configs.
11. Do not call this helper from M242-M246 helpers yet; full restore branch assembly remains deferred.
12. Do not implement logging, normalization call sites, `set_with_restore`, stride-1 behavior, public API, UI behavior, slicing behavior, G-code behavior, dependencies, crates, or independent pipeline behavior.

## Acceptance tests

- Equal source and target lengths matching expected size return no mismatch.
- Source-only mismatch returns mismatch.
- Target-only mismatch returns mismatch.
- Both-source-and-target mismatch returns mismatch.
- Zero expected size returns no mismatch for two empty vectors.
- Zero expected size returns mismatch when either side is non-empty.
