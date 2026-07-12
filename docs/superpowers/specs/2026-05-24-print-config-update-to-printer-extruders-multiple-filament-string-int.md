# M235 Spec: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments string/int copy

## Goal

Port OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` setup plus `coStrings`/`coInts` branches into `ares-core`, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9675`: guard, `filament_map`, `extruder_type`, `nozzle_volume_type`, per-filament variant-index preparation, and `coStrings`/`coInts` copy branches.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:664`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2401-2405`: `filament_map` option definition context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5292-5304`: `filament_extruder_variant` and `filament_self_index` option definition context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8744-8818`: existing `get_index_for_extruder` behavior used by the source function.

## Deferred behavior

- `coFloats`, `coPercents`, `coFloatsOrPercents`, `coBools`, and `coEnums` branches from `PrintConfig.cpp:9676-9810`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Modify `crates/ares-core/src/options/update_printer_extruders.rs` and focused helper modules under `crates/ares-core/src/options/update_printer_extruders/` only for implementation.
- Modify tests under `crates/ares-core/src/options/tests/update_printer_extruders/` only for tests.
- Create this spec, create `docs/milestones/m235-print-config-update-to-printer-extruders-multiple-filament-string-int.md`, create the matching implementation plan, and append M235 to `docs/roadmap.md`.
- Do not create new crates or dependencies.

## Functional requirements

1. Add a public `SliceOptions` helper for the M235 source slice while preserving existing M230-M234 API compatibility.
2. Accept the same source concepts as Orca: `printer_config`, sorted/unique key set, `id_name`, and `variant_name`.
3. Return `Ok(())` without mutation when `printer_config.support_different_extruders()` reports `extruder_count <= 1` and `supported == false`.
4. Return `Ok(())` without mutation when `printer_config` lacks `filament_map`, `extruder_type`, or `nozzle_volume_type`.
5. Parse `filament_map` as a non-empty integer vector; malformed present `filament_map` returns `SliceError::InvalidInput` without mutation.
6. For each filament index `f`, compute the mapped extruder index as `filament_map[f] - 1`; invalid zero/negative mapped indices or integer overflow return `InvalidInput` without mutation.
7. Read mapped `extruder_type` and `nozzle_volume_type` with first-value fallback, matching Orca vector `get_at` behavior.
8. Resolve `variant_index[f]` with existing Ares extruder-index lookup using filament id `f + 1`, `id_name`, mapped extruder type, mapped nozzle volume type, `variant_name`, and stride `1`.
9. If lookup returns negative, set the variant index to `0`, then if `id_name` exists, scan `id_name` for value `f + 1` and use that index when found.
10. Iterate a sorted/unique key set.
11. Skip keys with no Ares registry definition.
12. Skip keys missing from `self`.
13. Handle only `OptionValueKind::Strings` and `OptionValueKind::Ints`.
14. For each handled string/int key, allocate output length equal to `filament_count` and copy from `variant_index[f]` only when `variant_index[f]` is less than the source vector length.
15. When a variant index is out of range, leave that output slot at the type default (`""` for strings, `0` for ints), matching the source branch's resized vector plus skip behavior.
16. Empty handled source vectors are allowed and leave every output slot at the type default, matching the source branch's resized vector plus skip behavior.
17. Malformed handled source vectors return `InvalidInput` without partial mutation.
18. Collect all key updates before mutating `self` so later invalid handled keys do not partially update earlier keys.
19. Do not add float, percent, FloatOrPercent, bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, or independent pipeline behavior.

## Acceptance tests

- Tests prove no-op behavior for non-different single-extruder printer configs.
- Tests prove missing `filament_map`, `extruder_type`, or `nozzle_volume_type` skips without mutation when the guard passes.
- Tests prove all-filament string/int copy follows `filament_map` and per-filament variant lookup.
- Tests prove negative lookup falls back to matching `id_name == f + 1` before falling back to zero.
- Tests prove out-of-range variant indices and empty handled source vectors leave type default output slots.
- Tests prove malformed present `filament_map`, handled string vectors, and handled int vectors return `InvalidInput` with no partial mutation.
- Tests prove unsupported non-string/int keys are skipped.
- Existing M230-M234 tests remain passing.
- Plan/spec explicitly account for deferred float, percent, FloatOrPercent, bool, enum, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
