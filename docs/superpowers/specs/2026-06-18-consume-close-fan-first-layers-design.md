# Consume Close Fan First Layers Design

## Goal

Consume the existing `close_fan_the_first_x_layers` option in generated part-cooling fan G-code, as a source-cited Rust rewrite slice of OrcaSlicer `CoolingBuffer` fan-speed selection.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1510-1511` declares `slow_down_for_layer_cooling` and `close_fan_the_first_x_layers` in `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1837-1845` registers `close_fan_the_first_x_layers` as the number of first layers with cooling fans off, defaulting to `1`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:744-755` reads `close_fan_the_first_x_layers` and only calculates non-zero part-cooling fan speeds when `layer_id >= close_fan_the_first_x_layers`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:771-775` ramps part-cooling fan speed from `close_fan_the_first_x_layers` toward `full_fan_speed_layer`.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:796-800` forces part and additional fan speed to `0` before the close-fan layer threshold.

## Ares Destination Boundary

- `crates/ares-core/src/options/part_cooling_fan.rs` owns parsing current part-cooling fan runtime options and the layer-speed selection model.
- `crates/ares-core/src/gcode.rs` already emits layer-start fan commands from `PartCoolingFanRamp::speed_for_layer`.
- `crates/ares-core/src/options/tests/part_cooling_fan_runtime.rs` and `crates/ares-core/src/tests/part_cooling_fan_gcode.rs` own parser and generated-output tests.
- No option metadata, registry, public API, CLI, WASM, additional fan, or full `CoolingBuffer` post-processing behavior is added in this slice.

## Included Behavior

- Parse existing `close_fan_the_first_x_layers` from `SliceOptions`.
- Default missing `close_fan_the_first_x_layers` to `1`, matching upstream `PrintConfig.cpp:1845`.
- Accept integer-compatible scalar and first-entry vector/string forms consistently with current `full_fan_speed_layer` parsing.
- Reject invalid values at the same option parsing boundary used by current part-cooling fan behavior.
- For layer indexes where `layer_index < close_fan_the_first_x_layers`, return no part-cooling fan command.
- Once the close-fan threshold is reached, emit normal part-cooling fan commands again.
- When `full_fan_speed_layer > close_fan_the_first_x_layers`, ramp from the close-fan threshold toward max speed using the upstream factor from `CoolingBuffer.cpp:771-775`:
  - `factor = (layer_id + 1 - close_fan_the_first_x_layers) / (full_fan_speed_layer - close_fan_the_first_x_layers)`.
  - First compute Ares' existing min-to-max base fan speed for the layer.
  - Scale that base fan speed by `factor`, round with `+0.5`, and clamp to `0..=100`.
- When `full_fan_speed_layer <= close_fan_the_first_x_layers + 1`, use max fan speed on the first layer after the close-fan threshold, matching the upstream condition `layer_id + 1 < full_fan_speed_layer`.
- Preserve existing behavior when `fan_max_speed = 0`: no part-cooling fan commands are emitted.
- Preserve existing `part_cooling_fan_min_pwm` behavior after fan speed selection. If the close-fan threshold yields `None`, no floor command is emitted; if later non-zero fan speed is below the PWM floor, the writer still applies the floor.

## Deferred Behavior

- Do not implement full `CoolingBuffer` post-processing or line-time estimation.
- Do not consume `slow_down_layer_time`, `slow_down_min_speed`, `fan_cooling_layer_time`, `slow_down_for_layer_cooling`, `reduce_fan_stop_start_freq`, overhang fan options, support-interface fan options, internal-bridge fan options, ironing fan options, auxiliary fan options, fan speedup, or fan kickstart in this slice.
- Do not change deterministic default G-code unless the approved acceptance criteria and tests explicitly require it.

## Docs Impact

- This spec and its implementation plan are the documentation for the runtime slice.
- No user-facing docs, architecture docs, or roadmap changes are required because the change consumes an existing Orca option in generated G-code behavior without adding a public API, CLI flag, crate, or milestone boundary.

## Acceptance Criteria

- Default options emit no part-cooling fan command on layer 0 and emit the default max part-cooling fan command on the first layer after the close-fan threshold.
- `close_fan_the_first_x_layers = 0` preserves immediate first-layer fan output while still applying the source-cited close-layer ramp factor from `CoolingBuffer.cpp:771-775`; for example, `fan_min_speed = 25`, `fan_max_speed = 75`, and `full_fan_speed_layer = 3` yields 8%, 33%, then 75%.
- `close_fan_the_first_x_layers = 2` suppresses fan commands for layers 0 and 1, then emits a fan command on layer 2.
- With `fan_min_speed = 20`, `fan_max_speed = 60`, `full_fan_speed_layer = 4`, and `close_fan_the_first_x_layers = 1`, layer 0 emits no fan command, layer 1 emits a ramped command for 11%, layer 2 emits a ramped command for 31%, and layer 3 emits max 60%.
- With `fan_max_speed = 0`, no part-cooling fan commands are emitted regardless of `close_fan_the_first_x_layers`.
- Invalid `close_fan_the_first_x_layers` values return `SliceError::InvalidInput` mentioning the option key.
- All changed Rust source files under `crates/ares-core/src` remain at or below 400 LOC.

## Verification

- Targeted tests cover parser defaults, parser accepted/rejected forms, ramp speed selection, generated G-code placement, and interaction with `part_cooling_fan_min_pwm`.
- Final verification runs:
  - `cargo fmt --check`
  - `cargo test -p ares-core --lib`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `git diff --check`
  - the repo LOC gate for `crates/ares-core/src/**/*.rs`.
