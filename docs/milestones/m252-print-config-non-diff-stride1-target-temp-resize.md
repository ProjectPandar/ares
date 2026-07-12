# M252: DynamicPrintConfig non-diff stride-1 target temporary resize

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the non-stride-2 restore branch target clone, vector check, and target temporary resize inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9955-9961`, with preceding source resize context from `PrintConfig.cpp:9952-9953`, deferred restore context from `PrintConfig.cpp:9963`, declaration context from `PrintConfig.hpp:666-668`, `ConfigOptionVectorBase::resize(...)` declaration/comment context from `Config.hpp:341-362`, and concrete `ConfigOptionVector<T>::resize(...)` behavior from `Config.hpp:632-664`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that models the cloned target temporary normalization from `ConfigOptionUniquePtr rhs_owner(opt_target->clone())` through `if (rhs_vec->size() != expected_size) rhs_vec->resize(expected_size, opt_target);`.
- Preserve upstream resize behavior on the target temporary: matching sizes remain unchanged, zero expected size clears the temporary, oversized target values are truncated, and undersized non-empty target temporaries extend by duplicating the first target value.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement vector `set_with_restore`, `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
