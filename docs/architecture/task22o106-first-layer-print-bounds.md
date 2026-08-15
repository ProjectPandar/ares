# Task 22O.106: First-layer print bounds source bridge

## Upstream boundary

This slice follows `OrcaSlicer/src/libslic3r/GCode.cpp`'s runtime setup of
`first_layer_print_min`, `first_layer_print_max`, and `first_layer_print_size`
from the first-layer convex hull. Ares derives the initial runtime bounds from
resolved model-part vertices and instance/volume transforms in
`project_slice::gcode_emit`.

## Included and deferred

Included: typed project G-code receives model-space first-layer bounds without
fixture identification or reference-G-code input. Deferred: exact convex-hull
construction and extrusion-width expansion from Orca's `Print::first_layer_convex_hull`.
The current source boundary remains a compatibility shell until that upstream
hull behavior is ported.
