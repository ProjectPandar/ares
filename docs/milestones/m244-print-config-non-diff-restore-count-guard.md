# M244: DynamicPrintConfig non-diff restore count guard

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the child-greater-than-parent restore guard inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9910-9916`, with no-op predicate context from `PrintConfig.cpp:9905-9909`, later stride/restore context from `PrintConfig.cpp:9918-9964`, declaration context from `PrintConfig.hpp:666-668`, and variant-count setup context from `PrintConfig.cpp:9844-9864`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` predicate that returns whether a `different_keys` vector restore branch must be skipped because the current/child config has more variants than the target/base config.
- Preserve upstream strict `cur_variant_count > target_variant_count` behavior, including equality and fewer-current cases entering later restore behavior.
- Keep the predicate private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping all Rust files at or below 400 LOC.
- Do not implement stride selection, expected-size calculation, vector `set_with_restore`, stride-1/stride-2 normalization, logging helper, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
