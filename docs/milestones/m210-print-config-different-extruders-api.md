# M210: DynamicPrintConfig different extruders API

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8716-8742`, with `PrintConfig.hpp:660` declaration context, `Config.hpp:624-630` vector `get_at` fallback semantics, `PrintConfig.hpp:412-421` / `PrintConfig.cpp:565-575` extruder/nozzle-volume enum serialization context, `PrintConfig.cpp:5202-5225` / `PrintConfig.hpp:1408-1409` option-definition and field-type context, and existing `nozzle_diameter` option context already ported in the registry. It adds only a read-only `SliceOptions::is_using_different_extruders()` helper. It does not port `support_different_extruders`, `get_index_for_extruder`, variant lookup, preset bundle materialization, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit criteria

- Missing `nozzle_diameter` returns `false`.
- One nozzle diameter returns `false`.
- Multiple nozzle diameters with missing `extruder_type` or missing `nozzle_volume_type` return `false`.
- Multiple nozzle diameters with identical source `extruder_type` and `nozzle_volume_type` values return `false`.
- Any later extruder with a different `extruder_type` returns `true`.
- Any later extruder with a different `nozzle_volume_type` returns `true`.
- Enum vector `get_at` fallback matches source behavior: out-of-range enum arrays reuse the first value.
- Invalid non-vector/non-string enum boundary values and invalid nozzle diameter values return `SliceError::InvalidInput` instead of panicking.
- No plural filament/profile composition, variant lookup, slicing, extrusion, G-code, crate, or dependency changes.
