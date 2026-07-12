# M230 Spec: DynamicPrintConfig update_values_to_printer_extruders string/int copy

## Goal

Port the first source slice of OrcaSlicer's `DynamicPrintConfig::update_values_to_printer_extruders(...)` into `ares-core`: the guard, variant-index preparation, and `coStrings` / `coInts` copy branches, without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9567`: full `DynamicPrintConfig::update_values_to_printer_extruders(...)` function context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9400-9403`: `support_different_extruders(extruder_count)` guard and no-op condition.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9408-9413`: required `printer_config` `extruder_type` and `nozzle_volume_type` lookup and skip behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9414-9448`: variant-index preparation for a valid requested `extruder_id` or all extruders, including all-extruder fallback to `0` when `get_index_for_extruder` returns negative.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9450-9462`: config-definition/key lookup and unknown-key skip behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9463-9489`: `coStrings` and `coInts` branches: allocate `extruder_count * stride` and copy `opt->get_at(variant_index[e] * stride + i)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:663`: declaration context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8744-8818`: prerequisite `support_different_extruders(...)` and `get_index_for_extruder(...)` behavior already ported in earlier milestones.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: vector `get_at` first-value fallback semantics.
- Representative option-definition context from `PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304` for string/int variant vectors.

## Deferred behavior

- `coFloats`, `coPercents`, `coFloatsOrPercents`, `coBools`, and `coEnums` branches from `PrintConfig.cpp:9491-9560`.
- `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `PrintConfig.cpp:9569+`.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Add `crates/ares-core/src/options/update_printer_extruders.rs` with:
  - `PrinterExtruderUpdate<'a>` input struct.
  - `SliceOptions::update_values_to_printer_extruders_string_int_keys(update: PrinterExtruderUpdate<'_>) -> Result<(), SliceError>`.
- Register the module and export `PrinterExtruderUpdate` from `crates/ares-core/src/options.rs`.
- Add `crates/ares-core/src/options/tests/update_printer_extruders.rs` and register it in `crates/ares-core/src/options/tests.rs`.
- Do not create new crates or dependencies.

## Functional requirements

1. The new API is mutating on `self`, reads `printer_config`, `key_set`, `id_name`, `variant_name`, `stride`, and `extruder_id` from `PrinterExtruderUpdate`.
2. If `printer_config.support_different_extruders()` returns `supported == false` and `extruder_count <= 1`, return `Ok(())` with no mutation.
3. If `printer_config` lacks `extruder_type` or `nozzle_volume_type`, return `Ok(())` with no mutation.
4. If required present `extruder_type` or `nozzle_volume_type` values are malformed or contain unknown enum values used by the selected extruder(s), return `SliceError::InvalidInput` with no partial mutation.
5. If `extruder_id` is in the inclusive range `1..=extruder_count`, prepare exactly one variant index for that selected printer extruder and output vectors sized `stride`.
6. If `extruder_id` is `0` or greater than `extruder_count`, prepare one variant index per printer extruder and output vectors sized `extruder_count * stride`.
7. Variant indices are resolved against `self` using the previously ported `get_index_for_extruder` semantics for `id_name`, `variant_name`, and `stride`.
8. For the all-extruder branch only, any negative variant index is replaced with `0`, matching Orca's transient UI-state fallback.
9. For the selected-extruder branch, a negative variant index returns `SliceError::InvalidInput` with no mutation instead of asserting.
10. Iterate a sorted/unique key set.
11. Skip keys with no Ares registry definition.
12. Skip keys missing from `self`.
13. Only handle `OptionValueKind::Strings` and `OptionValueKind::Ints` in M230; skip all other kinds.
14. For each handled key, copy entries from source index `variant_index[e] * stride + i` using source vector first-value fallback when the computed source index exceeds the source vector length.
15. Empty handled source vectors return `SliceError::InvalidInput` because `ConfigOptionVector::get_at` requires at least one value to fall back to.
16. Invalid handled source values return `SliceError::InvalidInput`.
17. Integer values must fit `i32`; out-of-range integers return `SliceError::InvalidInput`.
18. Collect all key updates before mutating `self` so later invalid keys do not partially update earlier keys.
19. Preserve existing update helper APIs unchanged.
20. No float, percent, bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, or dependency behavior is added.

## Acceptance tests

- Tests prove the guard returns no mutation when the printer config has one non-different extruder.
- Tests prove absent `extruder_type` or `nozzle_volume_type` skips without mutation when the guard passes.
- Tests prove selected `extruder_id` copies string and int values for exactly that mapped extruder and respects `stride`.
- Tests prove all-extruder mode copies values for each printer extruder in order.
- Tests prove all-extruder negative variant lookup falls back to source variant index `0`.
- Tests prove computed source indices use first-value fallback when they exceed source vector length.
- Tests prove unknown keys, missing source keys, and unsupported kinds are skipped.
- Tests prove malformed handled values, empty handled vectors, invalid enums, and negative selected-extruder lookup return `InvalidInput` with no partial mutation.
- Plan/spec explicitly account for deferred float/percent/bool/enum branches, multiple-filament helper, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
