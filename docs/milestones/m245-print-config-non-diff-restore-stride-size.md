# M245: DynamicPrintConfig non-diff restore stride and expected size

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the restore-branch stride selection and expected-size calculation inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9923`, with child-greater-than-parent guard context from `PrintConfig.cpp:9910-9916`, later stride-2 branch context from `PrintConfig.cpp:9925-9942`, later stride-1 branch context from `PrintConfig.cpp:9943-9963`, declaration context from `PrintConfig.hpp:666-668`, and `variant_index` setup context from `PrintConfig.cpp:9844-9894`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that computes the restore stride and expected vector size from a key, `key_set2`, and restore index count.
- Preserve upstream behavior where keys in `key_set2` use stride 2, all other keys use stride 1, and `expected_size = restore_n * stride`.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping all Rust files at or below 400 LOC.
- Do not implement stride-2 float type checks, vector normalization, vector resizing, temporary target cloning, `set_with_restore`, logging helper, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
