# Spec: KSR FDM Test V4 task230 processor acceleration envelopes

## Observable contract

G-code time estimation clamps every `M204` print, retract, and travel acceleration to the corresponding effective machine envelope loaded from the 3MF. The emitted `M204` command remains unchanged; only processor timing, M73 placement, and printing-time metadata use the clamped state.

All limits come from typed `machine_max_acceleration_extruding`, `machine_max_acceleration_retracting`, and `machine_max_acceleration_travel` options. Production code does not infer limits from the fixture or reference timing.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode/GCodeProcessor.cpp:2108-2122`, `2439-2450`, and `5849-5887`: each time machine retains three scalar maximum accelerations and clamps later `M204` updates. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/processor.rs::ProcessorLimits` and `processor/motion.rs::MotionState`.

Included: normal-mode scalar acceleration initialization, legacy and modern M204 clamping, and KSR processor timing regeneration. Deferred: stealth timing, remaining planner float arithmetic, object-ID instability, geometry, cooling, and later exact G-code differences.
