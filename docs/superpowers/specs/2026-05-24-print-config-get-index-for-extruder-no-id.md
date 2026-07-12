# M212 Spec: DynamicPrintConfig get_index_for_extruder no-id lookup

## Goal

Port the no-id-map branch of OrcaSlicer's `DynamicPrintConfig::get_index_for_extruder(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818`: `DynamicPrintConfig::get_index_for_extruder(...)`, limited to the branch where `id_name.empty()` yields `id_opt == nullptr` and a variant match returns `index * stride` directly.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:662`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:586-604`: `get_extruder_variant_string` concatenates mapped extruder type, space, and mapped nozzle volume type.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:412-421`: `ExtruderType` and `NozzleVolumeType` enum discriminants.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:565-575`: source string maps for `Direct Drive`, `Bowden`, `Standard`, and `High Flow`.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: `ConfigOptionVector<T>::get_at(i)` returns `values[i]` or `values.front()` when `i` is out of range.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5252-5264` and `5292-5298`: printer and filament variant option context for future callers.

## Deferred behavior

- The `id_opt` branch of `get_index_for_extruder`.
- The `generated_extruder_id` lambda and `extruder_variant_list` generated ID lookup.
- `extruder_or_filament_id` matching.
- `update_values_to_printer_extruders*`, preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Create `crates/ares-core/src/options/extruder_index.rs` with `SliceOptions::get_index_for_extruder_no_id(&self, extruder_type: &str, nozzle_volume_type: &str, variant_name: &str, stride: usize) -> Result<isize, SliceError>`.
- Modify `crates/ares-core/src/options.rs` to register `mod extruder_index;`.
- Create `crates/ares-core/src/options/tests/extruder_index.rs`.
- Modify `crates/ares-core/src/options/tests.rs` to register `mod extruder_index;`.

## Functional requirements

1. Add public read-only API `SliceOptions::get_index_for_extruder_no_id(...) -> Result<isize, SliceError>`.
2. If `variant_name` is absent, return `Ok(-1)` before validating enum strings, matching the source `variant_opt != nullptr` guard.
3. If `variant_name` is present, it must be a non-empty string array.
4. Generate the target variant string as `{extruder_type} {nozzle_volume_type}` only after confirming the variant option exists.
5. Accept only source enum strings `Direct Drive` / `Bowden` and `Standard` / `High Flow` when lookup proceeds.
6. Unknown enum strings return `SliceError::InvalidInput` at the Ares boundary only when the variant option exists.
7. Iterate exactly `0..variant_values.len()` in source order.
8. For each index, compare the source `get_at(index)` string with the generated target string. Because the loop is bounded by the vector length, fallback is not normally observable, but the helper should preserve the source first-value fallback invariant.
9. Return the first matching `index * stride` as `isize`.
10. Allow `stride == 0`, matching the source `unsigned int stride` multiplication behavior.
11. Return `SliceError::InvalidInput` if public `usize` multiplication cannot fit the Rust `isize` return type.
12. Return `Ok(-1)` when no variant matches.
13. Invalid public boundary values return `SliceError::InvalidInput`: present variant is not an array, present variant array is empty, variant member is not a string, enum strings are unknown, or `index * stride` overflows the Rust return type.
13. Do not add the ID-map branch, generated ID lookup, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove missing `variant_name` returns `-1`, including when enum strings are unknown.
- Tests prove exact first match returns `index * stride`.
- Tests prove first match wins when duplicate variants exist.
- Tests prove no match returns `-1`.
- Tests prove all four valid source enum string combinations generate the expected target variant string.
- Tests prove `stride == 0` returns `0` for a match.
- Tests prove overflowing `index * stride` returns `SliceError::InvalidInput`.
- Tests prove invalid variant boundary values return `SliceError::InvalidInput`.
- Tests prove unknown extruder or nozzle enum strings return `SliceError::InvalidInput`.
- Plan/spec explicitly account for deferred ID-map lookup, generated extruder IDs, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
