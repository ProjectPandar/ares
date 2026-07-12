# Consume Part Cooling Fan Ramp Options Design

## Goal

Implement a small source-cited part-cooling fan runtime slice by consuming the existing `fan_min_speed`, `fan_max_speed`, and `full_fan_speed_layer` options in generated G-code.

## Source Boundary

This slice ports the basic fan-speed command formatting and layer-ramp behavior from:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1534-1538`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3325-3335`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4591-4599`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4651-4658`
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:740-773`
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:812`
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp:100-106`
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1095-1137`

Existing Ares registry metadata for `fan_min_speed`, `fan_max_speed`, and `full_fan_speed_layer` is reused. No new option metadata is added.

## Ares Destination Boundary

The Rust destination boundary is limited to:

- `crates/ares-core/src/options/part_cooling_fan.rs`: runtime option accessor for the current single-tool part-cooling fan ramp.
- `crates/ares-core/src/options.rs`: module registration and crate-private export only; this file is already at the 400 LOC gate, so one existing module declaration line must be compacted or reused without changing behavior.
- `crates/ares-core/src/options/tests/part_cooling_fan_runtime.rs`: runtime option parsing and ramp calculation tests.
- `crates/ares-core/src/options/tests.rs`: test module registration.
- `crates/ares-core/src/gcode_writer.rs`: part fan command formatter matching Orca's basic `M106` forms for active G-code flavors in Ares.
- `crates/ares-core/src/gcode_writer/tests.rs`: part fan formatting tests.
- `crates/ares-core/src/gcode.rs`: emit a layer-start fan command when the effective fan speed changes.
- `crates/ares-core/src/tests/part_cooling_fan_gcode.rs`: slice-level fan G-code behavior tests.
- `crates/ares-core/src/tests/mod.rs`: test module registration.

No other crates, dependencies, registry definitions, profile metadata, UI code, or pipeline architecture are in scope. All changed `crates/ares-core/src/*.rs` files must remain at or below 400 LOC. The implementation may use small helper modules inside `crates/ares-core/src` and may surgically compact existing module declarations in `options.rs` or test module declarations in `tests/mod.rs` without changing behavior. It must not split, rewrite, or reformat unrelated generated option metadata.

## Runtime Behavior

Ares must parse these existing options:

- `fan_min_speed`: non-negative numeric percent, numeric string, numeric-number array, or semicolon/comma separated numeric string. Missing defaults to `[20]`.
- `fan_max_speed`: non-negative numeric percent, numeric string, numeric-number array, or semicolon/comma separated numeric string. Missing defaults to `[100]`.
- `full_fan_speed_layer`: non-negative integer, integer string, integer-number array, or semicolon/comma separated integer string. Missing defaults to `[0]`.

For Ares' current single-tool output, the first entry of each parsed vector is used. Numeric arrays must contain only JSON numbers; string elements inside arrays are invalid even if the string contains a number. String scalar forms may contain one value or a semicolon/comma separated list. Empty strings, empty separators such as `"25;"`, empty arrays, arrays containing strings, arrays containing booleans, objects, booleans, and null are invalid. Percent values are limited to the upstream UI range `0..=100` by returning `SliceError::InvalidInput` for values above 100, negative values, or non-finite values. `full_fan_speed_layer` rejects floats, negative values, and integers outside `u32`. `fan_min_speed` values greater than `fan_max_speed` are normalized down to `fan_max_speed` for this slice.

The effective fan speed for each printed layer is:

- If `fan_max_speed == 0`, emit no fan commands.
- If `full_fan_speed_layer <= 1`, use `fan_max_speed` from the first printed layer onward.
- If the one-based layer number is lower than `full_fan_speed_layer`, linearly ramp from `fan_min_speed` toward `fan_max_speed`:
  `fan_min_speed + (fan_max_speed - fan_min_speed) * (layer_number - 1) / (full_fan_speed_layer - 1)`
- If the one-based layer number is greater than or equal to `full_fan_speed_layer`, use `fan_max_speed`.

