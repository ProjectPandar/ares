# M214: DynamicPrintConfig get_index_for_extruder generated-ID lookup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the incomplete integer ID-map sub-branch of `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818` (`DynamicPrintConfig::get_index_for_extruder` when `id_opt` exists but `id_opt->values.size() < variant_opt->values.size()`), including the local `generated_extruder_id` lambda in the same function. Supporting context is `PrintConfig.hpp:662`, `PrintConfig.cpp:586-604`, `PrintConfig.hpp:412-421`, `PrintConfig.cpp:565-575`, `Config.hpp:624-630`, `PrintConfig.cpp:5239-5244` `extruder_variant_list`, and `PrintConfig.cpp:5252-5264`, `5272-5284`, `5292-5304` variant/ID option definitions. It adds only a read-only `SliceOptions::get_index_for_extruder_generated_id_map(ExtruderIndexIdMapLookup { ... })` helper for source incomplete-ID behavior. It does not port preset/profile materialization, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit criteria

- Missing `variant_name` option returns `-1` before enum, ID-map, or `extruder_variant_list` validation.
- Present variant option must be a non-empty string array.
- The `id_name` option must exist as a non-empty C++ `int`-range vector and be shorter than the variant vector length; complete maps remain owned by M213.
- Missing `extruder_variant_list` makes generated IDs return source `0` for every target index.
- Present `extruder_variant_list` must be a non-empty string array.
- Generated ID iteration follows source order over `extruder_variant_list.values.size()`; each entry uses source `get_at(extruder_index)`.
- Generated ID splitting uses comma `boost::split(..., token_compress_on)`, then `boost::trim`, then skips empty trimmed tokens before counting variant indices.
- A matching variant only returns when generated ID equals `extruder_or_filament_id`.
- The first variant+generated-ID match returns `index * stride` as `isize`, including source-compatible `stride == 0`; Rust return-type overflow returns `SliceError::InvalidInput`.
- No match returns `-1`.
- No preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
