# Spec: Task 22O.153 scaled object-origin arithmetic

## Observable contract

The first-layer retained fitted arc emits `G3 X104.96 Y100.092 I.232 J-5.372 E.031`, not `X104.961`. Coordinate rounding must follow an object origin quantized to the active Clipper coordinate scale before local scaled points are converted to G-code coordinates.

The origin is derived from the loaded 3MF instance/model transform and generated model bounds. Production code must not inspect fixture names, reference G-code, or known endpoint constants.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:5380-5386,5659-5668,8123-8135`: object instance shifts are stored as scaled `Point` coordinates, `set_origin` receives their unscaled values, and `point_to_gcode` adds an unscaled local point to that quantized origin before the three-decimal G-code formatter runs.

Included behavior is active-scale truncation and unscaling of the model center used as `EmitState::offset`. Multi-instance traversal, cooling, timing, object identifiers, and later G-code differences are deferred.
