# Consume Fan Kickstart Design

## Purpose

Consume the existing OrcaSlicer `fan_kickstart` option into concrete Ares part-cooling fan G-code behavior. The slice must make a non-zero `fan_kickstart` visibly affect `M106` output instead of remaining registry metadata, while staying inside Ares' current single-pass, layer-oriented G-code writer.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1310` declares `GCodeConfig::fan_kickstart` as `ConfigOptionFloat`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3729-3738` defines the option as non-negative seconds, default `0`, with the behavior "emit max fan speed for this amount of seconds before reducing to target speed."
- `OrcaSlicer/src/libslic3r/GCode.cpp:3676-3686` and `3774-3784` instantiate `GCode/FanMover` whenever `fan_speedup_time != 0` or `fan_kickstart > 0`.
- `OrcaSlicer/src/libslic3r/GCode/FanMover.cpp:314-410` reacts to part-cooling `M106` speed increases by emitting a 100% fan command and scheduling the target fan command after a duration derived from `fan_kickstart`.
- `OrcaSlicer/src/libslic3r/GCode/FanMover.cpp:463-505` restores the delayed target fan command after buffered movement time reaches the kickstart duration.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:1095-1138` formats part-cooling fan commands and applies `part_cooling_fan_min_pwm` to non-zero fan speeds.

## Ares Boundary

- `crates/ares-core/src/options/part_cooling_fan.rs` owns parsed part-cooling fan runtime options.
- `crates/ares-core/src/gcode_role_fan.rs` owns part-cooling fan G-code state for baseline and role override fan commands.
- `crates/ares-core/src/gcode.rs` owns the layer/move loop that can provide move timing to the fan state.
- `crates/ares-core/src/gcode_writer.rs` owns flavor-aware `M106` formatting and already applies `part_cooling_fan_min_pwm`.
- `crates/ares-core/src/pipeline/tests/` owns pipeline-level G-code assertions for fan behavior.

## Included Behavior

- Parse `fan_kickstart` as a scalar float in seconds from existing `SliceOptions` values.
- Default `fan_kickstart` to `0.0`, preserving current G-code when absent or zero.
- Reject negative, non-finite, non-numeric, or malformed `fan_kickstart` values with `SliceError::InvalidInput` mentioning `fan_kickstart`.
- When `fan_kickstart > 0.0`, a part-cooling fan command that raises the requested logical speed by more than 10 percentage points from the current logical speed to a higher non-zero target emits an immediate 100% part-cooling fan command before the target command is restored. This intentionally follows the no-`fan_speedup_time` `FanMover.cpp:394-408` branch because `fan_speedup_time` is deferred in this slice.
- Compute the pending target restore duration with Orca's scaled formula from `FanMover.cpp:343` and `398`: `fan_kickstart * (target_speed - previous_logical_speed) / 100.0`.
- Accumulate movement time from Ares' emitted XY move stream after the kickstart command. The time source is the zipped `SpeedMove` sequence already used by `gcode.rs`: distance between consecutive emitted XY move points divided by `SpeedMove::speed_mm_s()`. Both travel and print moves count, matching Orca's G0/G1 move scope; zero-length moves and non-positive speeds add no time. Layer Z travel emitted before layer fan commands does not count.
- Restore the target logical speed after accumulated emitted-move time reaches the scaled kickstart duration. Ares may restore only on existing move boundaries; it must not split `G1` moves in this slice. The restore command is emitted after the move that meets or exceeds the duration and before the following emitted move.
- If available movement time in the current layer is shorter than the scaled kickstart duration, the target command remains pending into later moves in the same print. If the print ends first, Ares must flush the pending target command before finish G-code so final fan state is not left at 100%.
- Use the existing `GCodeWriter::set_fan` path for both the 100% kickstart command and restored target command so G-code flavor and `part_cooling_fan_min_pwm` behavior remain centralized.
- Apply the same kickstart state machine to baseline fan ramp commands and role fan override commands, because both are part-cooling `M106` requests in Ares' current writer.
- Do not emit a kickstart pulse when the requested logical speed is unchanged, lower than the current logical speed, zero, or at most 10 percentage points above the current logical speed.
- Track pending kickstart target state explicitly. If a new fan request arrives while a target is pending:
  - A higher upshift replaces the pending target, keeps the 100% physical fan command active, and adds another scaled duration using the old pending target as `previous_logical_speed`.
  - An equal, lower, or zero request cancels the pending target and emits that requested logical speed immediately without a kickstart pulse.
  - A fan-off request always cancels pending kickstart state and emits fan-off immediately.

## Deferred Behavior

- Full Orca `FanMover` parity remains deferred: moving fan commands earlier in time, G-code reordering through a time buffer, `_print_in_middle_G1`, `_put_in_middle_G1`, `_remove_slow_fan`, and G1/G0 splitting are not implemented here.
- `fan_speedup_time` and `fan_speedup_overhangs` remain deferred. This slice only consumes `fan_kickstart`.
- Ares does not yet post-process arbitrary custom G-code `M106` commands through a `FanMover` equivalent. This slice only covers Ares-generated part-cooling fan commands.
- Multi-extruder fan routing, Bambu-specific `M106 P1`, wipe tower fan handling, and arc-aware timing remain deferred.
- Exact Orca time-buffer placement remains deferred. Ares restores on emitted move boundaries instead of inserting commands into the middle of a move.

## Docs Impact

Update `docs/roadmap.md` with a new runtime slice entry after implementation. The entry must name the same upstream boundary and explicitly list the deferred full `FanMover` behavior.

## Acceptance Criteria

- A focused nextest test proves absent or zero `fan_kickstart` preserves the current `M106` sequence for a baseline fan command.
- A focused RED/GREEN nextest test uses deterministic move duration to prove `fan_kickstart > 0` inserts a 100% `M106` before a lower target part-cooling fan baseline command and restores the target after a move whose computed duration meets or exceeds `fan_kickstart * (target_speed - previous_logical_speed) / 100.0`, before the following emitted move.
- A focused nextest test proves role fan overrides also pass through kickstart, for example baseline 40% to overhang/bridge 75% emits 100% before the 75% role fan target and later restores baseline.
- A focused nextest test proves downshifts and fan-off commands do not receive kickstart pulses.
- A focused nextest test proves a new fan request arriving while a kickstart target is pending either replaces the pending target for a larger upshift or cancels it for an equal/lower request.
- A focused nextest test proves invalid `fan_kickstart` values return `SliceError::InvalidInput`.
- Existing fan behavior tests for `fan_min_speed`, `fan_max_speed`, `full_fan_speed_layer`, `close_fan_the_first_x_layers`, `part_cooling_fan_min_pwm`, `overhang_fan_speed`, `internal_bridge_fan_speed`, and `fan_cooling_layer_time` continue to pass.
- Verification uses `cargo nextest run`, not `cargo test`.

## Safety

The change is limited to platform-neutral `ares-core` option parsing and G-code string generation. It introduces no filesystem access, terminal behavior, UI, OpenGL, native-only APIs, dependencies, feature flags, or compatibility fallbacks.
