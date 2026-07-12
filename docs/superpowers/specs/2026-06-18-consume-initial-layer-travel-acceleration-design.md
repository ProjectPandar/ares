# Consume Initial Layer Travel Acceleration Design

## Goal

Implement a source-cited OrcaSlicer rewrite slice that consumes `initial_layer_travel_acceleration` in Ares G-code output, instead of leaving the parsed Orca option as inert metadata.

## Upstream Boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1422` declares `initial_layer_travel_acceleration` as `ConfigOptionFloatOrPercent`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3143-3149` defines it as first-layer travel acceleration, with percent values relative to `travel_acceleration`, minimum `0`, and default `100%`.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7301-7306` reads `initial_layer_travel_acceleration` during first-layer travel handling and emits travel acceleration when `default_acceleration` and the resolved first-layer travel acceleration are positive.
- `OrcaSlicer/src/libslic3r/GCode.cpp:7323-7324` uses `travel_acceleration` for non-first-layer travel moves.

## Ares Destination Boundary

- `crates/ares-core/src/options/acceleration.rs` resolves `initial_layer_travel_acceleration` as a non-negative number or percent over `travel_acceleration`.
- `crates/ares-core/src/speeds/kinematics.rs` stores the resolved first-layer travel acceleration and selects it only for `ToolpathMoveKind::Travel` on the first layer.
- G-code output continues to use the existing `SpeedOptions::acceleration_for_layer` and `GCodeWriter::set_acceleration` path, so the concrete behavior is visible as `M204` before first-layer travel moves.

## Included Behavior

- Missing `initial_layer_travel_acceleration` resolves to the already-resolved `travel_acceleration`, matching Orca's `100%` default over travel acceleration.
- Numeric values such as `420` are used as absolute first-layer travel acceleration in mm/s^2.
- Percent values such as `"50%"` resolve against `travel_acceleration`.
- A resolved value of `0` suppresses first-layer travel acceleration output when `default_acceleration` is positive; it does not fall back to `travel_acceleration`.
- `default_acceleration = 0` continues to suppress all acceleration output.
- Non-first-layer travel moves continue to use `travel_acceleration`.
- First-layer print moves continue to use `initial_layer_acceleration` and role acceleration behavior exactly as before.

## Deferred Behavior

- Orca's short-travel acceleration branch for non-first-layer overhang/external-perimeter travel remains out of scope because Ares does not yet model that upstream travel context.
- Klipper-specific combined acceleration/jerk emission remains out of scope; Ares keeps using the existing flavor-neutral `M204` writer path for this slice.
- Machine acceleration limit capping remains out of scope.
- No support, bridge detection, or new Ares-owned slicing pipeline behavior is introduced.

## Acceptance Criteria

- A first-layer travel move emits `M204` using `initial_layer_travel_acceleration` when provided as an absolute number.
- A first-layer travel move emits `M204` using a percent value relative to `travel_acceleration`.
- A non-first-layer travel move still emits `M204` using `travel_acceleration`.
- `initial_layer_travel_acceleration = 0` suppresses first-layer travel `M204` without suppressing later travel acceleration.
- Invalid `initial_layer_travel_acceleration` values are rejected through the existing acceleration parsing error path.
- Existing acceleration, jerk, speed, and full workspace checks continue to pass.
- No Rust file under `crates/` exceeds 400 LOC.
