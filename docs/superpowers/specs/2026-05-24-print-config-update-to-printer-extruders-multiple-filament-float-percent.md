# M236 Spec: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments float/percent copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` `coFloats` and `coPercents` branches into `ares-core` by extending the M235 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9633`: guard, `filament_map`, enum-vector prerequisites, variant-index setup, config definition lookup, and key iteration context reused from M235.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9676-9717`: `coFloats` and `coPercents` branches allocating `filament_count` entries and copying `opt->get_at(variant_index[f])` only when in range.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:664`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/Config.hpp:812-1091 and Config.hpp:1204-1257`: float, nullable float, integer/percent vector storage and nil serialization context.
- Representative option-definition context from `PrintConfig.cpp:2462-2470` (`filament_max_volumetric_speed`), `PrintConfig.cpp:5055-5060` (`filament_retract_before_wipe`), and `PrintConfig.cpp:5068-5075` (`filament_retraction_length`).

## Deferred behavior

- `coFloatsOrPercents`, `coBools`, and `coEnums` branches from `PrintConfig.cpp:9718-9787`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders/multiple_filament.rs` and related exports only as needed.
- Modify tests under `crates/ares-core/src/options/tests/update_printer_extruders/` only for tests.
- Create this spec, create `docs/milestones/m236-print-config-update-to-printer-extruders-multiple-filament-float-percent.md`, create the matching implementation plan, and append M236 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the M235 public API shape and extend what kinds it handles rather than adding a parallel API.
2. Preserve M235 guard and missing-prerequisite no-op behavior.
3. Preserve M235 filament-map and variant-index resolution behavior.
4. Iterate a sorted/unique key set.
5. Skip keys with no Ares registry definition.
6. Skip keys missing from `self`.
7. Continue handling M235 string/int kinds exactly as before.
8. Add handling for `OptionValueKind::Floats`, `OptionValueKind::FloatsNullable`, `OptionValueKind::Percent`, `OptionValueKind::Percents`, and `OptionValueKind::PercentsNullable`.
9. For each handled numeric key, allocate output length equal to `filament_count` and copy from `variant_index[f]` only when `variant_index[f]` is less than the source vector length.
10. When a variant index is out of range, leave that output slot as JSON number `0`, matching the source branch's resized vector plus skip behavior.
11. Empty handled numeric source vectors are allowed and leave every output slot as JSON number `0`.
12. Nullable numeric source entries may be JSON numbers or string `"nil"`; copied `"nil"` remains string `"nil"`.
13. Non-nullable numeric source entries must be finite JSON numbers.
14. Malformed numeric values, non-finite numbers, and non-nullable `"nil"` return `SliceError::InvalidInput` without partial mutation.
15. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
16. Do not add FloatOrPercent, bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, or independent pipeline behavior.

## Acceptance tests

- Tests prove all-filament float/percent copy follows the M235 `filament_map` and variant lookup.
- Tests prove out-of-range variant indices and empty handled numeric vectors leave numeric `0` output slots.
- Tests prove nullable numeric `"nil"` entries are copied and preserved for nullable kinds.
- Tests prove malformed numeric values and non-nullable `"nil"` return `InvalidInput` with no partial mutation.
- Tests prove FloatOrPercent, bool, and enum keys remain skipped by this milestone.
- Existing M230-M235 tests remain passing.
