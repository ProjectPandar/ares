# M234 Spec: DynamicPrintConfig update_values_to_printer_extruders enum copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders(...)` `coEnums` branch into `ares-core` by extending the existing M230-M233 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9567`: full `DynamicPrintConfig::update_values_to_printer_extruders(...)` function context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9400-9462`: existing guard, required printer enum-vector lookup, variant-index preparation, key lookup, and sorted/unique key processing context reused by this milestone.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9549-9560`: `coEnums` branch allocating `extruder_count * stride` and copying `opt->get_at(variant_index[e] * stride + i)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:663`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/Config.hpp:2101-2201`: generic enum vector and nullable generic enum vector storage, serialization, and nil semantics context.
- Representative option-definition context from `PrintConfig.cpp:5149-5162` (`z_hop_types`), `PrintConfig.cpp:5187-5200` (`retract_lift_enforce`), `PrintConfig.cpp:5215-5225` (`nozzle_volume_type`), and `CommonDefs.hpp:12-20` plus `PrintConfig.cpp:3652-3669` (`nozzle_type`).

## Deferred behavior

- `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `PrintConfig.cpp:9569+`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders.rs` and helper modules under `crates/ares-core/src/options/update_printer_extruders/` only for implementation.
- Modify tests under `crates/ares-core/src/options/tests/update_printer_extruders/` only for tests.
- Create this spec, create `docs/milestones/m234-print-config-update-to-printer-extruders-enum.md`, create the matching implementation plan, and append M234 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the existing public API shape `SliceOptions::update_values_to_printer_extruders_string_int_keys(update: PrinterExtruderUpdate<'_>) -> Result<(), SliceError>` for compatibility with M230-M233; extend what kinds it handles rather than adding a parallel API.
2. Preserve the M230 support-different-extruders guard and no-op behavior for unsupported single-extruder printer configs.
3. Preserve the M230 no-op behavior when `printer_config` lacks `extruder_type` or `nozzle_volume_type`.
4. Preserve M230 selected/all-extruder variant-index preparation, including all-extruder fallback to source variant index `0` when lookup is negative and selected-extruder `InvalidInput` when lookup is negative.
5. Iterate a sorted/unique key set.
6. Skip keys with no Ares registry definition.
7. Skip keys missing from `self`.
8. Continue handling previously ported string/int/float/percent/FloatOrPercent/bool kinds exactly as M230-M233 do.
9. Add handling for `OptionValueKind::Enums` and `OptionValueKind::EnumsNullable`.
10. For each handled enum key, copy entries from source index `variant_index[e] * stride + i` using first-value fallback when the computed source index exceeds the source vector length.
11. Output vector length is `stride` for selected `extruder_id` and `extruder_count * stride` for all-extruder mode.
12. Non-nullable enum source entries must be JSON strings and output as JSON strings.
13. Nullable enum source entries may be JSON strings, including `"nil"`; copied `"nil"` remains string `"nil"`.
14. Empty handled enum vectors return `SliceError::InvalidInput` because `ConfigOptionVector::get_at` requires at least one value to fall back to.
15. Non-string enum values and non-nullable `"nil"` return `SliceError::InvalidInput`.
16. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
17. Preserve existing helper APIs and exports unchanged.
18. Do not add multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, or dependency behavior.

## Acceptance tests

- Tests prove selected `extruder_id` copies enum values for exactly that mapped extruder and respects `stride`.
- Tests prove all-extruder mode copies enum values in printer order.
- Tests prove computed source indices use first-value fallback for short enum source vectors.
- Tests prove nullable enum string `"nil"` entries are copied and preserved for `EnumsNullable`.
- Tests prove malformed enum values, non-nullable `"nil"`, and empty handled vectors return `InvalidInput` with no partial mutation.
- Existing M230-M233 tests remain passing.
- Plan/spec explicitly account for deferred multiple-filament helper, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
