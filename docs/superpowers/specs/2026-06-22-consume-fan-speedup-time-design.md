# Fan Speedup Time Runtime Slice Design

## Source Boundary

This slice ports a narrow Rust rewrite of OrcaSlicer part-cooling fan speed-up behavior from:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1311-1312`: `fan_speedup_overhangs` and `fan_speedup_time` fields on `GCodeConfig`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3710-3727`: option defaults and user-facing semantics.
- `OrcaSlicer/src/libslic3r/GCode.cpp:3676-3684` and `GCode.cpp:3774-3782`: construction of `FanMover` when `fan_speedup_time != 0 || fan_kickstart > 0`.
- `OrcaSlicer/src/libslic3r/GCode/FanMover.hpp` and `GCode/FanMover.cpp`: fan-command buffering, overhang-only gating, and fan command reposting through `GCodeWriter::set_fan`.

The Rust destination boundary is `ares-core` fan G-code generation: `crates/ares-core/src/options/part_cooling_fan.rs`, a new small fan-speedup option module if needed to preserve the 400 LOC rule, `crates/ares-core/src/gcode_role_fan.rs`, `crates/ares-core/src/gcode.rs`, and focused fan pipeline tests.

## Problem

Ares already exposes `fan_speedup_overhangs` and `fan_speedup_time` in generated metadata, and already consumes `fan_kickstart` into real `M106` output. However, `fan_speedup_time` currently has no runtime effect. Profiles that ask the fan to start early before a higher-speed part-cooling command still receive the same just-in-time `M106` placement as profiles with the default `fan_speedup_time = 0`.

The user explicitly wants existing Orca options consumed into concrete slicing or G-code behavior instead of adding more option metadata.

## Included Behavior

- Parse `fan_speedup_time` as a scalar floating-point seconds value with Orca default `0`.
- Reject non-finite or negative `fan_speedup_time` values with `SliceError::InvalidInput`.
- Parse `fan_speedup_overhangs` as a boolean with Orca default `true`.
- Carry both values into the existing `RoleFanGCodeState`.
- Preserve existing output when `fan_speedup_time = 0`.
- Treat positive `fan_speedup_time` as enabling a bounded one-generated-move lookahead in this slice. If a generated part-cooling `M106` upshift is about to be emitted for an eligible print move and there is a previous same-layer generated move still in the one-move buffer, emit the fan command before that previous move. If there is no previous same-layer generated move, keep the command at its existing just-before-current-move position.
- If `fan_speedup_overhangs = true`, apply the early placement only to overhang/bridge fan upshifts already represented by Ares role fan overrides: `Bridge`, `InternalBridge`, and `OverhangPerimeter`.
- If `fan_speedup_overhangs = false`, also apply early placement to non-overhang generated role fan upshifts, including the current `ExternalPerimeter` override path produced by `overhang_fan_threshold = "0%"`.
- Keep per-layer baseline fan commands at their existing layer-start position in this slice; moving them into previous-layer motion requires cross-layer buffering and remains deferred.
- Keep the existing `fan_kickstart` pulse behavior compatible: when a command is both sped up and kickstarted, the 100% pulse remains first and the target restore is still delayed by move time.
- Use the existing `GCodeWriter::set_fan` path so flavor-specific fan formatting and `part_cooling_fan_min_pwm` remain unchanged.

## Deferred Behavior

- Full Orca `FanMover` buffering across arbitrary G-code text.
- Splitting `G1`/`G0` moves to insert fan commands inside a move.
- Cross-layer fan command movement beyond the current layer loop.
- Fan speed-up for custom G-code fan commands.
- Arc-fitting time estimation.
- Multi-extruder fan routing and Bambu-specific fan addressing beyond existing writer behavior.
- Toolchange and wipe-tower fan handling.
- Exact Orca treatment of negative `fan_speedup_time` as `with_D_option`; this slice rejects negative values because Ares has no `D` fan command path and negative values are not needed for the current concrete runtime behavior.

## Design

Add a compact `FanSpeedupControl` value to the part-cooling fan runtime options. `SliceOptions` parses it next to the existing fan ramp configuration. If `part_cooling_fan.rs` would exceed 400 LOC, move the new parsing and supporting types into a small sibling module rather than growing the existing file.

`RoleFanGCodeState` receives the speed-up control at construction. It already owns logical fan speed, physical fan speed, pending kickstart restoration, and move-time advancement. Extend `set_speed` so callers can describe whether a fan command is a baseline command or a role override for a specific `PrintPathRole`, and have the state report whether the emitted command is eligible for speed-up placement.

For this slice, “speed-up” means “place the generated fan upshift before the previous same-layer generated move when possible.” Ares does not yet have Orca's post-processing queue that can move commands by an exact number of seconds or split moves, so all positive `fan_speedup_time` values intentionally behave the same in this bounded slice. Exact seconds-based placement remains deferred. This still gives concrete behavior: in a same-layer sequence such as sparse infill followed by bridge, the bridge fan upshift moves from immediately before the bridge move to before the preceding sparse-infill move.

The `gcode.rs` layer loop should use a one-move output buffer around the existing per-move fan, retraction, role-change, and move emission. When the fan state reports a speed-up-eligible command for the current move and a previous same-layer move is buffered, flush the fan command before the buffered previous move, then flush the previous move, then continue with the current move as the new buffer. When no eligible speed-up command exists, flush the previous move before buffering the current move. At layer boundaries and finish, flush any buffered move before emitting layer/finish commands.

Tests should assert visible G-code ordering and value changes, not just parsed options.

## Acceptance Criteria

- Focused option tests prove the defaults and validation for `fan_speedup_time` and `fan_speedup_overhangs`.
- A focused RED test fails before implementation because `fan_speedup_time > 0` does not move an eligible bridge/overhang role fan upshift before the previous same-layer generated move.
- With `fan_speedup_time = 0`, existing fan output remains unchanged.
- With `fan_speedup_time > 0` and default `fan_speedup_overhangs = true`, a bridge/overhang/internal-bridge role fan upshift is emitted before the previous same-layer generated move when one exists.
- With `fan_speedup_time > 0` and default `fan_speedup_overhangs = true`, a non-overhang `ExternalPerimeter` override from `overhang_fan_threshold = "0%"` remains at its existing current-move boundary.
- With `fan_speedup_time > 0` and `fan_speedup_overhangs = false`, the same `ExternalPerimeter` override from `overhang_fan_threshold = "0%"` is emitted before the previous same-layer generated move.
- Positive `fan_speedup_time` values have identical placement in this slice; exact seconds-based placement is documented as deferred.
- Existing `fan_kickstart` tests continue to pass.
- Verification uses `cargo nextest run`, including focused fan tests and full workspace nextest before commit.
- All touched Rust source files remain at or below 400 LOC.

## Documentation Impact

Update `docs/roadmap.md` with a new 2026-06-22 runtime slice entry describing the source boundary, included behavior, and deferred full `FanMover` behavior.
