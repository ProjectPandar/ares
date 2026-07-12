# Consume Travel Acceleration G-code Design

## Source Boundary

This slice ports the travel-acceleration G-code behavior from:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1048` for `travel_acceleration`
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1422` for `initial_layer_travel_acceleration`
- `OrcaSlicer/src/libslic3r/GCode.cpp:7301-7342` for selecting first-layer and later-layer travel acceleration before travel moves
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:19-24` and `GCodeWriter.cpp:221-240` for flavors that support separate travel acceleration commands

The Rust destination boundary is `ares-core` G-code emission: `crates/ares-core/src/gcode_writer/acceleration.rs`, `crates/ares-core/src/gcode_writer.rs`, and `crates/ares-core/src/gcode_move_emit.rs`, with behavior tests under `crates/ares-core/src/gcode_writer/tests/` and `crates/ares-core/src/tests/`.

## Current State

Ares already parses `default_acceleration`, `travel_acceleration`, and `initial_layer_travel_acceleration` into `AccelerationOptions`. `SpeedMove` already carries the travel acceleration selected by first-layer and later-layer logic, and `gcode_move_emit` already emits acceleration commands before travel moves.

The gap is command shape. Ares currently emits every non-Klipper acceleration with `M204 S...`, so Marlin2, RepRapFirmware, and Repetier travel moves do not use Orca's separate travel acceleration commands.

## Included Behavior

1. For travel moves:
   - `marlin2` emits `M204 T<accel>` before the travel move.
   - `reprapfirmware` emits `M204 T<accel>` before the travel move.
   - `repetier` emits `M202 X<accel> Y<accel>` before the travel move.
   - `marlin`, `klipper`, and unsupported separate-travel flavors preserve existing generic acceleration behavior.

2. For print moves on separate-travel flavors:
   - `marlin2` and `reprapfirmware` emit `M204 P<accel>`.
   - `repetier` emits `M201 X<accel> Y<accel>`.
   - Existing Marlin legacy and Klipper behavior remains unchanged.

3. The writer keeps print and travel acceleration state separately only for flavors with separate travel acceleration support, matching Orca's separate `m_last_acceleration` / `m_last_travel_acceleration` behavior.

4. `initial_layer_travel_acceleration` continues to use the already-parsed first-layer value and now reaches the separate travel command shape for supported flavors.

5. `gcode_comments = true` appends `; adjust acceleration` to the emitted acceleration command, including separate travel commands.

## Deferred Behavior

This slice does not port Orca's full travel planner. The following remain deferred:

- `needs_retraction`, wipe disabling, and `AvoidCrossingPerimeters` detours from `GCode.cpp::travel_to`
- short-travel role-specific acceleration switching for external/overhang perimeters
- machine acceleration clamping through `machine_max_acceleration_travel`
- full jerk parity beyond Ares' existing `set_jerk_xy_with_comment`
- wipe tower, multi-extruder, support, and custom G-code post-processor interactions

## Acceptance Criteria

- Focused writer tests prove `marlin2` / `reprapfirmware` travel acceleration emits `M204 T`, print acceleration emits `M204 P`, and Repetier emits `M202` for travel plus `M201` for print.
- Focused slice tests prove `travel_acceleration` and `initial_layer_travel_acceleration` reach the expected separate travel command before travel moves for supported flavors.
- Existing Marlin legacy and Klipper acceleration tests continue to pass.
- `cargo nextest run` is used for all test runs.
- Full verification passes before commit: `cargo fmt --check`, targeted nextest, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, `git diff --check`, and touched Rust LOC guard.
