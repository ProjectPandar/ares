# Marlin positive junction-deviation planner parity

## Upstream boundary

This slice ports the OrcaSlicer beta `GCodeProcessor` Marlin positive-junction-deviation path into `ares-core`'s `RollingPlanner::prepare`.

Upstream sources:

- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:7802-7804` — Marlin junction-deviation configuration.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:7809-7820` — junction acceleration.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:7858-7896` — positive-junction-deviation maximum junction speed.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5544-5564` — `process_G1` junction-deviation dispatch and safe feedrate.
- `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp:5644-5645` — allowable speed and entry speed.

## Destination

The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/processor/motion/planner/RollingPlanner::prepare` and its planner helpers. Existing motion parsing supplies the block flavor, junction-deviation, direction, axis-feedrate, acceleration, jerk, and feedrate inputs.

## Decisions

When the active flavor is Marlin firmware and junction deviation is positive, the planner follows the upstream junction-deviation path. A move without a previous move starts from zero safe feedrate and zero junction speed. Subsequent moves use the upstream junction acceleration, XYZE junction geometry, nominal-feedrate caps, and safe-feedrate threshold.

When junction deviation is zero or negative, the existing classic-jerk path remains unchanged.

The port is a source-cited rewrite slice, not a new Ares planning design. Geometry generation, G-code formatting, delay ownership, Klipper planning, and unrelated motion behavior are outside this boundary.
