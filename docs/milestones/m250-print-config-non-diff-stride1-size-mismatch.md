# M250: DynamicPrintConfig non-diff stride-1 vector size mismatch detection

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the non-stride-2 restore branch vector access, source/target size capture, and mismatch predicate inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9943-9950`, with stride selection context from `PrintConfig.cpp:9918-9923`, M249 stride-2 sibling context from `PrintConfig.cpp:9930-9942`, deferred resize/clone/restore context from `PrintConfig.cpp:9952-9963`, declaration context from `PrintConfig.hpp:666-668`, `ConfigOptionVectorBase` context from `Config.hpp:341-360`, and `ConfigOptionVector<T>::set_with_restore(...)` semantics from `Config.hpp:488-504`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that returns stride-1/general vector source size, target size, and whether either size differs from `expected_size`.
- Preserve upstream predicate behavior: legacy-size logging is needed when `src_size != expected_size || dest_size != expected_size`.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement `log_normalize_legacy_vector_size`, vector resize behavior, target clone normalization, vector `set_with_restore`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
