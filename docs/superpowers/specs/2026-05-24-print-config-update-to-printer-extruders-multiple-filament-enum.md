# M239 Spec: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments enum copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` `coEnums` branch into `ares-core` by extending the M235-M238 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9633`: guard, `filament_map`, enum-vector prerequisites, variant-index setup, config definition lookup, and key iteration context reused from M235-M238.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9760-9780`: `coEnums` branch allocating `filament_count` entries and copying `opt->get_at(variant_index[f])` only when in range.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:664`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:2101-2201`: generic enum vector, nullable enum nil value, deserialize, and serialize context.
- Representative option-definition context from `PrintConfig.cpp:5149-5162` (`z_hop_types`), `PrintConfig.cpp:5187-5200` (`retract_lift_enforce`), `PrintConfig.cpp:5202-5213` (`extruder_type`), `PrintConfig.cpp:5215-5225` (`nozzle_volume_type`), and `PrintConfig.cpp:3652-3669` (`nozzle_type`).

## Deferred behavior

- `default` unsupported logging branch from `PrintConfig.cpp:9781-9783` beyond the existing skip behavior.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders/multiple_filament.rs` only as needed, keeping it under 400 LOC.
- Add tests under `crates/ares-core/src/options/tests/update_printer_extruders/` and register them in the local test module.
- Create this spec, create `docs/milestones/m239-print-config-update-to-printer-extruders-multiple-filament-enum.md`, create the matching implementation plan, and append M239 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the M235 public API shape and extend what kinds it handles rather than adding a parallel API.
2. Preserve M235-M238 guard and missing-prerequisite no-op behavior.
3. Preserve M235-M238 filament-map and variant-index resolution behavior.
4. Iterate a sorted/unique key set.
5. Skip keys with no Ares registry definition.
6. Skip keys missing from `self`.
7. Continue handling M235-M238 string, int, float, percent, FloatOrPercent, and bool kinds exactly as before.
8. Add handling for `OptionValueKind::Enums` and `OptionValueKind::EnumsNullable`.
9. For each handled enum key, allocate output length equal to `filament_count` and copy from `variant_index[f]` only when `variant_index[f]` is less than the source vector length.
10. When a variant index is out of range, leave that output slot as an empty string, matching the source branch's resized `int` vector plus Ares's enum-string representation.
11. Empty handled enum source vectors are allowed and leave every output slot as an empty string.
12. Nullable enum source entries may be strings or string `"nil"`; copied `"nil"` remains string `"nil"`.
13. Non-nullable enum source entries must be strings other than `"nil"`.
14. Malformed enum values and non-nullable `"nil"` return `SliceError::InvalidInput` without partial mutation.
15. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
16. Do not add default unsupported logging, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, or independent pipeline behavior.

## Acceptance tests

- Tests prove all-filament enum copy follows the M235 `filament_map` and variant lookup.
- Tests prove out-of-range variant indices and empty handled enum vectors leave empty-string output slots.
- Tests prove nullable enum `"nil"` entries are copied and preserved for nullable kinds.
- Tests prove malformed enum values and non-nullable `"nil"` return `InvalidInput` with no partial mutation.
- Tests prove unknown/missing/default unsupported behavior remains existing skip behavior.
- Existing M230-M238 tests remain passing.
