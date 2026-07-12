# M216 Spec: DynamicPrintConfig update_values_from_single_to_multi float limit

## Goal

Port the float-vector resize-and-limit branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_single_to_multi(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8880`: `DynamicPrintConfig::update_values_from_single_to_multi(...)`, limited to the existing guard/key lookup behavior plus the `coFloats` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:670`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8826-8831`: missing `multi_config.option(variant_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8833-8843`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8866-8880`: `coFloats` obtains a source float vector, creates/obtains the target option, asserts `variant_count == src_opt->size()`, resizes the target to `variant_count`, and replaces each target entry only when the target value is greater than the source value.
- `OrcaSlicer/src/libslic3r/Config.hpp:635-662`: vector resize duplicates the first value when extending and truncates when shrinking; if empty, default option data is used.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: `ConfigOptionFloats` vector context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:766-773`, `2349-2357`, and `4591-4599`: representative `coFloats` option context for `extruder_printable_height`, `fan_cooling_layer_time`, and `fan_max_speed`.

## Deferred behavior

- `coFloatsOrPercents` and `coBools` branches from `PrintConfig.cpp:8881-8908`.
- `update_values_from_multi_to_multi` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend the existing `crates/ares-core/src/options/update_single_to_multi.rs` implementation from M215 to include `OptionValueKind::Floats` behavior.
- Rename the public helper to `SliceOptions::update_values_from_single_to_multi_string_int_float_keys(&mut self, multi_config: &SliceOptions, key_set: &[&str], variant_name: &str) -> Result<isize, SliceError>` so the API name remains accurate; do not keep a compatibility shim for the old partial name.
- Update `crates/ares-core/src/options/tests/update_single_to_multi.rs` to call the renamed helper and add float-specific tests.
- Do not create new crates or dependencies.

## Functional requirements

1. Missing `variant_name` returns `Ok(-1)` and leaves `self` unchanged before validating any key values.
2. A present `variant_name` must be a string array. Its length is `variant_count`. Empty arrays are valid for this slice only when present copied float source arrays also have length zero.
3. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
4. Unknown registry keys are skipped without error.
5. Missing source values are skipped without creating or changing `self` values.
6. Existing `Strings` and `Ints` copy behavior from M215 remains unchanged under the renamed helper.
7. Unsupported option kinds other than `Strings`, `Ints`, and `Floats` are skipped.
8. A present `OptionValueKind::Floats` source value must be a JSON array of finite numbers and have length exactly `variant_count`; length mismatch or non-finite/non-number entries return `SliceError::InvalidInput`.
9. A present `OptionValueKind::Floats` target value must be a JSON array of finite numbers. An absent target value is materialized from the registry default's first numeric value before resize.
10. Resize target float arrays to `variant_count` using source `ConfigOptionVector::resize` semantics: truncate when longer; extend by duplicating the first existing target value; when absent, fill with the registry default first numeric value.
11. After resize, for every index, set the target entry to the source entry only if `target[index] > source[index]`; otherwise leave the target entry unchanged.
12. If any key validation fails, return `SliceError::InvalidInput` and leave `self` unchanged.
13. Return `Ok(0)` after successful processing.
14. Do not add `FloatOrPercent`, bool resize, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove the renamed helper preserves M215 string/int copy behavior.
- Tests prove missing `variant_name` returns `-1` and leaves `self` unchanged before invalid float source values are inspected.
- Tests prove unknown keys and unsupported kinds are skipped with float keys present.
- Tests prove a float target value larger than the source is clamped down to the source value.
- Tests prove a float target value less than or equal to the source is preserved.
- Tests prove target float arrays are truncated to `variant_count` before per-index limiting.
- Tests prove target float arrays are extended by duplicating the first target value before per-index limiting.
- Tests prove absent target float arrays are materialized with registry defaults and then limited by source values.
- Tests prove source float arrays must match `variant_count`, including accepting empty source arrays only when `variant_count == 0`.
- Tests prove invalid present source/target float arrays return `SliceError::InvalidInput` and leave `self` unchanged.
- Tests prove representative float option names from the source context work: `extruder_printable_height`, `fan_cooling_layer_time`, and `fan_max_speed`.
- Plan/spec explicitly account for deferred FloatOrPercent, bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
