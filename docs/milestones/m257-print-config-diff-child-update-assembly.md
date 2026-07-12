# M257: DynamicPrintConfig diff child-config update assembly

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the full staged body of `DynamicPrintConfig::update_diff_values_to_child_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9972-10048`, with declaration context from `PrintConfig.hpp:667-668`, already-ported M254 variant-index setup from `PrintConfig.cpp:9972-10022`, M255 direct-set branch from `PrintConfig.cpp:10024-10037`, M256 vector `set_only_diff` branch from `PrintConfig.cpp:10038-10045`, and vector mutation semantics from `Config.hpp:561-580`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that assembles the previously ported M254-M256 helpers into a single diff child-config update pass over target/child keys.
- Preserve upstream behavior: compute the diff child `variant_index`, iterate child keys in order, skip extruder id/variant metadata keys, require both source/current and target/child values to exist, skip equal values, directly set scalar and non-restore vector keys, and apply `set_only_diff` semantics for vector keys in `key_set1` or `key_set2` using stride `1` or `2` respectively.
- Represent staged nullable target vector entries with the existing `Option<Value>` helper shape so JSON `null` can exercise upstream nil-skip behavior without adding concrete nullable option classes in this milestone.
- Keep changed Rust files at or below 400 LOC by placing new full-assembly tests in a nested test module if needed.
- Do not implement public API wiring, concrete ConfigOption type dispatch, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
