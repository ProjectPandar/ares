# M221 Spec: DynamicPrintConfig update_values_from_multi_to_multi FloatOrPercent merge

## Goal

Port the `coFloatsOrPercents` old-value merge branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi(...)` into `ares-core`, building directly on the M219/M220 helper without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9123`: `DynamicPrintConfig::update_values_from_multi_to_multi(...)`, limited to required current/new variant and new-id guards, variant-index preparation, config-definition/key lookup behavior, M219 `coStrings` / `coInts`, M220 `coFloats`, and M221 `coFloatsOrPercents` merge branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:671`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8984-9017`: `new_extruder_variants`, `new_variant_indices`, and `extruder_variant_indices` preparation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8988-8993`: missing current `this->option(variant_name)`, new `new_config.option(variant_name)`, or new `new_config.option(id_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9019-9031`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9032-9093`: existing M219/M220 string, int, and float behavior must remain intact.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9095-9123`: `coFloatsOrPercents` copies `src_opt->values`, then lowers each matching new variant value to smaller old values from same-variant old indices.
- `OrcaSlicer/src/libslic3r/Config.hpp:31-42`: `FloatOrPercent { value, percent }` data shape.
- `OrcaSlicer/src/libslic3r/Config.hpp:1318-1448`: `ConfigOptionFloatsOrPercentsTempl` vector serialization, deserialization, and comparison context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2037`, `2322-2332`, and `3104-3112`: representative FloatOrPercent option context.

## Deferred behavior

- `coBools` old-value merge branch from `PrintConfig.cpp:9125-9155`.
- `update_values_from_multi_to_multi_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi.rs`.
- Rename the public helper to `SliceOptions::update_values_from_multi_to_multi_string_int_float_percent_keys(MultiToMultiUpdate { ... }) -> Result<isize, SliceError>` so the helper name reflects the widened source slice.
- Keep `MultiToMultiUpdate` as the parameter struct.
- Update tests in `crates/ares-core/src/options/tests/update_multi_to_multi/`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_multi_to_multi_string_int_float_percent_keys(...) -> Result<isize, SliceError>` using existing `MultiToMultiUpdate` parameters.
2. Remove the M220-only public helper name rather than keeping a legacy fallback alias.
3. If `self` lacks `variant_name`, or `new_config` lacks `variant_name`, or `new_config` lacks `id_name`, return `Ok(-1)` and leave `self` unchanged.
4. Present `variant_name` values must be string arrays and present `id_name` in `new_config` must be an i32 array. Returning `SliceError::InvalidInput` for present malformed JSON values is an Ares boundary deviation from C++ `dynamic_cast` returning `-1`.
5. Compute `extruder_variant_indices` for each entry in `new_extruder_variants`: collect indices in the current variant array equal to the requested variant; if none are found, use all current variant indices in source order.
6. Compute `new_variant_indices` for each entry in `new_extruder_variants`: find the first index in the new variant/id arrays where `new_id == i + 1` and `new_variant == new_extruder_variants[i]`; otherwise keep `-1`.
7. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
8. If a key has no Ares registry definition, skip it without error.
9. Preserve M219/M220 behavior for `OptionValueKind::Strings`, `OptionValueKind::Ints`, and `OptionValueKind::Floats`.
10. For `OptionValueKind::FloatOrPercent` keys present in `new_config`, validate the source as an array of finite FloatOrPercent values and validate the current `self` value as the same shape.
11. Ares FloatOrPercent JSON accepts numbers as absolute values and strings ending in `%` as percent values; copied percent values remain strings with `%` and copied absolute values remain JSON numbers.
12. If a FloatOrPercent key is absent from `self`, use the registry default first value repeated to `variant_count` as the old values. This is the Ares equivalent of `this->option<ConfigOptionFloatsOrPercents>(key, true)` creating the option before reading old values.
13. FloatOrPercent old value length must equal `variant_count`; source FloatOrPercent length must equal `new_variant_count`; mismatches return `SliceError::InvalidInput` with no mutation.
14. FloatOrPercent merge starts from a copy of the full source array.
15. For each requested new extruder variant index `i`, if `new_variant_indices[i] == -1`, leave source-copied values unchanged for that variant.
16. Otherwise, iterate all old indices in `extruder_variant_indices[i]`; if `old_values[idx] < merged_values[new_variant_index]` under Orca `FloatOrPercent::operator<` (`Config.hpp:42`), replace the merged new value with that lower old value, preserving both numeric value and percent flag from the old value. Equal numeric values therefore prefer an old absolute value over a copied percent value.
17. If a requested new extruder variant has no same-variant old indices, use the source fallback of all old indices, allowing the minimum lower old value from all old variants to be preserved.
18. Invalid present values for copied/merged keys return `SliceError::InvalidInput` with no partial `self` mutation.
19. Keys with any other `OptionValueKind` are skipped in M221.
20. Return `Ok(0)` after successful processing.
21. Do not add bool merge, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M219/M220 string/int/float tests continue to pass through the renamed widened helper.
- Tests prove FloatOrPercent keys copy source values when no old value is lower, preserving absolute number and percent string representation.
- Tests prove FloatOrPercent keys preserve lower old same-variant values and preserve the old percent flag when selected.
- Tests prove equal numeric values use Orca `FloatOrPercent::operator<` percent ordering, preferring an old absolute value over a copied percent value.
- Tests prove duplicate old variants consider all same-variant old indices and keep the lowest old numeric value that is lower than the copied source value.
- Tests prove absent old variant fallback considers all old indices.
- Tests prove missing new variant index leaves the copied source value unchanged.
- Tests prove missing current FloatOrPercent value uses the registry default repeated to `variant_count` before merge.
- Tests prove invalid FloatOrPercent source/current values and old/source length assertion mismatches return `SliceError::InvalidInput` with no partial mutation.
- Tests prove representative FloatOrPercent option names from the source context work: `outer_wall_line_width`, `line_width`, and `bridge_acceleration`.
- Plan/spec explicitly account for deferred bool merge, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
