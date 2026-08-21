# Spec: Task 22O.253 short outer-wall travel acceleration

## Observable contract

On KSR layer Z0.4, the short travel into an outer-wall path emits `M204 S5000` before `G1 X140.787 Y103.081 F60000`. The acceleration is selected from `outer_wall_acceleration` because the generated travel is shorter than `retraction_minimum_travel`; both values come from the loaded 3MF project.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:7350-7414`, especially the non-first-layer short-travel branches at lines 7380-7395 and acceleration emission before travel at lines 7409-7414. The Rust destination is `project_slice::gcode_emit::motion::{begin_object_travel,path}`.

## Deferred behavior

Travel jerk output, later executable-body divergences, timing, progress, and metadata remain outside this slice.
