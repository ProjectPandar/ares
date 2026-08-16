# Spec: Task 220.132 emitted-loop wipe path

## Observable contract

A wipe starts at the seam-clipped extrusion endpoint and continues around the just-emitted loop from its beginning for `wipe_distance`. The stored wipe path contains post-arc-fitting segment endpoints across every materialized subpath in the loop, rather than only raw points from its final subpath. Retraction is apportioned by each consumed distance. The first KSR wipe therefore begins at `X140.618 Y102.994`, matching OrcaSlicer.

The path, distance, wipe speed, retraction length, and retract-before-wipe ratio all come from generated extrusion state and effective project options. No fixture identity or reference coordinates enter production.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:310-358,7400-7448` wipe-path retention into `project_slice::gcode_emit::motion` and `motion::loop_paths`. Include the complete emitted loop and fitted segment endpoints; retain the existing distance clipping, proportional E retraction, and wipe markers.
