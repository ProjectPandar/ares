# M228 Spec: DynamicPrintConfig update_values_from_multi_to_single_2 bool nullable collapse

## Goal

Port the `coBools` nullable bool branch of OrcaSlicer's commented `DynamicPrintConfig::update_values_from_multi_to_single_2(...)` helper into `ares-core`, completing this helper's three option branches without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9371`: commented `DynamicPrintConfig::update_values_from_multi_to_single_2(...)`, limited to setup, existing M226/M227 branch context, and the M228 `coBools` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:673-674`: commented declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9290-9304`: config-definition guard, source-key iteration, key-set filtering, option-definition lookup, and unknown-definition skip behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9345-9363`: `coBools` branch using `ConfigOptionBoolsNullable`, nil skip, first non-nil bool selection, erase from index `1` to end, and optional index `0` overwrite.
- `OrcaSlicer/src/libslic3r/Config.hpp:1857-1967`: nullable bool vector stores `unsigned char`, uses `std::numeric_limits<unsigned char>::max()` as nil, and serializes/deserializes bool vector values as `1`, `0`, or `nil`.

## Deferred behavior

- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Extend `crates/ares-core/src/options/update_multi_to_multi/multi_to_single_2.rs`.
- Keep the existing `SliceOptions::update_values_from_multi_to_single_2_float_keys(...) -> Result<isize, SliceError>` API name for this combined M226/M227/M228 helper.
- Extend focused tests in `crates/ares-core/src/options/tests/update_multi_to_single_2.rs`; split the test file only if needed to preserve the `<400 LOC` rule.
- Do not create new crates or dependencies.

## Functional requirements

1. Preserve every M226 and M227 behavior unchanged.
2. Extend `update_values_from_multi_to_single_2_float_keys(...)` to also handle `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable` keys.
3. Iterate keys already present in `self.values()` in sorted map order and skip keys absent from `key_set`.
4. Skip any key with no Ares registry definition.
5. Skip every unsupported kind other than `Floats`, `FloatsNullable`, `FloatOrPercent`, `Bools`, and `BoolsNullable`.
6. Validate handled bool values as nullable bool vectors where JSON booleans are bool values, numeric `1`/`0` and string `"1"`/`"0"` are bool values, JSON string `"nil"` is nil, and string `"true"`/`"false"` are rejected because upstream serialized bool vectors accept only `1`, `0`, or `nil` when substitution is disabled.
7. Empty handled bool vectors return `SliceError::InvalidInput` with no mutation because the upstream branch keeps and optionally overwrites index `0`.
8. For each handled bool key, scan every source index in source order and ignore nil entries.
9. If at least one non-nil source value is found, write the first non-nil bool to index `0`; `false` is a real non-nil selected value.
10. Collapse each handled vector to one output element.
11. If all source entries are nil, keep the original first entry after collapse, including original first nil.
12. Invalid present source values return `SliceError::InvalidInput` with no partial mutation.
13. Collect all resulting key/value pairs before mutating `self` so later invalid keys do not partially update earlier keys.
14. Return `Ok(0)` after successful processing.
15. Do not add preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Existing M226/M227 float and FloatOrPercent tests continue to pass unchanged.
- Tests prove bool vectors collapse to one entry and write the first non-nil source bool.
- Tests prove nil entries are skipped, `false` is selected when it is the first non-nil value, and scanning stops before later `true` values.
- Tests prove all-nil vectors preserve the original first nil while collapsing to one entry.
- Tests prove numeric/string `1`/`0` bool tokens are accepted and string `true`/`false` tokens are rejected.
- Tests prove invalid bool values and empty handled arrays return `SliceError::InvalidInput` with no mutation.
- Tests prove multi-key no-partial-mutation when an earlier valid bool key is followed by a later invalid bool key.
- Tests prove representative bool option names work from existing registry coverage, such as `filament_long_retractions_when_cut`, `filament_retract_when_changing_layer`, and `filament_wipe`.
- Plan/spec explicitly account for deferred preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
