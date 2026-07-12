# Consume Support Interface And Ironing Role Fan Design

## Scope

Consume the existing OrcaSlicer cooling options `support_material_interface_fan_speed` and `ironing_fan_speed` into Ares role-based fan G-code behavior. This is a source-cited `libslic3r` rewrite slice for G-code role fan control, not new option metadata and not support or ironing path generation.

Upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1628-1630` declares `support_material_interface_fan_speed`, `internal_bridge_fan_speed`, and `ironing_fan_speed` as integer vector print-config options.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3337-3347` defines `support_material_interface_fan_speed` with default `-1`, range `-1..=100`, and first-layer fan disable behavior.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3361-3370` defines `ironing_fan_speed` with default `-1` and range `-1..=100`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6843-6851` marks support-interface and ironing fan regions when the configured fan speed is non-negative and the extrusion path role is `erSupportMaterialInterface` or `erIroning`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:6918` and `OrcaSlicer/src/libslic3r/GCode.cpp:7079` apply those role-based fan markers before emitting extrusion.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:747-806` derives support-interface and ironing role fan controls from the current extruder, disables them before `close_fan_the_first_x_layers`, and keeps `-1` as disabled.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:989-1000` turns fan markers into part-cooling fan commands using the configured role speeds.

Ares destination boundary:

- `crates/ares-core/src/print_paths.rs` for rendering-neutral print-path roles.
- `crates/ares-core/src/extrusion_entity.rs` for mapping print-path roles to Orca-style extrusion roles.
- `crates/ares-core/src/extrusions/options.rs`, `crates/ares-core/src/speeds/config/accessors.rs`, and `crates/ares-core/src/speeds/slow_down_layers.rs` for the minimal exhaustive width, flow, and speed mappings required after adding the two new print-path roles.
- `crates/ares-core/src/options/part_cooling_fan.rs` and a small split module under `crates/ares-core/src/options/part_cooling_fan/` for parsing and carrying role fan speeds without exceeding the 400 LOC file limit.
- Focused pipeline tests under `crates/ares-core/src/pipeline/tests/`.

## Current Behavior

Ares already parses and consumes `overhang_fan_speed`, `overhang_fan_threshold`, and `internal_bridge_fan_speed` through `RoleFanControl` and `LayerRoleFanControl`.

The current role fan logic can emit and restore `M106` commands around `Bridge`, `InternalBridge`, `OverhangPerimeter`, and threshold-enabled `ExternalPerimeter` print paths. It does not yet consume `support_material_interface_fan_speed` or `ironing_fan_speed`, and `PrintPathRole` has no support-interface or ironing path role even though `ExtrusionRole` already models `SupportMaterialInterface` and `Ironing`.

## Required Behavior

- Add `PrintPathRole::SupportMaterialInterface` and `PrintPathRole::Ironing` as rendering-neutral print path roles.
- Map those print-path roles to `ExtrusionRole::SupportMaterialInterface` and `ExtrusionRole::Ironing`.
- Parse `support_material_interface_fan_speed` and `ironing_fan_speed` from `SliceOptions` as Orca integer-vector fan speed options:
  - absent option defaults to `-1`;
  - the first vector value is used;
  - `-1` disables the role fan override;
  - `0..=100` enables the role fan override at that percent;
  - empty, non-numeric, fractional, non-finite, below `-1`, or above `100` values return `SliceError::InvalidInput` naming the offending key.
- Extend `RoleFanControl::for_layer` so support-interface and ironing overrides are:
  - suppressed before `close_fan_the_first_x_layers`;
  - not ramp-scaled by `full_fan_speed_layer`, matching Orca's direct configured role speed for these two roles;
  - independent of `enable_overhang_bridge_fan`, because Orca's support-interface and ironing marker gate is keyed to their own option values, not the overhang bridge fan option.
- Extend `LayerRoleFanControl::speed_for_role` so support-interface and ironing print moves emit the configured fan speed before the role move and restore the layer baseline, or turn the fan off when no baseline exists, after leaving the role.
- Preserve existing behavior for bridge, internal bridge, overhang, fan kickstart, fan speedup, close-first-layers, min PWM, and layer-time baseline fan control.

## Deferred Behavior

- Generating real support interface paths, ironing paths, support body paths, or support ironing geometry.
- Full Orca `CoolingBuffer` marker scheduling and rewritten comment marker output. Ares continues using its direct role-fan state machine around `PrintPathRole`.
- Multi-extruder indexing beyond Ares's existing first-value parsing pattern.
- Any UI, filesystem, terminal, OpenGL, or platform-specific behavior in `ares-core`.
- Changing movement feedrates, extrusion amounts, path ordering, or fan behavior for unrelated roles.

## Acceptance Criteria

- Focused RED/GREEN tests prove `support_material_interface_fan_speed = 65` emits a support-interface fan override before a `SupportMaterialInterface` print path and restores the prior layer baseline afterward.
- Focused RED/GREEN tests prove `ironing_fan_speed = 15` emits an ironing fan override before an `Ironing` print path and turns the fan off when no layer baseline exists.
- Tests prove both defaults `-1` emit no support-interface or ironing fan override.
- Tests prove `close_fan_the_first_x_layers` suppresses support-interface and ironing overrides on closed layers.
- Tests prove invalid role fan values are rejected with the corresponding option key.
- Focused nextest passes:
  `cargo nextest run -p ares-core support_ironing_role_fan_gcode`
- Adjacent role fan tests pass:
  `cargo nextest run -p ares-core internal_bridge_fan_gcode overhang_bridge_fan_gcode fan_kickstart`
- Full verification passes:
  `cargo fmt --check`
  `cargo nextest run --workspace`
  `cargo clippy --workspace --all-targets -- -D warnings`
  `cargo check -p ares-core --target wasm32-unknown-unknown`
  `git diff --check`
  `git diff --cached --check`
- Touched Rust files remain at or below 400 LOC.
