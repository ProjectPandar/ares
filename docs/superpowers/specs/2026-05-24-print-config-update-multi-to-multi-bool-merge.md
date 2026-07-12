# M222 Spec: DynamicPrintConfig update_values_from_multi_to_multi bool merge

## Goal

Port the `coBools` old-value merge branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi(...)` into `ares-core`, building directly on the M219-M221 helper without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9155`: `DynamicPrintConfig::update_values_from_multi_to_multi(...)`, limited to required current/new variant and new-id guards, variant-index preparation, config-definition/key lookup behavior, existing M219-M221 `coStrings` / `coInts` / `coFloats` / `coFloatsOrPercents`, and M222 `coBools` merge branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:671`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8984-9017`: `new_extruder_variants`, `new_variant_indices`, and `extruder_variant_indices` preparation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8988-8993`: missing current `this->option(variant_name)`, new `new_config.option(variant_name)`, or new `new_config.option(id_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9019-9031`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9032-9123`: existing M219-M221 string, int, float, and FloatOrPercent behavior must remain intact.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9125-9155`: `coBools` copies `src_opt->values`, then sets each matching new variant value to true if any same-variant old value was true.
- `OrcaSlicer/src/libslic3r/Config.hpp:635-662`: vector option resize/default context for bool vector values.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1800-1804`, `2252-2255`, `2812-2816`, and `6628-6633`: representative bool option context.

## Deferred behavior

- `update_values_from_multi_to_multi_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi.rs`; a small `options::update_multi_to_multi::bools` module may be introduced if needed to keep files under 400 LOC.
- Rename the public helper to `SliceOptions::update_values_from_multi_to_multi_string_int_float_percent_bool_keys(MultiToMultiUpdate { ... }) -> Result<isize, SliceError>` so the helper name reflects the widened source slice.
- Keep `MultiToMultiUpdate` as the parameter struct.
- Update tests in `crates/ares-core/src/options/tests/update_multi_to_multi/`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_multi_to_multi_string_int_float_percent_bool_keys(...) -> Result<isize, SliceError>` using existing `MultiToMultiUpdate` parameters.
2. Remove the M221-only public helper name rather than keeping a legacy fallback alias.
3. If `self` lacks `variant_name`, or `new_config` lacks `variant_name`, or `new_config` lacks `id_name`, return `Ok(-1)` and leave `self` unchanged.
4. Present `variant_name` values must be string arrays and present `id_name` in `new_config` must be an i32 array. Returning `SliceError::InvalidInput` for present malformed JSON values is an Ares boundary deviation from C++ `dynamic_cast` returning `-1`.
5. Compute `extruder_variant_indices` for each entry in `new_extruder_variants`: collect indices in the current variant array equal to the requested variant; if none are found, use all current variant indices in source order.
6. Compute `new_variant_indices` for each entry in `new_extruder_variants`: find the first index in the new variant/id arrays where `new_id == i + 1` and `new_variant == new_extruder_variants[i]`; otherwise keep `-1`.
7. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
8. If a key has no Ares registry definition, skip it without error.
9. Preserve M219-M221 behavior for `OptionValueKind::Strings`, `OptionValueKind::Ints`, `OptionValueKind::Floats`, and `OptionValueKind::FloatOrPercent`.
10. For `OptionValueKind::Bools` keys present in `new_config`, validate the source as an array of bool values and validate the current `self` value as the same shape.
11. If a bool key is absent from `self`, use the registry default first value repeated to `variant_count` as the old values. This is the Ares equivalent of `this->option<ConfigOptionBools>(key, true)` creating the option before reading old values.
12. Bool old value length must equal `variant_count`; source bool length must equal `new_variant_count`; mismatches return `SliceError::InvalidInput` with no mutation.
13. Bool merge starts from a copy of the full source array.
14. For each requested new extruder variant index `i`, if `new_variant_indices[i] == -1`, leave source-copied values unchanged for that variant.
15. Otherwise, iterate all old indices in `extruder_variant_indices[i]`; if any `old_values[idx]` is true, set `merged_values[new_variant_index]` to true.
16. If a requested new extruder variant has no same-variant old indices, use the source fallback of all old indices, allowing any old true value to be preserved.
17. Invalid present values for copied/merged keys return `SliceError::InvalidInput` with no partial `self` mutation.
18. Keys with any other `OptionValueKind` are skipped in M222.
19. Return `Ok(0)` after successful processing.
20. Do not add `update_values_from_multi_to_multi_2`, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M219-M221 string/int/float/FloatOrPercent tests continue to pass through the renamed widened helper.
- Tests prove bool keys copy source values when no old value is true.
- Tests prove bool keys preserve old same-variant true values over copied false values.
- Tests prove duplicate old variants consider all same-variant old indices and any true old value wins.
- Tests prove absent old variant fallback considers all old indices.
- Tests prove missing new variant index leaves the copied source value unchanged.
- Tests prove missing current bool value uses the registry default repeated to `variant_count` before merge.
- Tests prove invalid bool source/current values and old/source length assertion mismatches return `SliceError::InvalidInput` with no partial mutation.
- Tests prove representative bool option names from the source context work: `activate_air_filtration`, `enable_pressure_advance`, `filament_is_support`, and `wipe`.
- Plan/spec explicitly account for deferred preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
