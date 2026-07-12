# M249: DynamicPrintConfig non-diff stride-2 set_with_restore mapping

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the stride-2 restore mutation call inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9942`, with `ConfigOptionVector<T>::set_with_restore(...)` semantics from `OrcaSlicer/src/libslic3r/Config.hpp:488-504`, M248 normalization context from `PrintConfig.cpp:9930-9941`, M246 type-check context from `PrintConfig.cpp:9925-9928`, M247 size mismatch context from `PrintConfig.cpp:9933-9937`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that applies stride-2 `set_with_restore` semantics to normalized source and target temporary float vectors.
- Preserve upstream behavior where source is first replaced with target temporary values, then each non-`-1` restore index restores the matching stride-2 pair from the backed-up original source values.
- Preserve invalid target size rejection when target temporary size is not `restore_index.len() * stride`, including upstream ordering where source has already been replaced by target temporary values before the error is returned.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement `log_normalize_legacy_vector_size`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
