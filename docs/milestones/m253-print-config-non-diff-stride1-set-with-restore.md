# M253: DynamicPrintConfig non-diff stride-1 set_with_restore mapping

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the non-stride-2 restore mutation call inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9963`, with `ConfigOptionVector<T>::set_with_restore(...)` semantics from `OrcaSlicer/src/libslic3r/Config.hpp:488-504`, M250-M252 stride-1/general branch context from `PrintConfig.cpp:9943-9961`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionVectorBase` context from `Config.hpp:341-360`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that applies stride-1/general `set_with_restore` semantics to a source vector and normalized target temporary.
- Preserve upstream behavior where source is first replaced with target temporary values, then invalid target size is rejected when `target.len() != restore_index.len() * stride`, and each non-`-1` restore index restores one backed-up source element into the corresponding target position.
- Keep changed Rust files at or below 400 LOC; split the staged restore-vector helper module if necessary without changing existing behavior.
- Add focused tests while keeping the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Do not implement `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
