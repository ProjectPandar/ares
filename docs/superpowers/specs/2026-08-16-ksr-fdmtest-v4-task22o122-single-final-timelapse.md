# Spec: Task 22o.122 single final-layer timelapse

## Observable contract

The KSR project emits its configured `time_lapse_gcode` exactly once for each of the 460 printed layers. The final layer is rendered through the same per-layer path as every earlier layer and is not rendered again during export finalization.

## Upstream boundary

This ports the `has_insert_timelapse_gcode` final-layer behavior in `OrcaSlicer/src/libslic3r/GCode.cpp:5208-5522`. Ares keeps the existing project-derived template renderer and removes only its obsolete second finalization call.
