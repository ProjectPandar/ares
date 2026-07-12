# M232 Spec: DynamicPrintConfig update_values_to_printer_extruders FloatOrPercent copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders(...)` `coFloatsOrPercents` branch into `ares-core` by extending the existing M230/M231 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9567`: full `DynamicPrintConfig::update_values_to_printer_extruders(...)` function context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9400-9462`: existing guard, required printer enum-vector lookup, variant-index preparation, key lookup, and sorted/unique key processing context reused by this milestone.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9519-9532`: `coFloatsOrPercents` branch allocating `extruder_count * stride` and copying `opt->get_at(variant_index[e] * stride + i)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:663`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/Config.hpp:31-42`: `FloatOrPercent` value plus percent-flag data shape.
- `OrcaSlicer/src/libslic3r/Config.hpp:1318-1450`: FloatOrPercent scalar/vector serialization, percent suffix, finite/nil rules, and nullable alias context.
- Representative option-definition context from `PrintConfig.cpp:3017-3043` (`infill_anchor`), `PrintConfig.cpp:3045-3066` (`infill_anchor_max`), `PrintConfig.cpp:3104-3112` (`bridge_acceleration`), `PrintConfig.cpp:4016-4026` (`sparse_infill_line_width`), and `PrintConfig.cpp:6936-6947` (`hole_to_polyhole_threshold`).

## Deferred behavior

- `coBools` and `coEnums` branches from `PrintConfig.cpp:9534-9560`.
- Nullable `ConfigOptionFloatsOrPercentsNullable` as a distinct registry kind; current Ares registry exposes non-nullable `OptionValueKind::FloatOrPercent` for this branch.
- `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `PrintConfig.cpp:9569+`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders.rs` and helper modules under `crates/ares-core/src/options/update_printer_extruders/` only for implementation.
- Modify tests under `crates/ares-core/src/options/tests/update_printer_extruders/` only for tests.
- Create this spec, create `docs/milestones/m232-print-config-update-to-printer-extruders-float-or-percent.md`, create the matching implementation plan, and append M232 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the existing public API shape `SliceOptions::update_values_to_printer_extruders_string_int_keys(update: PrinterExtruderUpdate<'_>) -> Result<(), SliceError>` for compatibility with M230/M231; extend what kinds it handles rather than adding a parallel API.
2. Preserve the M230 support-different-extruders guard and no-op behavior for unsupported single-extruder printer configs.
3. Preserve the M230 no-op behavior when `printer_config` lacks `extruder_type` or `nozzle_volume_type`.
4. Preserve M230 selected/all-extruder variant-index preparation, including all-extruder fallback to source variant index `0` when lookup is negative and selected-extruder `InvalidInput` when lookup is negative.
5. Iterate a sorted/unique key set.
6. Skip keys with no Ares registry definition.
7. Skip keys missing from `self`.
8. Continue handling `Strings`, `Ints`, `Floats`, `FloatsNullable`, `Percent`, `Percents`, and `PercentsNullable` exactly as M230/M231 do.
9. Add handling for `OptionValueKind::FloatOrPercent`.
10. For each handled FloatOrPercent key, copy entries from source index `variant_index[e] * stride + i` using first-value fallback when the computed source index exceeds the source vector length.
11. Output vector length is `stride` for selected `extruder_id` and `extruder_count * stride` for all-extruder mode.
12. Absolute FloatOrPercent source entries may be finite JSON numbers or finite numeric strings without `%`; output absolute entries as JSON numbers.
13. Percent FloatOrPercent source entries are strings with a trailing `%`; output percent entries as strings with a trailing `%`.
14. Empty handled FloatOrPercent vectors return `SliceError::InvalidInput` because `ConfigOptionVector::get_at` requires at least one value to fall back to.
15. Malformed FloatOrPercent values, string `"nil"`, and non-finite values return `SliceError::InvalidInput`.
16. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
17. Preserve existing helper APIs and exports unchanged.
18. Do not add bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, or dependency behavior.

## Acceptance tests

- Tests prove selected `extruder_id` copies absolute and percent FloatOrPercent values for exactly that mapped extruder and respects `stride`.
- Tests prove all-extruder mode copies FloatOrPercent values in printer order.
- Tests prove computed source indices use first-value fallback for short FloatOrPercent source vectors.
- Tests prove numeric strings without `%` are accepted and serialized as JSON numbers while percent strings retain `%`.
- Tests prove malformed FloatOrPercent values, `"nil"`, and empty handled vectors return `InvalidInput` with no partial mutation.
- Tests prove bool and enum keys remain skipped by this milestone.
- Existing M230/M231 string/int/float/percent tests remain passing.
- Plan/spec explicitly account for deferred bool, enum, multiple-filament helper, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
