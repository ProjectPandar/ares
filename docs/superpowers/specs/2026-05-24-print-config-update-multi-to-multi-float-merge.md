# M220 Spec: DynamicPrintConfig update_values_from_multi_to_multi float merge

## Goal

Port the `coFloats` old-value merge branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi(...)` into `ares-core`, building directly on the M219 guard/index/string/int slice without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9093`: `DynamicPrintConfig::update_values_from_multi_to_multi(...)`, limited to required current/new variant and new-id guards, variant-index preparation, config-definition/key lookup behavior, M219 `coStrings` / `coInts`, and M220 `coFloats` merge branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:671`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8984-9017`: `new_extruder_variants`, `new_variant_indices`, and `extruder_variant_indices` preparation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8988-8993`: missing current `this->option(variant_name)`, new `new_config.option(variant_name)`, or new `new_config.option(id_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9019-9031`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9032-9064`: existing M219 `coStrings` and `coInts` full source copy behavior must remain intact.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9065-9093`: `coFloats` copies `src_opt->values`, then lowers each matching new variant value to smaller old values from same-variant old indices.
- `OrcaSlicer/src/libslic3r/Config.hpp:635-662`: vector option sizing/indexing context for source assertions.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:766-773`, `2349-2357`, and `4591-4599`: representative float vector option context.

## Deferred behavior

- `coFloatsOrPercents` and `coBools` old-value merge branches from `PrintConfig.cpp:9095-9155`.
- `update_values_from_multi_to_multi_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi.rs`.
- Replace the M219 temporary public API with `SliceOptions::update_values_from_multi_to_multi_string_int_float_keys(MultiToMultiUpdate { ... }) -> Result<isize, SliceError>` so the helper name and parameter struct reflect the widened source slice.
- Update `crates/ares-core/src/options.rs` and `crates/ares-core/src/lib.rs` re-exports from `MultiToMultiStringIntUpdate` to `MultiToMultiUpdate`.
- Extend `crates/ares-core/src/options/tests/update_multi_to_multi.rs`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_multi_to_multi_string_int_float_keys(...) -> Result<isize, SliceError>` using `MultiToMultiUpdate` parameters: `new_config`, `key_set`, `id_name`, `variant_name`, and `new_extruder_variants`.
2. Remove the M219-only public API name rather than keeping a legacy fallback alias.
3. If `self` lacks `variant_name`, or `new_config` lacks `variant_name`, or `new_config` lacks `id_name`, return `Ok(-1)` and leave `self` unchanged.
4. Present `variant_name` values must be string arrays and present `id_name` in `new_config` must be an i32 array. Returning `SliceError::InvalidInput` for present malformed JSON values is an Ares boundary deviation from C++ `dynamic_cast` returning `-1`.
5. Compute `extruder_variant_indices` for each entry in `new_extruder_variants`: collect indices in the current variant array equal to the requested variant; if none are found, use all current variant indices in source order.
6. Compute `new_variant_indices` for each entry in `new_extruder_variants`: find the first index in the new variant/id arrays where `new_id == i + 1` and `new_variant == new_extruder_variants[i]`; otherwise keep `-1`.
7. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
8. If a key has no Ares registry definition, skip it without error.
9. Preserve M219 behavior for `OptionValueKind::Strings` and `OptionValueKind::Ints`: if `new_config` contains that key, validate and copy the full JSON array into `self`; if absent, skip.
10. For `OptionValueKind::Floats` keys present in `new_config`, validate the source as a finite float array and validate the current `self` value as a finite float array.
11. If a float key is absent from `self`, use the registry default first value repeated to `variant_count` as the old values. This is the Ares equivalent of `this->option<ConfigOptionFloats>(key, true)` creating the option before reading old values.
12. Float old value length must equal `variant_count`; source float length must equal `new_variant_count`; mismatches return `SliceError::InvalidInput` with no mutation.
13. Float merge starts from a copy of the full source float array.
14. For each requested new extruder variant index `i`, if `new_variant_indices[i] == -1`, leave source-copied values unchanged for that variant.
15. Otherwise, iterate all old indices in `extruder_variant_indices[i]`; if `old_values[idx] < merged_values[new_variant_index]`, replace the merged new value with that lower old value.
16. If a requested new extruder variant has no same-variant old indices, use the source fallback of all old indices, allowing the minimum lower old value from all old variants to be preserved.
17. Invalid present values for copied/merged keys return `SliceError::InvalidInput` with no partial `self` mutation.
18. Keys with any other `OptionValueKind` are skipped in M220.
19. Return `Ok(0)` after successful processing.
20. Do not add FloatOrPercent, bool merge, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M219 string/int tests continue to pass through the renamed widened helper.
- Tests prove float keys copy source values when no old value is lower.
- Tests prove float keys preserve lower old same-variant values into matching new variant positions after source copy.
- Tests prove duplicate old variants consider all same-variant old indices and keep the lowest old value that is lower than the copied source value.
- Tests prove absent old variant fallback considers all old indices.
- Tests prove missing new variant index leaves the copied source value unchanged.
- Tests prove missing current float value uses the registry default repeated to `variant_count` before merge.
- Tests prove invalid float source/current values and old/source length assertion mismatches return `SliceError::InvalidInput` with no partial mutation.
- Tests prove representative float option names from the source context work: `extruder_printable_height`, `fan_cooling_layer_time`, and `fan_max_speed`.
- Plan/spec explicitly account for deferred FloatOrPercent, bool merge, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
