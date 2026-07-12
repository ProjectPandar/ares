# M218 Spec: DynamicPrintConfig update_values_from_single_to_multi bool resize

## Goal

Port the bool-vector resize-only branch of OrcaSlicer's `DynamicPrintConfig::update_values_from_single_to_multi(...)` into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8908`: `DynamicPrintConfig::update_values_from_single_to_multi(...)`, limited to the existing guard/key lookup behavior plus the `coBools` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:670`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8826-8831`: missing `multi_config.option(variant_name)` returns `-1`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8833-8843`: missing config definition returns `-1`; missing key definition logs warning and continues. Ares has a static registry, so missing config definition is not representable; missing option definition maps to skip.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8897-8908`: `coBools` obtains a source vector, creates/obtains the target option, asserts `variant_count == src_opt->size()`, and resizes the target to `variant_count`; it does not copy source bool values after resize.
- `OrcaSlicer/src/libslic3r/Config.hpp:635-662`: vector resize duplicates the first value when extending and truncates when shrinking; if empty, default option data is used.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1800-1804`, `2252-2255`, `2812-2816`, and `6628-6633`: representative `coBools` option context for `activate_air_filtration`, `enable_pressure_advance`, `filament_is_support`, and `wipe`.

## Deferred behavior

- `update_values_from_multi_to_multi` and other update helpers.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend the existing `crates/ares-core/src/options/update_single_to_multi.rs` implementation from M215-M217 to include `OptionValueKind::Bools` behavior.
- Rename the public helper to `SliceOptions::update_values_from_single_to_multi_string_int_float_percent_bool_keys(&mut self, multi_config: &SliceOptions, key_set: &[&str], variant_name: &str) -> Result<isize, SliceError>` so the API name remains accurate; do not keep a compatibility shim for the old partial name.
- Update `crates/ares-core/src/options/tests/update_single_to_multi/` to call the renamed helper and add bool-specific tests in `bool.rs`.
- Do not create new crates or dependencies.

## Functional requirements

1. Missing `variant_name` returns `Ok(-1)` and leaves `self` unchanged before validating any key values.
2. A present `variant_name` must be a string array. Its length is `variant_count`. Empty arrays are valid for this slice only when present bool source arrays also have length zero.
3. Process `key_set` in sorted unique order to match source `std::set<std::string>& key_set` iteration.
4. Unknown registry keys are skipped without error.
5. Missing source values are skipped without creating or changing `self` values.
6. Existing `Strings`, `Ints`, `Floats`, and `FloatOrPercent` behavior from M215-M217 remains unchanged under the renamed helper.
7. Unsupported option kinds other than `Strings`, `Ints`, `Floats`, `FloatOrPercent`, and `Bools` are skipped.
8. A present `OptionValueKind::Bools` source value must be a JSON array of booleans.
9. A present `OptionValueKind::Bools` source array must have length exactly `variant_count`; length mismatch or invalid entries return `SliceError::InvalidInput`.
10. A present `OptionValueKind::Bools` target value must be a JSON array of booleans. An absent target value is materialized from the registry default's first bool value before resize.
11. Resize target bool arrays to `variant_count` using source `ConfigOptionVector::resize` semantics: truncate when longer; extend by duplicating the first existing target value; when absent, fill with the registry default first bool value.
12. After resize, do not copy or limit against source bool entries. The output is the resized target/default array.
13. If any key validation fails, return `SliceError::InvalidInput` and leave `self` unchanged.
14. Return `Ok(0)` after successful processing.
15. Do not add preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove the renamed helper preserves M215 string/int, M216 float, and M217 FloatOrPercent behavior.
- Tests prove missing `variant_name` returns `-1` and leaves `self` unchanged before invalid bool source values are inspected.
- Tests prove bool source values are not copied into existing target values when sizes already match.
- Tests prove target bool arrays are truncated to `variant_count` without copying source values.
- Tests prove target bool arrays are extended by duplicating the first target value without copying source values.
- Tests prove absent target bool arrays are materialized with registry defaults and not overwritten by source values.
- Tests prove source bool arrays must match `variant_count`, including accepting empty source arrays only when `variant_count == 0`.
- Tests prove invalid present source/target bool arrays return `SliceError::InvalidInput` and leave `self` unchanged.
- Tests prove representative bool option names from the source context work: `activate_air_filtration`, `enable_pressure_advance`, `filament_is_support`, and `wipe`.
- Plan/spec explicitly account for deferred preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
