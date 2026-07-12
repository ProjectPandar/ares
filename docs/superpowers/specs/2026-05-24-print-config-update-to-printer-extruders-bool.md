# M233 Spec: DynamicPrintConfig update_values_to_printer_extruders bool copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders(...)` `coBools` branch into `ares-core` by extending the existing M230-M232 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9567`: full `DynamicPrintConfig::update_values_to_printer_extruders(...)` function context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9400-9462`: existing guard, required printer enum-vector lookup, variant-index preparation, key lookup, and sorted/unique key processing context reused by this milestone.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9534-9547`: `coBools` branch allocating `extruder_count * stride` and copying `opt->get_at(variant_index[e] * stride + i)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:663`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/Config.hpp:1857-1967`: bool vector and nullable bool vector storage/nil semantics context.
- Representative option-definition context from `PrintConfig.cpp:1800-1804` (`activate_air_filtration`), `PrintConfig.cpp:2252-2255` (`enable_pressure_advance`), `PrintConfig.cpp:2334-2338` (`reduce_fan_stop_start_freq`), `PrintConfig.cpp:5062-5066` (`filament_retract_when_changing_layer`), and `PrintConfig.cpp:5081-5086` (`filament_long_retractions_when_cut`).

## Deferred behavior

- `coEnums` branch from `PrintConfig.cpp:9549-9560`.
- `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `PrintConfig.cpp:9569+`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders.rs` and helper modules under `crates/ares-core/src/options/update_printer_extruders/` only for implementation.
- Modify tests under `crates/ares-core/src/options/tests/update_printer_extruders/` only for tests.
- Create this spec, create `docs/milestones/m233-print-config-update-to-printer-extruders-bool.md`, create the matching implementation plan, and append M233 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the existing public API shape `SliceOptions::update_values_to_printer_extruders_string_int_keys(update: PrinterExtruderUpdate<'_>) -> Result<(), SliceError>` for compatibility with M230-M232; extend what kinds it handles rather than adding a parallel API.
2. Preserve the M230 support-different-extruders guard and no-op behavior for unsupported single-extruder printer configs.
3. Preserve the M230 no-op behavior when `printer_config` lacks `extruder_type` or `nozzle_volume_type`.
4. Preserve M230 selected/all-extruder variant-index preparation, including all-extruder fallback to source variant index `0` when lookup is negative and selected-extruder `InvalidInput` when lookup is negative.
5. Iterate a sorted/unique key set.
6. Skip keys with no Ares registry definition.
7. Skip keys missing from `self`.
8. Continue handling previously ported string/int/float/percent/FloatOrPercent kinds exactly as M230-M232 do.
9. Add handling for `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable`.
10. For each handled bool key, copy entries from source index `variant_index[e] * stride + i` using first-value fallback when the computed source index exceeds the source vector length.
11. Output vector length is `stride` for selected `extruder_id` and `extruder_count * stride` for all-extruder mode.
12. Non-nullable bool source entries must be JSON booleans and output as JSON booleans.
13. Nullable bool source entries may be JSON booleans or string `"nil"`; copied `"nil"` remains string `"nil"`.
14. Empty handled bool vectors return `SliceError::InvalidInput` because `ConfigOptionVector::get_at` requires at least one value to fall back to.
15. Malformed bool values, numeric/string `0`/`1`, and non-nullable `"nil"` return `SliceError::InvalidInput`.
16. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
17. Preserve existing helper APIs and exports unchanged.
18. Do not add enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, or dependency behavior.

## Acceptance tests

- Tests prove selected `extruder_id` copies bool values for exactly that mapped extruder and respects `stride`.
- Tests prove all-extruder mode copies bool values in printer order.
- Tests prove computed source indices use first-value fallback for short bool source vectors.
- Tests prove nullable bool string `"nil"` entries are copied and preserved for `BoolsNullable`.
- Tests prove malformed bool values, numeric/string `0`/`1`, non-nullable `"nil"`, and empty handled vectors return `InvalidInput` with no partial mutation.
- Tests prove enum keys remain skipped by this milestone.
- Existing M230-M232 tests remain passing.
- Plan/spec explicitly account for deferred enum, multiple-filament helper, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