The ramp is intentionally defined from `fan_min_speed` to `fan_max_speed` instead of Orca's full CoolingBuffer layer-time value because Ares does not yet calculate per-layer print time or cooling slowdown. This still consumes the cited fan options as concrete G-code behavior and leaves CoolingBuffer's time-based fan behavior for a later source slice.

When the effective speed changes, Ares emits a part-cooling fan command immediately after the layer's Z travel command and before segment/path comments for that layer. This keeps the slice local to Ares' existing layer loop and makes fan state changes visible before extrusion moves on that layer.

`GCodeWriter::set_fan` formats percent speeds as Orca does:

- `0` emits `M106 S0` for Marlin/RepRap/Klipper/Teacup/Mach3/Machinekit-style flavors and `M127` for MakerWare/Sailfish.
- Non-zero speeds emit `M106 S{floor(255.5 * speed / 100)}` for Marlin/RepRap/Klipper/Teacup-style flavors, `M106 P{floor(255.5 * speed / 100)}` for Mach3/Machinekit, and `M126` for MakerWare/Sailfish.

MakerWare, Sailfish, Mach3, Machinekit, Teacup, Smoothie, RepRapSprinter, and NoExtrusion remain inactive through `SliceOptions::gcode_flavor` in this slice. Their fan command behavior is covered only by direct `GCodeWriter` unit tests against the existing `GCodeFlavor` enum. This slice does not change flavor parsing, registry metadata, or profile compatibility.

The first emitted fan command must be non-zero for the default options because Orca defaults are `fan_min_speed = 20`, `fan_max_speed = 100`, and `full_fan_speed_layer = 0`.

## Deferred Behavior

This slice does not implement:

- `close_fan_the_first_x_layers`.
- Layer-time cooling logic from `slow_down_layer_time` / `fan_cooling_layer_time`.
- `reduce_fan_stop_start_freq`.
- Overhang, internal bridge, support interface, ironing, auxiliary, chamber, exhaust, or air-filtration fan controls.
- `part_cooling_fan_min_pwm` PWM floor.
- FanMover delayed movement of fan commands.
- Multi-extruder fan switching beyond first-vector-entry semantics.
- Fan comments controlled by Orca's `full_gcode_comment`; Ares' existing `gcode_comments` command comments are not changed by this slice.

## Docs Impact

No user-facing documentation, architecture decision record, roadmap entry, or example update is required for this runtime slice. The new SDD spec and plan are the traceable design artifacts, and the public API surface remains unchanged.

## Acceptance Criteria

- Default slicing output emits a part-cooling fan command before first-layer extrusion.
- `fan_max_speed = 0` emits no part-cooling fan commands.
- `fan_min_speed = 25`, `fan_max_speed = 75`, and `full_fan_speed_layer = 3` emit layer-start fan commands for 25%, 50%, then 75%, suppressing unchanged later layer commands.
- `full_fan_speed_layer = 1` emits `fan_max_speed` on the first printed layer.
- `fan_min_speed > fan_max_speed` emits `fan_max_speed`, not an increasing or invalid ramp.
- Invalid fan speed or full-speed-layer values return `SliceError::InvalidInput` mentioning the offending option key.
- MakerWare/Sailfish and Mach3/Machinekit fan command formatting follows the source-cited writer behavior.
- All changed `crates/ares-core/src/*.rs` files remain at or below 400 LOC.

## Verification Criteria

The implementation is not complete until all of the following pass with fresh output:

- Targeted runtime option tests for defaults, accepted numeric forms, ramp calculation, min/max normalization, zero suppression, and invalid inputs.
- Targeted writer tests for `M106 S`, `M106 P`, `M126`, `M127`, and fan-off formatting.
- Targeted slice-level tests for default fan output, disabled max-speed output, ramped layer-start output, full-speed-first-layer output, invalid input propagation, and command placement before extrusion moves.
- `cargo fmt --check`
- `cargo test -p ares-core --lib`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- A `crates/ares-core/src/*.rs` LOC check proving every core source file is at or below 400 lines.
