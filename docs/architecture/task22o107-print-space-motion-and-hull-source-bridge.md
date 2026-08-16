# Task 22O.107: Print-space motion and first-layer hull bridge

## Upstream boundary

This slice follows `OrcaSlicer/src/libslic3r/GCode.cpp`'s use of
`print.translate_to_print_space` for first-layer runtime placeholders and the
G-code writer's conversion of centered object paths into print-space motion.

## Included and deferred

Included: Ares derives the model center from transformed 3MF model-part
vertices, translates centered extrusion paths into print space, and uses the
same offset for first-layer placeholder bounds. Deferred: the complete Orca
`Print::first_layer_convex_hull` union of first-layer islands, skirt, brim, and
wipe-tower polygons. No fixture-specific output or reference G-code is read.
