# M238 Spec: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments bool copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` `coBools` branch into `ares-core` by extending the M235-M237 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9633`: guard, `filament_map`, enum-vector prerequisites, variant-index setup, config definition lookup, and key iteration context reused from M235-M237.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9739-9758`: `coBools` branch allocating `filament_count` entries and copying `opt->get_at(variant_index[f])` only when in range.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:664`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:1857-1967`: bool vector, nullable bool nil value, `get_at`, deserialize, and serialize context.
- Representative option-definition context from `PrintConfig.cpp:2252-2255` (`enable_pressure_advance`), `PrintConfig.cpp:2557-2565` (`filament_adaptive_volumetric_speed` nullable conversion context), `PrintConfig.cpp:5062-5066` (`retract_when_changing_layer`), `PrintConfig.cpp:5081-5086` (`long_retractions_when_cut`), and `PrintConfig.cpp:6628-6633` (`wipe`).

## Deferred behavior

- `coEnums` branch from `PrintConfig.cpp:9760-9780`.
- `default` unsupported logging branch from `PrintConfig.cpp:9781-9783` beyond the existing skip behavior.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders/multiple_filament.rs` only as needed, keeping it under 400 LOC.
- Add tests under `crates/ares-core/src/options/tests/update_printer_extruders/` and register them in the local test module.
- Create this spec, create `docs/milestones/m238-print-config-update-to-printer-extruders-multiple-filament-bool.md`, create the matching implementation plan, and append M238 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the M235 public API shape and extend what kinds it handles rather than adding a parallel API.
2. Preserve M235-M237 guard and missing-prerequisite no-op behavior.
3. Preserve M235-M237 filament-map and variant-index resolution behavior.
4. Iterate a sorted/unique key set.
5. Skip keys with no Ares registry definition.
6. Skip keys missing from `self`.
7. Continue handling M235-M237 string, int, float, percent, and FloatOrPercent kinds exactly as before.
8. Add handling for `OptionValueKind::Bools` and `OptionValueKind::BoolsNullable`.
9. For each handled bool key, allocate output length equal to `filament_count` and copy from `variant_index[f]` only when `variant_index[f]` is less than the source vector length.
10. When a variant index is out of range, leave that output slot as JSON `false`, matching the source branch's resized `unsigned char` vector plus skip behavior.
11. Empty handled bool source vectors are allowed and leave every output slot as JSON `false`.
12. Nullable bool source entries may be JSON bools or string `"nil"`; copied `"nil"` remains string `"nil"`.
13. Non-nullable bool source entries must be JSON bools.
14. Malformed bool values and non-nullable `"nil"` return `SliceError::InvalidInput` without partial mutation.
15. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
16. Do not add enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, or independent pipeline behavior.

## Acceptance tests

- Tests prove all-filament bool copy follows the M235 `filament_map` and variant lookup.
- Tests prove out-of-range variant indices and empty handled bool vectors leave JSON `false` output slots.
- Tests prove nullable bool `"nil"` entries are copied and preserved for nullable kinds.
- Tests prove malformed bool values and non-nullable `"nil"` return `InvalidInput` with no partial mutation.
- Tests prove enum keys remain skipped by this milestone.
- Existing M230-M237 tests remain passing.
