# M217 Spec: DynamicPrintConfig update_values_from_single_to_multi FloatOrPercent limit

## Goal

Port the FloatOrPercent-vector resize-and-limit branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_single_to_multi(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8896`: `DynamicPrintConfig::update_values_from_single_to_multi(...)`, limited to the existing guard/key lookup behavior plus the `coFloatsOrPercents` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:670`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8826-8831`: missing `multi_config.option(variant_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8833-8843`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8881-8896`: `coFloatsOrPercents` obtains a source vector, creates/obtains the target option, asserts `variant_count == src_opt->size()`, resizes the target to `variant_count`, and replaces each target entry only when `target.value > source.value`.
- `OrcaSlicer/src/libslic3r/Config.hpp:31-39`: `FloatOrPercent { double value; bool percent; }` data shape.
- `OrcaSlicer/src/libslic3r/Config.hpp:635-662`: vector resize duplicates the first value when extending and truncates when shrinking; if empty, default option data is used.
- `OrcaSlicer/src/libslic3r/Config.hpp:1318-1448`: `ConfigOptionFloatsOrPercents` vector parse/serialize context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2037`, `2322-2332`, and `3104-3112`: representative `coFloatOrPercent` option context for `outer_wall_line_width`, `line_width`, and `bridge_acceleration`.

## Deferred behavior

- `coBools` branch from `PrintConfig.cpp:8897-8908`.
- `update_values_from_multi_to_multi` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend the existing `crates/ares-core/src/options/update_single_to_multi.rs` implementation from M215/M216 to include `OptionValueKind::FloatOrPercent` behavior.
- Rename the public helper to `SliceOptions::update_values_from_single_to_multi_string_int_float_percent_keys(&mut self, multi_config: &SliceOptions, key_set: &[&str], variant_name: &str) -> Result<isize, SliceError>` so the API name remains accurate; do not keep a compatibility shim for the old partial name.
- Update `crates/ares-core/src/options/tests/update_single_to_multi.rs` to call the renamed helper and add FloatOrPercent-specific tests.
- Do not create new crates or dependencies.

## Functional requirements

1. Missing `variant_name` returns `Ok(-1)` and leaves `self` unchanged before validating any key values.
2. A present `variant_name` must be a string array. Its length is `variant_count`. Empty arrays are valid for this slice only when present copied FloatOrPercent source arrays also have length zero.
3. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
4. Unknown registry keys are skipped without error.
5. Missing source values are skipped without creating or changing `self` values.
6. Existing `Strings`, `Ints`, and `Floats` behavior from M215/M216 remains unchanged under the renamed helper.
7. Unsupported option kinds other than `Strings`, `Ints`, `Floats`, and `FloatOrPercent` are skipped.
8. A present `OptionValueKind::FloatOrPercent` source value must be a JSON array of finite numbers or finite numeric strings. Strings ending with `%` represent `{ value, percent: true }`; numbers and numeric strings without `%` represent `{ value, percent: false }`.
9. A present `OptionValueKind::FloatOrPercent` source array must have length exactly `variant_count`; length mismatch or invalid entries return `SliceError::InvalidInput`.
10. A present `OptionValueKind::FloatOrPercent` target value must be a JSON array of finite numbers or finite numeric strings using the same representation. An absent target value is materialized from the registry default's first FloatOrPercent value before resize.
11. Resize target FloatOrPercent arrays to `variant_count` using source `ConfigOptionVector::resize` semantics: truncate when longer; extend by duplicating the first existing target value; when absent, fill with the registry default first FloatOrPercent value.
12. After resize, for every index, set the target entry to the complete source entry only if `target.value > source.value`; otherwise leave the target entry unchanged, including its percent flag.
13. Serialized JSON output preserves absolute values as numbers and percent values as strings with `%` suffix.
14. If any key validation fails, return `SliceError::InvalidInput` and leave `self` unchanged.
15. Return `Ok(0)` after successful processing.
16. Do not add bool resize, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove the renamed helper preserves M215 string/int and M216 float behavior.
- Tests prove missing `variant_name` returns `-1` and leaves `self` unchanged before invalid FloatOrPercent source values are inspected.
- Tests prove unknown keys and unsupported kinds are skipped with FloatOrPercent keys present.
- Tests prove an absolute target value larger than the source value is clamped down to the complete source entry.
- Tests prove target values less than or equal to source values are preserved, including target percent flags.
- Tests prove source percent entries preserve percent flags when they replace target entries.
- Tests prove target FloatOrPercent arrays are truncated to `variant_count` before per-index limiting.
- Tests prove target FloatOrPercent arrays are extended by duplicating the first target entry before per-index limiting.
- Tests prove absent target FloatOrPercent arrays are materialized with registry defaults and then limited by source values.
- Tests prove source FloatOrPercent arrays must match `variant_count`, including accepting empty source arrays only when `variant_count == 0`.
- Tests prove invalid present source/target FloatOrPercent arrays return `SliceError::InvalidInput` and leave `self` unchanged.
- Tests prove representative FloatOrPercent option names from the source context work: `outer_wall_line_width`, `line_width`, and `bridge_acceleration`.
- Plan/spec explicitly account for deferred bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
