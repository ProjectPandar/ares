# M247: DynamicPrintConfig non-diff stride-2 size mismatch detection

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the stride-2 restore branch source/target size capture and mismatch predicate inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9933-9937`, with float-vector type-check context from `PrintConfig.cpp:9925-9928`, deferred normalization and restore context from `PrintConfig.cpp:9939-9942`, logging helper context from `PrintConfig.cpp:9832-9841`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that returns stride-2 source size, target size, and whether either size differs from `expected_size`.
- Preserve upstream predicate behavior: log is needed when `src_size != expected_size || dest_size != expected_size`.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement `log_normalize_legacy_vector_size`, vector normalization calls, vector `set_with_restore`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
