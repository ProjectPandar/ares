# Spec: Task 22O.144 pre-seam arc-range simplification

## Observable contract

With 3MF-resolved `enable_arc_fitting = true` and `spiral_mode = false`, Ares simplifies complete extrusion paths before seam selection and retains fitted range boundaries while emitting the split path. The first KSR outer wall therefore keeps the source boundary move `G1 X140.618 Y102.994 E.00049` rather than collapsing it into the following line range.

The result derives from generated path geometry plus the typed `enable_arc_fitting`, `spiral_mode`, and `resolution` options. Production code does not inspect fixture identity or reference G-code.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/LayerRegion.cpp:1055-1126`, `src/libslic3r/Polyline.cpp:152-157,732-738`, and `src/libslic3r/Circle.cpp:308-488`. Region paths are arc-fitted and simplified before seam placement; G-code emission consumes the retained fitted ranges instead of simplifying the seam-split path a second time. Sparse-infill-specific tolerance, arc clipping metadata, wipe traversal, and subsequent exact G-code differences remain deferred source-cited slices.
