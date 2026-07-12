# M212: DynamicPrintConfig get_index_for_extruder no-id lookup

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is the no-id-map branch of `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818` (`DynamicPrintConfig::get_index_for_extruder` when `id_name.empty()` makes `id_opt == nullptr`), with `PrintConfig.hpp:662` declaration context, `PrintConfig.cpp:586-604` `get_extruder_variant_string`, `PrintConfig.hpp:412-421` `ExtruderType` / `NozzleVolumeType` discriminants, `PrintConfig.cpp:565-575` enum string maps, `Config.hpp:624-630` vector `get_at` fallback semantics, and `PrintConfig.cpp:5252-5264` / `5292-5298` variant option context. It adds only a read-only `SliceOptions::get_index_for_extruder_no_id(...)` helper for the source branch where no integer ID map is supplied. It does not port the `id_opt` branch, generated extruder IDs, `extruder_variant_list` ID derivation, preset/profile materialization, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit criteria

- Missing `variant_name` option returns `-1` before enum validation, matching the source `variant_opt != nullptr` guard.
- Present variant option must be a non-empty string array; invalid public boundary values return `SliceError::InvalidInput` instead of panicking.
- Valid `extruder_type` values are source strings `Direct Drive` and `Bowden`; valid `nozzle_volume_type` values are `Standard` and `High Flow`.
- Invalid enum strings return `SliceError::InvalidInput` at the Ares public boundary when lookup proceeds instead of porting Orca's empty-string logging fallback.
- The target variant string is built as `{extruder_type} {nozzle_volume_type}` using the upstream enum maps.
- The lookup iterates `variant_opt->values.size()` in source order and compares each `variant_opt->get_at(index)` with the generated target string.
- The first matching variant returns `index * stride` as `isize`, including source-compatible `stride == 0` behavior; Rust return-type overflow returns `SliceError::InvalidInput`.
- No match returns `-1`.
- No `id_opt` branch, generated extruder ID, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
