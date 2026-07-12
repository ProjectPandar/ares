# Consume Part Cooling Fan Minimum PWM Design

## Goal

Consume the existing `part_cooling_fan_min_pwm` option in generated part-cooling fan G-code, as a source-cited Rust rewrite slice of OrcaSlicer `GCodeWriter::set_fan`.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1313-1316` documents and declares `part_cooling_fan_min_pwm` inside `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3740-3755` registers the option as the minimum non-zero part-cooling fan speed.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1095-1101` clamps non-zero fan speeds below `part_cooling_fan_min_pwm` before flavor-specific fan command formatting.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1133-1138` passes the active config value into `GCodeWriter::set_fan`.
- `OrcaSlicer/src/libslic3r/GCode/FanMover.cpp:231-236` applies the same floor when reposting fan commands, but Ares does not yet have the FanMover buffering layer. This slice records that adjacent behavior as deferred.

## Ares Destination Boundary

- `crates/ares-core/src/options/part_cooling_fan.rs` owns parsing of existing part-cooling fan options into runtime behavior.
- `crates/ares-core/src/gcode_writer.rs` owns flavor-specific part-cooling fan command formatting.
- `crates/ares-core/src/gcode.rs` wires parsed runtime options into the generated layer G-code.
- Tests remain under `crates/ares-core/src/options/tests/` and `crates/ares-core/src/gcode_writer/` after splitting oversized writer tests if needed.
- LOC-safe test splits or surgical compaction are allowed only when directly needed to keep changed `crates/ares-core/src/*.rs` files at or below 400 LOC.

## Included Behavior

- Parse existing `part_cooling_fan_min_pwm` from `SliceOptions`.
- Default missing `part_cooling_fan_min_pwm` to `0`, preserving current output.
- Accept integer-compatible scalar values in the percent range `0..=100`.
- Reject invalid values at the same option parsing boundary used by current part-cooling fan behavior.
- Apply the floor only to non-zero part-cooling fan percentages.
- Preserve `0` exactly so fan-off commands remain fan-off commands.
- Apply the floor before converting percentage to firmware PWM output, matching OrcaSlicer `GCodeWriter.cpp:1095-1101`.
- Preserve existing flavor-specific fan command forms:
  - MakerWare/Sailfish: `M127` for off, `M126` for any non-zero speed.
  - Mach3/Machinekit: `M106 S0` for off, `M106 P<pwm>` for non-zero.
  - Other flavors: `M106 S<pwm>`.
- Cover inactive flavors only through direct `GCodeWriter` unit tests; do not expand `SliceOptions::gcode_flavor` parsing or registry behavior in this slice.

## Deferred Behavior

- Do not implement FanMover delayed fan-command movement from `OrcaSlicer/src/libslic3r/GCode/FanMover.cpp`.
- Do not consume `fan_kickstart`, `fan_speedup_time`, `fan_speedup_overhangs`, overhang fan options, slow-down layer-time options, or first-X-layer fan options in this slice.
- Do not add new option metadata, public API, crates, dependencies, or an Ares-owned slicing pipeline feature.

## Input Contract

- Missing `part_cooling_fan_min_pwm` means `0`.
- Accepted values are JSON numbers with an integer value in `0..=100`.
- Rejected values include negative numbers, values above 100, fractional numbers, strings, arrays, booleans, objects, and null.
- This stricter scalar contract matches the current GCodeConfig option shape and avoids adding vector/extruder fan-floor behavior before the upstream rewrite boundary requires it.

## Docs Impact

- This spec and its implementation plan are the documentation for the runtime slice.
- No user-facing usage docs, architecture docs, or roadmap changes are required because this slice only consumes an already-registered option in existing generated G-code behavior and does not add a public API, CLI flag, or new milestone boundary.

## Acceptance Criteria

- With default options, generated part-cooling fan G-code is unchanged.
- With `fan_min_speed = 20`, `fan_max_speed = 60`, `full_fan_speed_layer = 3`, and `part_cooling_fan_min_pwm = 30`, the first emitted part-cooling fan command is clamped from 20% to 30% before PWM conversion.
- With `fan_max_speed = 0`, no part-cooling fan commands are emitted, even when `part_cooling_fan_min_pwm` is non-zero.
- Calling the writer with fan speed `0` still emits the flavor-specific off command, not the PWM floor.
- Calling the writer with a non-zero speed below the configured floor emits the floored PWM value.
- Calling the writer with a non-zero speed at or above the configured floor emits the requested speed.
- Invalid `part_cooling_fan_min_pwm` values outside `0..=100` or with unsupported types return `SliceError::InvalidInput`.
- All changed Rust source files under `crates/ares-core/src` remain at or below 400 LOC.

## Verification

- Targeted tests cover runtime parsing, writer clamp behavior, flavor preservation, and generated layer G-code integration.
- Final verification runs:
  - `cargo fmt --check`
  - `cargo test -p ares-core --lib`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - the repo LOC gate for `crates/ares-core/src/**/*.rs`.
