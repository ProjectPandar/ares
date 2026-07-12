# M246: DynamicPrintConfig non-diff stride-2 float type check

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the stride-2 restore branch type check inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`, with stride and expected-size context from `PrintConfig.cpp:9918-9923`, later stride-2 normalization and restore context from `PrintConfig.cpp:9930-9942`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that validates the future stride-2 restore branch receives float-vector source and target values.
- Preserve upstream behavior where either non-float-vector side rejects the restore branch with an invalid configuration error equivalent to Orca's `ConfigurationError` message for `ConfigOptionFloats` stride-2 keys.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement source/target cloning, size logging, vector normalization calls, vector `set_with_restore`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
