# M243: DynamicPrintConfig non-diff different-key no-op predicate

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the `different_keys` no-op predicate inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9905-9909`, with direct-inheritance context from `PrintConfig.cpp:9896-9904`, later restore-branch context from `PrintConfig.cpp:9910-9964`, declaration context from `PrintConfig.hpp:666-668`, and option type context from `Config.hpp`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` predicate that returns whether a `different_keys` entry should keep the current value instead of entering restore behavior.
- Preserve upstream behavior for scalar target options, vector keys absent from both restore key sets, an empty stride-2 key set, membership in either restore key set, missing target values, and unknown-present JSON values classified by shape.
- Keep the predicate private to `ares-core` options update code until later milestones assemble the full `different_keys` branch.
- Add focused tests while keeping all Rust files at or below 400 LOC.
- Do not implement vector `set_with_restore`, child-greater-than-parent guard, stride-1/stride-2 normalization, logging helper, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
