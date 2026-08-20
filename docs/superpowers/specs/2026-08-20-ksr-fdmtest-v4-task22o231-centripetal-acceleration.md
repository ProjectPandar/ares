# Spec: KSR FDM Test V4 task231 centripetal acceleration

## Observable contract

The G-code time planner limits XY cruise speed through a shallow direction change with the active print acceleration, even when the motion itself is travel and uses a larger travel acceleration for its acceleration/deceleration trapezoid. Both values come from typed 3MF printer options and emitted M204 state; no fixture identity or output constant enters production code.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode/GCodeProcessor.cpp:4029-4031` and `4387-4389`: `get_acceleration()` supplies centripetal acceleration, while each block retains its move-type-specific acceleration for trapezoid planning. The Rust destination is `project_slice/gcode_emit/processor/motion.rs` and `motion/planner.rs`.

## Included behavior

- Preserve print and move-type acceleration separately on each parsed motion block.
- Use print acceleration only for the shallow XY turn radius speed ceiling.
- Continue using travel, retract, or print acceleration for the block trapezoid.

## Deferred behavior

Float-width parity, source planner batching, geometry, cooling, object labels, and all unrelated G-code processor commands remain deferred source-cited slices.

## Acceptance

A focused processor test distinguishes a 500 mm/s² print acceleration from a 10,000 mm/s² travel acceleration on a travel block. The complete KSR fixture is regenerated and its timing, progress count, structural counts, and next normalized divergence are recorded. Focused tests, rustfmt, and Clippy pass.
