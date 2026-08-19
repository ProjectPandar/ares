# Spec: KSR FDM Test V4 task224 processor machine envelopes

## Observable contract

The post-generation time processor applies emitted `M201` per-axis maximum acceleration and `M203` per-axis maximum feedrate values to subsequent G0/G1/G2/G3 blocks. Positive and negative extrusion-only moves use retract acceleration. Adjacent collinear motion blocks retain speed at their shared junction instead of stopping because of an off-by-one planner entry.

The values originate in the effective project options and the generated machine-envelope prologue. The processor does not inspect fixture identity or reference G-code.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode/GCodeProcessor.cpp:3964-4184`, `5160-5204`, and `src/libslic3r/GCode/GCodeProcessor.hpp:424-477`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/processor/motion.rs`, split from `processor.rs` to keep both implementation files below 400 LOC.

Included: M201/M203 parsing, per-axis feedrate and acceleration clamping, extrusion-only retract acceleration, and correctly indexed shared-junction entry speed. Deferred: exact Orca safe-feedrate junction calculation, planner queue flushing, G28 synthetic moves, delay attribution, preparation/first-layer accounting, and later normalized G-code differences.
