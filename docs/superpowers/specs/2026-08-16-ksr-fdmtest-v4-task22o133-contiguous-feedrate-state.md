# Spec: Task 220.133 contiguous extrusion feedrate state

## Observable contract

Adjacent materialized extrusion fragments that share an endpoint and resolve to the same feedrate emit one `G1 F...` command. A travel, retraction/deretraction sequence, or changed resolved extrusion feedrate invalidates that continuity and requires the next extrusion feedrate command.

For the first KSR inner wall, the source-equivalent fragments therefore contain exactly one `G1 F3000` before the first outer wall. All values remain derived from path kinematics and effective project options; no fixture identity or reference output enters production.

## Upstream boundary

OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:6924-7010` resolves and emits one speed for each source extrusion path through `GCodeWriter::set_speed`. Ares may materialize that source path as multiple adjacent fragments, so `project_slice::gcode_emit::motion::emit_points` retains the effective extrusion feedrate across uninterrupted fragments and emits only the source-equivalent command sequence.
