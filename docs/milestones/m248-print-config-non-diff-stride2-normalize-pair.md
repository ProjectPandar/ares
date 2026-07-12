# M248: DynamicPrintConfig non-diff stride-2 source and target normalization

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the stride-2 restore branch source float access, target temporary float copy, and paired normalization calls inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9930-9941`, with M246 type-check context from `PrintConfig.cpp:9925-9928`, M247 size mismatch context from `PrintConfig.cpp:9933-9937`, M240 `normalize_stride2_floats(...)` context from `PrintConfig.cpp:9789-9830`, deferred restore mutation context from `PrintConfig.cpp:9942`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that normalizes a mutable stride-2 source float vector in place and returns a normalized target temporary vector copy.
- Preserve upstream behavior where source is normalized in place, target is normalized through a temporary clone, and both use the same `expected_size` and M240 `normalize_stride2_floats(...)` semantics.
- Keep the helper private to `ares-core` options update code until later milestones assemble the full restore branch.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement `set_with_restore`, `log_normalize_legacy_vector_size`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
