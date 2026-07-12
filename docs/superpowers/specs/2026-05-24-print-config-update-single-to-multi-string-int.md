# M215 Spec: DynamicPrintConfig update_values_from_single_to_multi string/int copy

## Goal

Port the string-vector and int-vector copy branches of OrcaSlicer's `DynamicPrintConfig::update_values_from_single_to_multi(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8865`: `DynamicPrintConfig::update_values_from_single_to_multi(...)`, limited to the required `variant_name` lookup, config-definition/key lookup behavior, and `coStrings` / `coInts` copy branches.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:670`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8826-8831`: missing `multi_config.option(variant_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8833-8843`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8844-8865`: `coStrings` and `coInts` copy `src_opt->values` into `this->option(key, true)->values` when the source option exists.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`: representative string/int vector option context.

## Deferred behavior

- `coFloats`, `coFloatsOrPercents`, and `coBools` branches from `PrintConfig.cpp:8866-8908`.
- `update_values_from_multi_to_multi` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Create `crates/ares-core/src/options/update_single_to_multi.rs` with `SliceOptions::update_values_from_single_to_multi_string_int_keys(&mut self, multi_config: &SliceOptions, key_set: &[&str], variant_name: &str) -> Result<isize, SliceError>`. The C++ `id_name` parameter is intentionally omitted because it is unused by the selected `coStrings`/`coInts` source branch.
- Register `mod update_single_to_multi;` from `crates/ares-core/src/options.rs`.
- Create `crates/ares-core/src/options/tests/update_single_to_multi.rs` and register it from `crates/ares-core/src/options/tests.rs`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_single_to_multi_string_int_keys(...) -> Result<isize, SliceError>`.
2. If `multi_config` lacks `variant_name`, return `Ok(-1)` and leave `self` unchanged.
3. If `variant_name` exists, it must be a string array. Empty arrays are valid for this M215 slice because the string/int branches do not use `variant_count`. Returning `SliceError::InvalidInput` for a present non-string-array JSON value is an Ares boundary deviation from C++ `dynamic_cast` returning `-1`.
4. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
5. If a key has no Ares registry definition, skip it without error.
6. If a key has `OptionValueKind::Strings` and `multi_config` contains that key, validate it as a string array and copy the full JSON array into `self` under the same key.
7. If a key has `OptionValueKind::Ints` and `multi_config` contains that key, validate it as an array of Rust `i32` / C++ `int` values and copy the full JSON array into `self` under the same key.
8. If a supported key is absent from `multi_config`, skip it without creating or changing `self` values.
9. If a key has any other `OptionValueKind`, skip it in M215.
10. Invalid present source values for copied keys return `SliceError::InvalidInput`: string-vector key is not an array or contains non-strings; int-vector key is not an array, contains non-integers, or contains integers outside `i32` range.
11. Return `Ok(0)` after successful processing.
12. Do not add float, FloatOrPercent, bool resize, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove missing `variant_name` returns `-1` and leaves `self` unchanged even when key values in `multi_config` are invalid.
- Tests prove present empty string-array `variant_name` is accepted.
- Tests prove `Strings` keys copy source arrays and overwrite existing `self` values.
- Tests prove `Ints` keys copy source arrays and overwrite existing `self` values.
- Tests prove missing source values for supported keys leave existing `self` values unchanged.
- Tests prove keys are processed in sorted unique order and unknown keys are skipped.
- Tests prove unsupported option kinds are skipped.
- Tests prove copied string/int source boundary errors return `SliceError::InvalidInput`.
- Tests prove representative option names from the source context work: `printer_extruder_variant`, `print_extruder_variant`, `filament_extruder_variant`, `printer_extruder_id`, `print_extruder_id`, and `filament_self_index`.
- Plan/spec explicitly account for deferred float, FloatOrPercent, bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
