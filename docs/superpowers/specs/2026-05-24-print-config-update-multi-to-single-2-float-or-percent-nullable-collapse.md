# M227 Spec: DynamicPrintConfig update_values_from_multi_to_single_2 FloatOrPercent nullable collapse

## Goal

Port the `coFloatsOrPercents` nullable FloatOrPercent branch of OrcaSlicer's commented `DynamicPrintConfig::update_values_from_multi_to_single_2(...)` helper into `ares-core`, building on M226 without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9344`: commented `DynamicPrintConfig::update_values_from_multi_to_single_2(...)`, limited to setup, existing M226 float branch context, and the M227 `coFloatsOrPercents` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:673-674`: commented declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9290-9304`: config-definition guard, source-key iteration, key-set filtering, option-definition lookup, and unknown-definition skip behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9326-9344`: `coFloatsOrPercents` branch using `ConfigOptionFloatsOrPercentsNullable`, `FloatOrPercent min{9999.f, true}`, nil skip, strict raw `.value < min.value` minimum selection, erase from index `1` to end, and optional index `0` overwrite.
- `OrcaSlicer/src/libslic3r/Config.hpp:31-42`: `FloatOrPercent { value, percent }` data shape and ordering context. M227 uses only the branch's raw `.value < min.value` comparison.
- `OrcaSlicer/src/libslic3r/Config.hpp:1344-1345` and `Config.hpp:1450`: nullable FloatOrPercent nil is `NaN` in `.value` and `is_nil(idx)` checks `std::isnan(value)`.

## Deferred behavior

- `coBools` branch from `PrintConfig.cpp:9345-9363`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi/multi_to_single_2.rs`.
- Keep the existing `SliceOptions::update_values_from_multi_to_single_2_float_keys(...) -> Result<isize, SliceError>` API name for this combined M226/M227 helper.
- Extend focused tests in `crates/ares-core/src/options/tests/update_multi_to_single_2.rs`; split the test file only if needed to preserve the `<400 LOC` rule.
- Do not create new crates or dependencies.

## Functional requirements

1. Preserve every M226 `OptionValueKind::Floats` and `OptionValueKind::FloatsNullable` behavior unchanged.
2. Extend `update_values_from_multi_to_single_2_float_keys(...)` to also handle `OptionValueKind::FloatOrPercent` keys.
3. Iterate keys already present in `self.values()` in sorted map order and skip keys absent from `key_set`.
4. Skip any key with no Ares registry definition.
5. Skip every unsupported kind other than `Floats`, `FloatsNullable`, and `FloatOrPercent`.
6. Validate handled FloatOrPercent values as nullable FloatOrPercent vectors where JSON numbers are absolute values, strings ending in `%` are percent values, and JSON string `"nil"` is nil.
7. Empty handled FloatOrPercent vectors return `SliceError::InvalidInput` with no mutation because the upstream branch keeps and optionally overwrites index `0`.
8. For each handled FloatOrPercent key, scan every source index in source order, ignore nil entries, initialize the candidate to `9999%`, and replace it only when a non-nil source value has numeric `.value < candidate.value`.
9. Equal numeric source values keep the first selected candidate because the upstream branch compares only strict `<`; source values equal to or greater than the initial `9999` sentinel do not replace the sentinel and therefore do not select.
10. Collapse each handled vector to one output element.
11. If a selected source value exists, write it at index `0` while preserving its percent flag; otherwise keep the original first entry after collapse.
12. Invalid present source values return `SliceError::InvalidInput` with no partial mutation.
13. Collect all resulting key/value pairs before mutating `self` so later invalid keys do not partially update earlier keys.
14. Return `Ok(0)` after successful processing.
15. Do not add bool, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M226 nullable/non-nullable float tests continue to pass unchanged.
- Tests prove FloatOrPercent vectors collapse to one entry and write the selected source value below the `9999%` sentinel while preserving the selected value's percent flag.
- Tests prove nil entries are skipped and all-nil vectors preserve the original first nil while collapsing to one entry.
- Tests prove equal numeric source values keep the first selected candidate, not `FloatOrPercent::operator<` percent ordering; the suite must include a case such as `["50%", 50.0] -> ["50%"]`.
- Tests prove source values equal to or greater than `9999`, including exactly `9999`, preserve the original first entry after collapse.
- Tests prove invalid FloatOrPercent values and empty handled arrays return `SliceError::InvalidInput` with no mutation.
- Tests prove multi-key no-partial-mutation when an earlier valid FloatOrPercent key is followed by a later invalid FloatOrPercent key.
- Tests prove representative FloatOrPercent option names work from existing registry coverage, such as `bridge_acceleration`, `line_width`, and `outer_wall_line_width`.
- Plan/spec explicitly account for deferred bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
