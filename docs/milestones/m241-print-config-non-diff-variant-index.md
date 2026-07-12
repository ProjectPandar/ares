# M241: DynamicPrintConfig non-diff base-config variant index setup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the setup and `variant_index` calculation prefix of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9844-9894`, with declaration context from `PrintConfig.hpp:666-668`, the later `set_with_restore` consumer context from `PrintConfig.cpp:9918-9963`, string/int vector storage context from `Config.hpp`, and the M240 stride-2 helper context from `PrintConfig.cpp:9789-9830`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add a platform-neutral `ares-core` helper that computes Orca-compatible non-diff base-config `variant_index` values from current and target `SliceOptions`.
- Preserve upstream behavior for optional extruder-id loading, variant-list loading, target-sized `-1` initialization, missing current variant fallback to first source variant, current id/vector length mismatch, target id/vector length mismatch, and nested variant/id matching.
- Keep this helper internal to `ares-core` options update code until later milestones consume it.
- Add focused tests for every branch in `PrintConfig.cpp:9844-9894` that can be represented safely in Rust.
- Do not implement the key loop, equality comparison, `different_keys` handling, `set_with_restore`, logging helper, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
