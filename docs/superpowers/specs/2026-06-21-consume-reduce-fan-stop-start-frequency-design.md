# Consume Reduce Fan Stop Start Frequency Design

## Goal

Consume OrcaSlicer's `reduce_fan_stop_start_freq` option in Ares part-cooling fan G-code generation so long layers can keep the fan at `fan_min_speed` instead of fully stopping between short-layer cooling events.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1519` declares `reduce_fan_stop_start_freq` as a `ConfigOptionBools` field on `PrintConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2334-2338` defines the option as "Keep fan always on", defaults it to `false`, and documents that enabling it keeps the part-cooling fan at least at minimum speed.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:740-768` uses the current extruder's `reduce_fan_stop_start_freq` value when selecting the baseline part-cooling fan speed for a layer: the baseline starts at `fan_min_speed` when enabled and `0` when disabled, then short-layer cooling can raise it to an interpolated or maximum fan speed.
- `OrcaSlicer/src/libslic3r/GCode/CoolingBuffer.cpp:771-775` applies the existing full-fan-layer ramp after the baseline is selected.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp` remains the writer boundary for converting percent fan speeds into concrete `M106` commands.

## Ares Boundary

- `crates/ares-core/src/options/part_cooling_fan.rs` owns current single-extruder part-cooling fan option parsing and `PartCoolingFanRamp` baseline selection.
- `crates/ares-core/src/gcode_layer_fan.rs` already computes per-layer print time from generated `LayerSpeedMoves` and calls the ramp.
- `crates/ares-core/src/gcode.rs` already routes the returned baseline through existing fan state and `GCodeWriter` formatting.
- `crates/ares-core/src/gcode_role_fan.rs` owns `RoleFanGCodeState`, including the physical fan state needed to suppress a redundant first `M106 S0` while still emitting fan-off when a later baseline transitions from non-zero to zero.
- Tests live in `crates/ares-core/src/options/tests/fan_cooling_layer_time_runtime.rs` for option/ramp behavior and `crates/ares-core/src/tests/part_cooling_fan_gcode.rs` for concrete emitted G-code.

## Included Behavior

- Parse `reduce_fan_stop_start_freq` inside `SliceOptions::part_cooling_fan_ramp()` as a first-entry single-extruder boolean:
  - missing value defaults to `false`;
  - scalar JSON bool is accepted;
  - non-empty JSON bool array uses the first item;
  - non-boolean scalar, empty array, and array whose first item is not a bool are rejected with `SliceError::InvalidInput` mentioning `reduce_fan_stop_start_freq`.
- Preserve existing `fan_min_speed`, `fan_max_speed`, `close_fan_the_first_x_layers`, `full_fan_speed_layer`, `slow_down_layer_time`, `fan_cooling_layer_time`, `fan_kickstart`, role fan, min PWM, and G-code flavor behavior except where the upstream long-layer baseline requires this option.
- When a layer is before `close_fan_the_first_x_layers`, part-cooling baseline remains suppressed.
- When `fan_max_speed = 0`, part-cooling baseline remains suppressed.
- When computed layer time is shorter than `slow_down_layer_time`, baseline remains `fan_max_speed`.
- When computed layer time is at least `slow_down_layer_time` and shorter than `fan_cooling_layer_time`, baseline remains the current Orca-style interpolation between `fan_max_speed` and `fan_min_speed`.
- When computed layer time is at least `fan_cooling_layer_time`, Ares uses `fan_min_speed` if `reduce_fan_stop_start_freq = true` and explicit fan-off speed `0` if it is false, then applies the existing full-fan-layer ramp to that selected baseline.
- The G-code layer baseline path must distinguish "fan is already off" from "fan must be turned off". A first long layer with baseline `0` and no prior physical fan command must not emit a redundant `M106 S0`; a later long layer after a non-zero short-layer baseline or role fan override must emit the flavor-appropriate fan-off command.
- Existing layer-number fallback behavior for callers that do not provide layer time remains available through `speed_for_layer()`; the G-code path continues to provide layer time through `gcode_layer_fan::baseline_speed()`.

## Deferred Behavior

- Full Orca `CoolingBuffer` G-code rewriting, fan command deferral, multi-extruder state, fan command ordering across custom markers, support-interface fan markers, ironing fan markers, wipe-tower cooling, arcs, wipes, and fan speedup scheduling remain deferred.
- `first_x_layer_fan_speed` is not implemented as automatic auxiliary or part-cooling fan behavior in this slice. Local source search shows Orca exposes it to the placeholder parser but does not read it in `GCode/CoolingBuffer.cpp` automatic fan emission.
- No new public API, crate, dependency, file I/O, terminal behavior, UI behavior, OpenGL behavior, or WASM-incompatible code is added.

## Acceptance Criteria

- Runtime option tests prove default `false`, scalar bool parsing, first-entry bool-array parsing, and invalid value rejection for `reduce_fan_stop_start_freq`.
- Runtime ramp tests prove long-layer baseline is `Some(0)` when the option is false and `Some(fan_min_speed)` when true, while short-layer max/interpolated behavior is unchanged.
- Runtime ramp tests prove the selected long-layer minimum baseline is still scaled by `full_fan_speed_layer`.
- G-code tests prove a first long layer emits no redundant part-cooling `M106 S0` when the fan is already off, a later long layer emits fan-off after a prior non-zero part-cooling command when `reduce_fan_stop_start_freq = false`, and the same later long layer emits or keeps `fan_min_speed` when `reduce_fan_stop_start_freq = true`.
- Existing tests for part-cooling fan close layers, full-speed layer ramp, min PWM, role fan overrides, fan kickstart, and `fan_cooling_layer_time` continue to pass.
- Touched Rust files remain at or below 400 LOC.
- Verification uses `cargo nextest run`, not `cargo test`, and includes focused nextest, `cargo fmt --check`, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and a Rust LOC guard for touched files.
