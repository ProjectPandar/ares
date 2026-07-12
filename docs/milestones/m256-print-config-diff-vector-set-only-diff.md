# M256: DynamicPrintConfig diff vector set_only_diff branch

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the vector branch stride selection and `opt_vec_src->set_only_diff(opt_vec_dest, variant_index, stride)` call inside `DynamicPrintConfig::update_diff_values_to_child_config(...)` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10038-10045`, with `ConfigOptionVector<T>::set_only_diff(...)` semantics from `OrcaSlicer/src/libslic3r/Config.hpp:561-580`, M254 variant-index setup context from `PrintConfig.cpp:9972-10022`, M255 direct-set branch context from `PrintConfig.cpp:10024-10037`, and declaration context from `PrintConfig.hpp:668`. It remains a source-cited `libslic3r` rewrite slice, not an Ares-owned pipeline feature.

## Exit criteria

- Add internal `ares-core` helpers that select stride `2` when the key is in `key_set2` and otherwise stride `1`, then apply `set_only_diff` semantics to a source vector and target vector.
- Preserve upstream `set_only_diff` behavior: reject when source length is not `diff_index.len() * stride`, leave `-1` diff indexes unchanged, copy each selected stride segment from the target index into the corresponding source index, and skip copying a whole stride segment when the target slot at `diff_index[i] * stride` is nil.
- Add focused tests while keeping changed Rust files at or below 400 LOC.
- Do not implement full `update_diff_values_to_child_config`, JSON option type dispatch, nil option classes beyond the helper's explicit nullable target representation, preset/profile loading, UI runtime behavior, slicing, extrusion, G-code, crate, dependency, or independent Ares pipeline behavior.
