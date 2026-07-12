# M255: DynamicPrintConfig diff child-config direct set branch

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the key iteration prefix, extruder id/variant key skip, source/target presence and inequality check, and direct `opt_src->set(opt_target)` branch inside `DynamicPrintConfig::update_diff_values_to_child_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10024-10037`, with M254 variant-index setup context from `PrintConfig.cpp:9972-10022`, deferred vector branch context from `PrintConfig.cpp:10038-10045`, declaration context from `PrintConfig.hpp:668`, and scalar/vector option context from `Config.hpp`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add an internal `ares-core` helper that applies the diff child-config direct-set branch over target/child keys.
- Preserve upstream behavior: skip `extruder_id_name` and `extruder_variant_name`, require both current/source and target/child values to exist, skip equal values, and copy target into current when target is scalar or the key is absent from both restore key sets.
- Preserve vector restore-needed classification for keys present in `key_set1` or `key_set2` by leaving current unchanged in this milestone.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement vector `set_only_diff`, stride selection, nil handling, full `update_diff_values_to_child_config`, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
