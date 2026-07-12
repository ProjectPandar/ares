# M231 Spec: DynamicPrintConfig update_values_to_printer_extruders float/percent copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders(...)` `coFloats` and `coPercents` branches into `ares-core` by extending the existing M230 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9567`: full `DynamicPrintConfig::update_values_to_printer_extruders(...)` function context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9400-9462`: existing M230 guard, required printer enum-vector lookup, variant-index preparation, key lookup, and sorted/unique key processing context reused by this milestone.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9491-9503`: `coFloats` branch allocating `extruder_count * stride` and copying `opt->get_at(variant_index[e] * stride + i)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9505-9517`: `coPercents` branch with the same allocation and indexed copy shape.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:663`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-870`: float vector and nullable float vector storage context.
- Representative option-definition context:
  - Floats: `PrintConfig.cpp:2227-2237` (`filament_flow_ratio`), `PrintConfig.cpp:4591-4599` (`fan_max_speed`), `PrintConfig.cpp:4651-4658` (`fan_min_speed`).
  - Percents: `PrintConfig.cpp:737-747` (`elefant_foot_layers_density`) and `PrintConfig.cpp:6839-6845` (`prime_tower_infill_gap`).

## Deferred behavior

- `coFloatsOrPercents`, `coBools`, and `coEnums` branches from `PrintConfig.cpp:9519-9560`.
- `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `PrintConfig.cpp:9569+`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders.rs` only for implementation.
- Modify `crates/ares-core/src/options/tests/update_printer_extruders/` only for tests.
- Create this spec, create `docs/milestones/m231-print-config-update-to-printer-extruders-float-percent.md`, create the matching implementation plan, and append M231 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the existing public API shape `SliceOptions::update_values_to_printer_extruders_string_int_keys(update: PrinterExtruderUpdate<'_>) -> Result<(), SliceError>` for compatibility with M230; extend what kinds it handles rather than adding a parallel API.
2. Preserve the M230 support-different-extruders guard and no-op behavior for unsupported single-extruder printer configs.
3. Preserve the M230 no-op behavior when `printer_config` lacks `extruder_type` or `nozzle_volume_type`.
4. Preserve M230 selected/all-extruder variant-index preparation, including all-extruder fallback to source variant index `0` when lookup is negative and selected-extruder `InvalidInput` when lookup is negative.
5. Iterate a sorted/unique key set.
6. Skip keys with no Ares registry definition.
7. Skip keys missing from `self`.
8. Continue handling `OptionValueKind::Strings` and `OptionValueKind::Ints` exactly as M230 does.
9. Add handling for `OptionValueKind::Floats`, `OptionValueKind::FloatsNullable`, `OptionValueKind::Percent`, `OptionValueKind::Percents`, and `OptionValueKind::PercentsNullable`.
10. For each handled float/percent key, copy entries from source index `variant_index[e] * stride + i` using first-value fallback when the computed source index exceeds the source vector length.
11. Output vector length is `stride` for selected `extruder_id` and `extruder_count * stride` for all-extruder mode.
12. Numeric source entries must be finite JSON numbers.
13. Nullable float/percent vectors may contain string `"nil"`; copied `nil` remains string `"nil"`.
14. Empty handled float/percent vectors return `SliceError::InvalidInput` because `ConfigOptionVector::get_at` requires at least one value to fall back to.
15. Malformed handled float/percent values return `SliceError::InvalidInput`.
16. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
17. Preserve existing helper APIs and exports unchanged.
18. Do not add `FloatOrPercent`, bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, or dependency behavior.

## Acceptance tests

- Tests prove selected `extruder_id` copies float and percent values for exactly that mapped extruder and respects `stride`.
- Tests prove all-extruder mode copies float and percent values in printer order.
- Tests prove computed source indices use first-value fallback for short float/percent source vectors.
- Tests prove nullable string `"nil"` entries are copied and preserved for nullable float/percent vectors.
- Tests prove malformed float/percent values and empty handled vectors return `InvalidInput` with no partial mutation.
- Tests prove unsupported `FloatOrPercent`, bool, and enum keys remain skipped by this milestone.
- Existing M230 string/int tests remain passing.
- Plan/spec explicitly account for deferred `FloatOrPercent`, bool, enum, multiple-filament helper, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
