# M213 Spec: DynamicPrintConfig get_index_for_extruder complete-id lookup

## Goal

Port the complete integer ID-map branch of OrcaSlicer's `DynamicPrintConfig::get_index_for_extruder(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818`: `DynamicPrintConfig::get_index_for_extruder(...)`, limited to `id_opt` present and `int(id_opt->values.size()) >= v_size`, where a variant match compares `id_opt->get_at(index)` with `extruder_or_filament_id` before returning `index * stride`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:662`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:586-604`: `get_extruder_variant_string` concatenates mapped extruder type, space, and mapped nozzle volume type.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:412-421`: `ExtruderType` and `NozzleVolumeType` enum discriminants.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:565-575`: source string maps for `Direct Drive`, `Bowden`, `Standard`, and `High Flow`.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: `ConfigOptionVector<T>::get_at(i)` returns `values[i]` or `values.front()` when `i` is out of range.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`: `printer_extruder_id` / `printer_extruder_variant`, `print_extruder_id` / `print_extruder_variant`, and `filament_self_index` / `filament_extruder_variant` option context for future callers.

## Deferred behavior

- The incomplete-ID-map branch using the `generated_extruder_id` lambda.
- `extruder_variant_list` generated ID lookup.
- `update_values_to_printer_extruders*`, preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/extruder_index.rs` with `SliceOptions::get_index_for_extruder_complete_id_map(&self, lookup: ExtruderIndexIdMapLookup<'_>) -> Result<isize, SliceError> plus exported `ExtruderIndexIdMapLookup` fields for `extruder_or_filament_id`, `id_name`, `extruder_type`, `nozzle_volume_type`, `variant_name`, and `stride``.
- Extend `crates/ares-core/src/options/tests/extruder_index.rs` with complete-ID-map tests.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public read-only API `SliceOptions::get_index_for_extruder_complete_id_map(ExtruderIndexIdMapLookup { ... }) -> Result<isize, SliceError>`.
2. If `variant_name` is absent, return `Ok(-1)` before validating enum strings or ID-map values, matching the source `variant_opt != nullptr` guard.
3. If `variant_name` is present, it must be a non-empty string array.
4. `id_name` must name a present non-empty integer vector whose members fit C++ `int` / Rust `i32` for this complete-ID-map API.
5. The ID vector length must be at least the variant vector length. Shorter ID vectors are deferred to the future generated-ID milestone and return `SliceError::InvalidInput` here.
6. Generate the target variant string as `{extruder_type} {nozzle_volume_type}` only after confirming the variant option exists.
7. Accept only source enum strings `Direct Drive` / `Bowden` and `Standard` / `High Flow` when lookup proceeds.
8. Unknown enum strings return `SliceError::InvalidInput` at the Ares boundary only when the variant option exists.
9. Iterate exactly `0..variant_values.len()` in source order.
10. For each index, compare the source `get_at(index)` variant string with the generated target string.
11. For matching variants, compare source `id_opt->get_at(index)` with `extruder_or_filament_id` using `i32` semantics.
12. Return the first matching `index * stride` as `isize`.
13. Allow `stride == 0`, matching the source `unsigned int stride` multiplication behavior.
14. Return `SliceError::InvalidInput` if public `usize` multiplication cannot fit the Rust `isize` return type.
15. Return `Ok(-1)` when no variant+ID pair matches.
16. Invalid public boundary values return `SliceError::InvalidInput`: present variant is not an array, present variant array is empty, variant member is not a string, missing/non-array/empty/incomplete ID vector, non-integer or non-`i32` ID member, enum strings are unknown, or `index * stride` overflows the Rust return type.
17. Do not add generated ID lookup, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove missing `variant_name` returns `-1`, including when enum strings and ID options are invalid or absent.
- Tests prove exact variant+ID match returns `index * stride` for `printer_extruder_id` + `printer_extruder_variant`, `print_extruder_id` + `print_extruder_variant`, and `filament_self_index` + `filament_extruder_variant`.
- Tests prove matching variant with non-matching ID continues searching and can match later entries.
- Tests prove duplicate variants return the first entry whose ID also matches.
- Tests prove no variant+ID match returns `-1`.
- Tests prove all four valid source enum string combinations generate expected target variant strings while checking IDs.
- Tests prove `stride == 0` returns `0` for a match.
- Tests prove overflowing `index * stride` returns `SliceError::InvalidInput`.
- Tests prove invalid variant boundary values return `SliceError::InvalidInput`.
- Tests prove missing, empty, incomplete, non-array, non-integer, and non-`i32` integer ID vectors return `SliceError::InvalidInput` when lookup proceeds.
- Tests prove unknown extruder or nozzle enum strings return `SliceError::InvalidInput` when lookup proceeds.
- Plan/spec explicitly account for deferred generated extruder IDs, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
