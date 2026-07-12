# Consume Auxiliary Fan G-code Design

## Goal

Consume the existing `auxiliary_fan` and `additional_cooling_fan_speed` options in generated G-code as a source-cited Rust rewrite slice of OrcaSlicer auxiliary part-cooling fan output.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1404` declares `auxiliary_fan` on `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3704-3708` registers `auxiliary_fan` as the machine capability gate, defaulting to `false`, with G-code command `M106 P2 S(0-255)`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4660-4668` registers `additional_cooling_fan_speed` as the auxiliary part-cooling fan speed, defaulting to `0`, with G-code command `M106 P2 S(0-255)`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:743` reads `additional_cooling_fan_speed` into the additional fan speed candidate.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:800` forces `additional_fan_speed_new = 0` before the close-fan layer threshold.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:815-818` emits `set_additional_fan` only when the additional fan speed changes and `auxiliary_fan` is enabled.
- `OrcaSlicer/src/libslic3r/GCodeWriter.hpp:107-108` exposes `set_additional_fan`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1141-1156` formats auxiliary fan commands as `M106 P2 S{floor(255 * percent / 100)}`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3408-3415` closes the additional fan at print end when `auxiliary_fan` is enabled.

## Ares Destination Boundary

- `crates/ares-core/src/options/auxiliary_fan.rs` will own parsing current auxiliary fan runtime options and selecting during-print/completion speeds.
- `crates/ares-core/src/options.rs` will expose the new runtime module.
- `crates/ares-core/src/gcode_writer.rs` will add auxiliary fan command formatting.
- `crates/ares-core/src/gcode_auxiliary_fan.rs`, `crates/ares-core/src/gcode_print_move.rs`, `crates/ares-core/src/gcode_startup.rs`, and `crates/ares-core/src/gcode.rs` will place auxiliary fan commands in generated output while keeping `gcode.rs` under the repo LOC limit.
- `crates/ares-core/src/options/tests/auxiliary_fan_runtime.rs`, `crates/ares-core/src/gcode_writer/tests/mod.rs`, and a new generated-output test module will cover behavior.
- No option metadata, registry changes, CLI/WASM changes, FanMover behavior, full `CoolingBuffer` post-processing, role-based fan markers, or Ares-owned pipeline design is added in this slice.

## Included Behavior

- Parse `auxiliary_fan` as a scalar boolean, defaulting to `false`.
- Parse `additional_cooling_fan_speed` as the first integer-compatible vector entry, defaulting to `0`, and reject values outside `0..=100`.
- Accept `additional_cooling_fan_speed` in the same concrete first-entry forms as current integer vector runtime parsing: scalar integer JSON number, integer string, first entry of a JSON number array, first entry of a semicolon-separated string, and first entry of a comma-separated string.
- Reject `additional_cooling_fan_speed` values that are fractional numbers or fractional strings, empty strings, strings with empty list entries, empty arrays, arrays containing non-numeric values, booleans, null, objects, negative values, or values above `100`.
- If `auxiliary_fan` is disabled, emit no auxiliary fan commands even when `additional_cooling_fan_speed` is non-zero.
- If `auxiliary_fan` is enabled and `additional_cooling_fan_speed` is zero, emit no during-print command and emit no completion shutdown command.
- If `auxiliary_fan` is enabled and `additional_cooling_fan_speed` is non-zero, port only the additional-fan speed selection and `auxiliary_fan` gate from `CoolingBuffer`: emit `M106 P2 S{pwm}` after the close-fan threshold is reached and before that layer's segment output.
- Reuse `close_fan_the_first_x_layers` for this first concrete slice so the auxiliary fan does not run during the same no-cooling first layers as the part-cooling fan. Dedicated `close_additional_fan_first_x_layers`, `first_x_layer_fan_speed`, and `additional_fan_full_speed_layer` remain deferred to a later auxiliary-fan ramp slice.
- Emit `M106 P2 S0` before `M2` at completion only when a non-zero auxiliary fan command was emitted during the print.
- Skip auxiliary fan commands for `gcode_flavor = "klipper"` in this slice, matching current Ares startup/completion policy for chamber and exhaust fan machine commands.

## Deferred Behavior

- Do not consume `close_additional_fan_first_x_layers`, `first_x_layer_fan_speed`, or `additional_fan_full_speed_layer`.
- Do not implement full `CoolingBuffer` layer-time fan speed selection or line-time slowdown.
- Do not implement FanMover `fan_speedup_time`, `fan_speedup_overhangs`, or `fan_kickstart`.
- Do not add role-based fan behavior for overhang, support interface, internal bridge, or ironing.
- Do not add option metadata or public API surface beyond the crate-internal runtime selection needed for G-code generation.

## Docs Impact

- This spec and its implementation plan are the documentation for the runtime slice.
- No user-facing docs, architecture docs, or roadmap changes are required because the change consumes existing Orca options in generated G-code behavior without adding a public API, CLI flag, crate, or milestone boundary.

## Acceptance Criteria

- Default options emit no `M106 P2` auxiliary fan commands.
- `auxiliary_fan = false` with `additional_cooling_fan_speed = 70` emits no `M106 P2` commands.
- `auxiliary_fan = true` with default `additional_cooling_fan_speed = 0` emits no `M106 P2` commands.
- `auxiliary_fan = true`, `additional_cooling_fan_speed = 70`, and default `close_fan_the_first_x_layers = 1` emits `M106 P2 S178` after `;LAYER:1`, emits no `M106 P2` command on layer 0, and emits `M106 P2 S0` before `M2`.
- `auxiliary_fan = true`, `additional_cooling_fan_speed = 70`, and `close_fan_the_first_x_layers = 0` emits `M106 P2 S178` on layer 0 before `; segment_count`.
- `gcode_flavor = "klipper"` with auxiliary fan enabled emits no `M106 P2` commands.
- Invalid `auxiliary_fan` values return `SliceError::InvalidInput` mentioning `auxiliary_fan`.
- Invalid `additional_cooling_fan_speed` values return `SliceError::InvalidInput` mentioning `additional_cooling_fan_speed`.
- All changed Rust source files under `crates/ares-core/src` remain at or below 400 LOC.

## Verification

- Targeted tests cover parser defaults, enabled/disabled gate behavior, accepted/rejected speed forms, writer formatting, generated G-code placement, completion shutdown, close-fan threshold interaction, and Klipper skip behavior.
- Final verification runs:
  - `cargo fmt --check`
  - `cargo test -p ares-core --lib`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - the repo LOC gate for `crates/ares-core/src/**/*.rs`.
