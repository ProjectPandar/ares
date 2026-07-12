# M219 Spec: DynamicPrintConfig update_values_from_multi_to_multi string/int copy

## Goal

Port the guard, variant-index preparation, and string/int-vector copy branches of OrcaSlicer's `DynamicPrintConfig::update_values_from_multi_to_multi(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9064`: `DynamicPrintConfig::update_values_from_multi_to_multi(...)`, limited to required current/new variant and new-id guards, variant-index preparation, config-definition/key lookup behavior, and `coStrings` / `coInts` copy branches.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:671`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8984-9017`: `new_extruder_variants`, `new_variant_indices`, and `extruder_variant_indices` preparation.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8988-8993`: missing current `this->option(variant_name)`, new `new_config.option(variant_name)`, or new `new_config.option(id_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9019-9031`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9032-9064`: `coStrings` and `coInts` copy `src_opt->values` from `new_config` into `this->option(key, true)->values` when the new source option exists.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`: representative string/int vector option context.

## Deferred behavior

- `coFloats`, `coFloatsOrPercents`, and `coBools` old-value merge branches from `PrintConfig.cpp:9065-9155`.
- `update_values_from_multi_to_multi_2` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Create `crates/ares-core/src/options/update_multi_to_multi.rs` with `SliceOptions::update_values_from_multi_to_multi_string_int_keys(MultiToMultiStringIntUpdate { ... }) -> Result<isize, SliceError>`.
- Register `mod update_multi_to_multi;` from `crates/ares-core/src/options.rs`.
- Create `crates/ares-core/src/options/tests/update_multi_to_multi.rs` and register it from `crates/ares-core/src/options/tests.rs`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_multi_to_multi_string_int_keys(...) -> Result<isize, SliceError>`.
2. If `self` lacks `variant_name`, or `new_config` lacks `variant_name`, or `new_config` lacks `id_name`, return `Ok(-1)` and leave `self` unchanged.
3. Present `variant_name` values must be string arrays and present `id_name` in `new_config` must be an i32 array. Returning `SliceError::InvalidInput` for present malformed JSON values is an Ares boundary deviation from C++ `dynamic_cast` returning `-1`.
4. Compute `extruder_variant_indices` for each entry in `new_extruder_variants`: collect indices in the current variant array equal to the requested variant; if none are found, use all current variant indices in source order.
5. Compute `new_variant_indices` for each entry in `new_extruder_variants`: find the first index in the new variant/id arrays where `new_id == i + 1` and `new_variant == new_extruder_variants[i]`; otherwise keep `-1`. This is prepared for later M220+ float/FloatOrPercent/bool merge slices.
6. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
7. If a key has no Ares registry definition, skip it without error.
8. If a key has `OptionValueKind::Strings` and `new_config` contains that key, validate it as a string array and copy the full JSON array into `self` under the same key.
9. If a key has `OptionValueKind::Ints` and `new_config` contains that key, validate it as an array of Rust `i32` / C++ `int` values and copy the full JSON array into `self` under the same key.
10. If a supported key is absent from `new_config`, skip it without creating or changing `self` values.
11. If a key has any other `OptionValueKind`, skip it in M219.
12. Invalid present source values for copied keys return `SliceError::InvalidInput`: string-vector key is not an array or contains non-strings; int-vector key is not an array, contains non-integers, or contains integers outside `i32` range.
13. If any key validation fails, return `SliceError::InvalidInput` and leave `self` unchanged.
14. Return `Ok(0)` after successful processing.
15. Do not add float, FloatOrPercent, bool merge, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove missing current variant, missing new variant, and missing new id return `-1` and leave `self` unchanged even when copied key values in `new_config` are invalid.
- Tests prove present malformed current/new variant or new id arrays return `SliceError::InvalidInput` and leave `self` unchanged.
- Tests prove `Strings` keys copy new source arrays and overwrite existing `self` values.
- Tests prove `Ints` keys copy new source arrays and overwrite existing `self` values.
- Tests prove missing source values for supported keys leave existing `self` values unchanged.
- Tests prove keys are processed in sorted unique order and unknown keys are skipped.
- Tests prove unsupported option kinds are skipped.
- Tests prove copied string/int source boundary errors return `SliceError::InvalidInput` with no partial mutation.
- Tests prove representative option names from the source context work: `printer_extruder_variant`, `print_extruder_variant`, `filament_extruder_variant`, `printer_extruder_id`, `print_extruder_id`, and `filament_self_index`.
- Tests exercise non-empty `new_extruder_variants` and guard/index preparation without exposing a separate Ares pipeline API.
- Plan/spec explicitly account for deferred float, FloatOrPercent, bool merge, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
