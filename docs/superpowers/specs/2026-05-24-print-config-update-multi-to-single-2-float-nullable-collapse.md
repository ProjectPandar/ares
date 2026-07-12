# M226 Spec: DynamicPrintConfig update_values_from_multi_to_single_2 float nullable collapse

## Goal

Port the first `coFloats` nullable float branch of OrcaSlicer's commented `DynamicPrintConfig::update_values_from_multi_to_single_2(...)` helper into `ares-core`, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9321`: commented `DynamicPrintConfig::update_values_from_multi_to_single_2(...)`, limited to setup and the first `coFloats` branch.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:673-674`: commented declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9290-9304`: config-definition guard, source-key iteration, key-set filtering, option-definition lookup, and unknown-definition skip behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9306-9321`: `coFloats` branch using `ConfigOptionFloatsNullable`, `9999.0` sentinel, nil skip, strict `<` minimum selection, erase from index `1` to end, and optional index `0` overwrite.
- `OrcaSlicer/src/libslic3r/Config.hpp:837-838` and `Config.hpp:952`: nullable float nil is `NaN` and `is_nil(idx)` checks `std::isnan(value)`.

## Deferred behavior

- `coFloatsOrPercents` branch from `PrintConfig.cpp:9322-9338`.
- `coBools` branch from `PrintConfig.cpp:9339-9355`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Add a new focused module under `crates/ares-core/src/options/update_multi_to_multi/` rather than growing near-limit files.
- Export a small parameter struct if needed and add `SliceOptions::update_values_from_multi_to_single_2_float_keys(...) -> Result<isize, SliceError>`.
- Add focused tests under `crates/ares-core/src/options/tests/`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add public mutating API `SliceOptions::update_values_from_multi_to_single_2_float_keys(key_set: &[&str]) -> Result<isize, SliceError>` or an equivalently small parameter wrapper.
2. Iterate keys already present in `self.values()` in sorted map order and skip keys absent from `key_set`.
3. Skip any key with no Ares registry definition.
4. Skip every unsupported kind other than `Floats` and `FloatsNullable`.
5. Validate handled values as nullable float vectors where JSON numbers are finite values and JSON string `"nil"` is nil.
6. Empty handled vectors return `SliceError::InvalidInput` with no mutation because the upstream branch keeps and optionally overwrites index `0`.
7. For each handled key, scan every source index in source order, ignore nil entries, initialize the candidate minimum to `9999.0`, and replace it only when a non-nil value is strictly less than the candidate.
8. Collapse each handled vector to one output element.
9. If a selected value exists, write it at index `0`; otherwise keep the original first entry after collapse.
10. Values equal to or greater than the initial `9999.0` sentinel do not count as selected and therefore leave the original first entry after collapse.
11. Invalid present source values return `SliceError::InvalidInput` with no partial mutation.
12. Collect all resulting key/value pairs before mutating `self` so later invalid keys do not partially update earlier keys.
13. Return `Ok(0)` after successful processing.
14. Do not add FloatOrPercent, bool, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove supported float keys collapse to one entry and write the minimum non-nil value below `9999.0`.
- Tests prove nil entries are skipped and all-nil vectors preserve the original first nil while collapsing to one entry.
- Tests prove values equal to or greater than `9999.0` do not replace the original first entry.
- Tests prove single-entry vectors remain single-entry and are overwritten only when that value is selected by the strict sentinel rule.
- Tests prove unsupported kinds and absent keys are skipped without mutation.
- Tests prove invalid float values and empty handled arrays return `SliceError::InvalidInput` with no mutation.
- Tests prove multi-key no-partial-mutation when an earlier valid float key is followed by a later invalid float key.
- Tests prove representative float option names work from existing registry coverage, such as `filament_flow_ratio`, `filament_retraction_length`, and `fan_max_speed`.
- Plan/spec explicitly account for deferred FloatOrPercent, bool, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
