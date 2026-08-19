# Spec: KSR FDM Test V4 task225 G28 processor motion

## Observable contract

The time processor treats Orca's `G28` homing command as a synthetic absolute `G1` move to zero for each requested axis, or all XYZ axes when no axis is specified. The synthetic move uses the current feedrate, positioning mode, and acceleration/axis limits, and updates logical position for subsequent estimates.

The behavior is derived from emitted G-code and does not inspect fixture names or expected output.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode/GCodeProcessor.cpp:4915-4939`, where `process_G28` rewrites the requested homing axes to zero and routes the result through `process_G1`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/processor/motion.rs::MotionState::motion`.

Included: axis-specific and all-axis synthetic motion. Deferred: Orca's full G-code line ID/cache attribution, machine-specific homing semantics, delay attribution, and remaining timing/G-code differences.
