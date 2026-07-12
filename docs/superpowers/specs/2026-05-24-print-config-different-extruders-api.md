# M210 Spec: DynamicPrintConfig different extruders API

## Goal

Port OrcaSlicer's read-only `DynamicPrintConfig::is_using_different_extruders()` helper into `ares-core` without designing an Ares-owned pipeline.

## Upstream source boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8716-8742`: `DynamicPrintConfig::is_using_different_extruders()` branch logic.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:660`: declaration context.
- `OrcaSlicer/src/libslic3r/Config.hpp:624-630`: `ConfigOptionVector<T>::get_at(i)` returns `values[i]` or `values.front()` when `i` is out of range.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:412-421`: `ExtruderType` and `NozzleVolumeType` enum discriminants.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:565-575`: enum string maps for `ExtruderType` and `NozzleVolumeType`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5202-5225` / `PrintConfig.hpp:1408-1409`: `extruder_type` and `nozzle_volume_type` option-definition and field-type context.
- Existing Ares registry context for `nozzle_diameter` and already-ported Orca numeric-vector parsing is used only to obtain the source nozzle vector length.

## Deferred behavior

- `DynamicPrintConfig::support_different_extruders(int&)` from `PrintConfig.cpp:8744-8766`.
- `DynamicPrintConfig::get_index_for_extruder(...)` from `PrintConfig.cpp:8768+`.
- Variant-list splitting/lookup and generated variant IDs.
- Preset/profile loading or materialization.
- UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.
- New crates or dependencies.

## Destination boundary

- Create `crates/ares-core/src/options/different_extruders.rs` with `SliceOptions::is_using_different_extruders(&self) -> Result<bool, SliceError>` and private enum-vector helpers.
- Modify `crates/ares-core/src/options.rs` to register `mod different_extruders;`.
- Create `crates/ares-core/src/options/tests/different_extruders.rs`.
- Modify `crates/ares-core/src/options/tests.rs` to register `mod different_extruders;`.

## Functional requirements

1. Add public read-only API `SliceOptions::is_using_different_extruders(&self) -> Result<bool, SliceError>`.
2. If `nozzle_diameter` is absent, use existing Ares default single-nozzle behavior and return `false`.
3. If `nozzle_diameter` resolves to one or zero entries, return `false`.
4. If there are multiple nozzle diameters but `extruder_type` is absent, return `false`.
5. If there are multiple nozzle diameters but `nozzle_volume_type` is absent, return `false`.
6. If both enum vectors are present, compare each later extruder index against index `0`.
7. Return `true` when any later `extruder_type` differs from index `0`.
8. Return `true` when any later `nozzle_volume_type` differs from index `0`.
9. Return `false` when all compared extruder type and nozzle volume type pairs match.
10. Enum vector access must match source `get_at`: if the requested index is out of range for a non-empty vector, use the first vector value.
11. For public boundary safety, a present `extruder_type` or `nozzle_volume_type` value required by this API must be a non-empty string array containing only valid source enum strings: `Direct Drive`, `Bowden`, `Standard`, `High Flow` as appropriate. Invalid shapes or unknown values return `SliceError::InvalidInput`.
12. Invalid `nozzle_diameter` values return the existing `SliceError::InvalidInput` from `SliceOptions::nozzle_diameters()`.
13. Do not add support-different-extruders, variant-index lookup, preset/model loading, slicing, extrusion, G-code behavior, new crates, or dependencies.

## Acceptance tests

- Tests prove absent and single `nozzle_diameter` return `false`.
- Tests prove multiple nozzle diameters with missing `extruder_type` or `nozzle_volume_type` return `false`.
- Tests prove matching `extruder_type` and `nozzle_volume_type` arrays return `false`.
- Tests prove a later `extruder_type` mismatch returns `true`.
- Tests prove a later `nozzle_volume_type` mismatch returns `true`.
- Tests prove enum vector out-of-range source fallback reuses the first enum value.
- Tests prove malformed enum vectors, unknown enum strings, empty enum vectors, and invalid nozzle diameter values return `SliceError::InvalidInput`.
- Plan/spec explicitly account for deferred `support_different_extruders`, `get_index_for_extruder`, variant lookup, preset materialization, UI runtime, slicing, extrusion, and G-code behavior.
