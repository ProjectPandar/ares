# M242: DynamicPrintConfig non-diff base-config direct inheritance

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the key-loop entry and non-`different_keys` direct inheritance branch of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9896-9904`, with setup context from `PrintConfig.cpp:9844-9894`, declaration context from `PrintConfig.hpp:666-668`, and later `different_keys` handling context from `PrintConfig.cpp:9905-9964`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that applies the Orca direct-inheritance branch for keys that are not in `different_keys`.
- Preserve upstream behavior for ordered key iteration, source/target presence checks, equality skip, `different_keys` skip, target-value cloning, repeated keys, and no-op missing/equal/different-key cases.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full `update_non_diff_values_to_base_config` flow.
- Move existing inline tests for this module into a private test submodule if needed to keep Rust files at or below 400 LOC.
- Do not implement scalar `different_keys` no-op handling, vector `set_with_restore`, stride-1/stride-2 normalization, logging helper, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
