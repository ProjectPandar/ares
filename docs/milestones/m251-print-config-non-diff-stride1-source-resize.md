# M251: DynamicPrintConfig non-diff stride-1 source vector resize

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the non-stride-2 restore branch source resize inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9952-9953`, with preceding size-mismatch context from `PrintConfig.cpp:9943-9950`, deferred target clone/resize and restore context from `PrintConfig.cpp:9955-9963`, declaration context from `PrintConfig.hpp:666-668`, `ConfigOptionVectorBase::resize(...)` declaration/comment context from `Config.hpp:341-362`, and concrete `ConfigOptionVector<T>::resize(...)` behavior from `Config.hpp:632-664`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that applies the `if (opt_vec_src->size() != expected_size) opt_vec_src->resize(expected_size, opt_target);` source-vector normalization for the stride-1/general restore branch.
- Preserve upstream resize behavior for this source slice: matching sizes remain unchanged, zero expected size clears the source, oversized source values are truncated, non-empty undersized sources extend by duplicating the first source value, and empty sources extend from the first target/default value.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement target clone/resize normalization, vector `set_with_restore`, `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
