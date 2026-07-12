# M214 Spec: DynamicPrintConfig get_index_for_extruder generated-ID lookup

## Goal

Port the incomplete-ID generated-extruder-ID branch of OrcaSlicer's `DynamicPrintConfig::get_index_for_extruder(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818`: `DynamicPrintConfig::get_index_for_extruder(...)`, limited to `id_opt` present and `int(id_opt->values.size()) < v_size`, where the local `generated_extruder_id(index)` lambda supplies the ID compared with `extruder_or_filament_id`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:662`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:586-604`: `get_extruder_variant_string`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:412-421`: `ExtruderType` and `NozzleVolumeType` enum discriminants.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:565-575`: source string maps for `Direct Drive`, `Bowden`, `Standard`, and `High Flow`.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` fallback semantics.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239-5244`: `extruder_variant_list` option context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`: representative printer, print, and filament variant/ID option context.

## Deferred behavior

- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/extruder_index.rs` with `SliceOptions::get_index_for_extruder_generated_id_map(&self, lookup: ExtruderIndexIdMapLookup<'_>) -> Result<isize, SliceError>`.
- Split `crates/ares-core/src/options/tests/extruder_index.rs` before adding tests so modified Rust files remain under 400 LOC.
- Add generated-ID tests under the split extruder-index test module.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public read-only API `SliceOptions::get_index_for_extruder_generated_id_map(lookup) -> Result<isize, SliceError>`.
2. If `variant_name` is absent, return `Ok(-1)` before validating enum strings, ID-map values, or `extruder_variant_list`.
3. If `variant_name` is present, it must be a non-empty string array.
4. `id_name` must name a present non-empty integer vector whose members fit C++ `int` / Rust `i32`.
5. The ID vector length must be shorter than the variant vector length. Complete ID vectors return `SliceError::InvalidInput` because M213 owns that source branch.
6. Generate the target variant string as `{extruder_type} {nozzle_volume_type}` only after confirming the variant option exists.
7. Accept only source enum strings `Direct Drive` / `Bowden` and `Standard` / `High Flow` when lookup proceeds.
8. Unknown enum strings return `SliceError::InvalidInput` at the Ares boundary only when the variant option exists.
9. Iterate exactly `0..variant_values.len()` in source order.
10. For each matching variant string, compute generated ID for that variant index.
11. If `extruder_variant_list` is absent, generated ID is `0`.
12. If `extruder_variant_list` is present, it must be a non-empty string array.
13. Generated ID loop iterates `0..extruder_variant_list.values.len()` and reads `extruder_variant_list.get_at(extruder_index)`.
14. Split each `extruder_variant_list` string by comma with source `boost::split(..., token_compress_on)` semantics: repeated adjacent commas are one separator, while leading/trailing separators produce empty boundary tokens.
15. Trim each split token like `boost::trim`, skip empty trimmed tokens, and increment the generated variant counter only for non-empty trimmed tokens.
16. When the generated variant counter equals the target variant index, return `extruder_index + 1` as the generated ID.
17. If no generated variant counter reaches the target variant index, generated ID is `0`.
18. Return the first matching `index * stride` as `isize` when generated ID equals `lookup.extruder_or_filament_id`.
19. Allow `stride == 0`, matching the source `unsigned int stride` multiplication behavior.
20. Return `SliceError::InvalidInput` if public `usize` multiplication cannot fit the Rust `isize` return type.
21. Return `Ok(-1)` when no variant+generated-ID pair matches.
22. Invalid public boundary values return `SliceError::InvalidInput`: present variant is not an array, present variant array is empty, variant member is not a string, missing/non-array/empty/complete ID vector, non-integer or non-`i32` ID member, present `extruder_variant_list` is not a string array, enum strings are unknown, or `index * stride` overflows the Rust return type.
23. Do not add preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove missing `variant_name` returns `-1`, including when enum strings, ID options, and `extruder_variant_list` are invalid or absent.
- Tests prove incomplete ID map uses generated IDs, not existing short IDs.
- Tests prove generated IDs are `extruder_index + 1` based on non-empty trimmed variant token order in `extruder_variant_list`.
- Tests prove missing `extruder_variant_list` generates ID `0`, allowing target ID `0` to match and nonzero target IDs not to match.
- Tests prove generated ID fallback returns `0` when the target variant index is beyond generated non-empty tokens.
- Tests prove comma split, trim, token-compress, and empty-token skipping edge cases affect generated ID order.
- Tests prove representative option names work with generated IDs.
- Tests prove first variant+generated-ID match returns `index * stride`, duplicate variants continue until generated ID matches, and no pair match returns `-1`.
- Tests prove all four valid source enum string combinations generate expected target variant strings while checking generated IDs.
- Tests prove `stride == 0` returns `0` for a match and overflow returns `SliceError::InvalidInput`.
- Tests prove invalid variant, ID, `extruder_variant_list`, and enum boundary values return `SliceError::InvalidInput` when lookup proceeds.
- Plan/spec explicitly account for deferred preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
