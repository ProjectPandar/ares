# M240: DynamicPrintConfig normalize stride-2 float vectors

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the anonymous-namespace `normalize_stride2_floats(...)` helper in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9789-9830`, with downstream call context from `PrintConfig.cpp:9922-9942`, declaration context from `PrintConfig.hpp:666-668`, float vector storage context from `Config.hpp:812-870`, and machine-limit stride-2 key context from `PrintConfig.cpp:9925-9928`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add a platform-neutral `ares-core` helper for normalizing stride-2 float vectors to an expected length.
- Preserve upstream behavior for expected size zero, empty vectors, one-value vectors, odd vector lengths, truncation, and pair replication.
- Keep the helper internal to `ares-core` options update code until the later `update_non_diff_values_to_base_config` milestone consumes it.
- Add focused tests for every branch of the upstream helper.
- Do not implement `update_non_diff_values_to_base_config`, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
