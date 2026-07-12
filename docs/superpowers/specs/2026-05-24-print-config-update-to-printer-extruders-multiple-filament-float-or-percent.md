# M237 Spec: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments FloatOrPercent copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` `coFloatsOrPercents` branch into `ares-core` by extending the M235/M236 helper, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9633`: guard, `filament_map`, enum-vector prerequisites, variant-index setup, config definition lookup, and key iteration context reused from M235/M236.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9718-9738`: `coFloatsOrPercents` branch allocating `filament_count` entries and copying `opt->get_at(variant_index[f])` only when in range.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:664`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/Config.hpp:31-42` and `Config.hpp:1318-1450`: `FloatOrPercent` storage, ordering, serialization, percent suffix, and nullable nil context.
- Representative option-definition context from `PrintConfig.cpp:3017-3043` (`infill_anchor`), `PrintConfig.cpp:3045-3066` (`infill_anchor_max`), `PrintConfig.cpp:3104-3112` (`bridge_acceleration`), `PrintConfig.cpp:4016-4026` (`sparse_infill_line_width`), and `PrintConfig.cpp:6936-6947` (`hole_to_polyhole_threshold`).

## Deferred behavior

- `coBools` and `coEnums` branches from `PrintConfig.cpp:9739-9780`.
- `default` unsupported logging branch from `PrintConfig.cpp:9781-9783` beyond the existing skip behavior.
- Nullable FloatOrPercent vectors as a separate handled option kind; Ares currently exposes the copied branch as `OptionValueKind::FloatOrPercent`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders/multiple_filament.rs` only as needed, keeping it under 400 LOC.
- Add tests under `crates/ares-core/src/options/tests/update_printer_extruders/` and register them in the local test module.
- Create this spec, create `docs/milestones/m237-print-config-update-to-printer-extruders-multiple-filament-float-or-percent.md`, create the matching implementation plan, and append M237 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Keep the M235 public API shape and extend what kinds it handles rather than adding a parallel API.
2. Preserve M235/M236 guard and missing-prerequisite no-op behavior.
3. Preserve M235/M236 filament-map and variant-index resolution behavior.
4. Iterate a sorted/unique key set.
5. Skip keys with no Ares registry definition.
6. Skip keys missing from `self`.
7. Continue handling M235/M236 string, int, float, and percent kinds exactly as before.
8. Add handling for `OptionValueKind::FloatOrPercent`.
9. For each handled FloatOrPercent key, allocate output length equal to `filament_count` and copy from `variant_index[f]` only when `variant_index[f]` is less than the source vector length.
10. When a variant index is out of range, leave that output slot as JSON number `0`, matching the source branch's resized vector plus skip behavior.
11. Empty handled FloatOrPercent source vectors are allowed and leave every output slot as JSON number `0`.
12. Source entries may be finite JSON numbers for absolute values or strings parsable as finite FloatOrPercent values, including percent strings like `"20%"`.
13. Copied absolute values are serialized as JSON numbers; copied percent values are serialized as strings with a `%` suffix.
14. Malformed FloatOrPercent values, non-finite numbers, and string `"nil"` return `SliceError::InvalidInput` without partial mutation.
15. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
16. Do not add bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, or independent pipeline behavior.

## Acceptance tests

- Tests prove all-filament FloatOrPercent copy follows the M235 `filament_map` and variant lookup.
- Tests prove out-of-range variant indices and empty handled FloatOrPercent vectors leave numeric `0` output slots.
- Tests prove absolute JSON numbers and percent strings are preserved in copied slots.
- Tests prove malformed strings and `"nil"` return `InvalidInput` with no partial mutation.
- Tests prove bool and enum keys remain skipped by this milestone.
- Existing M230-M236 tests remain passing.
