# Spec: Task 22O.251 outer-wall aligned seam split

## Observable contract

For the KSR fixture's first-layer rounded rectangular island, Ares emits the OrcaSlicer outer-wall travel destination and preserves the short first extrusion segment: `G1 X140.625 Y102.983 F60000` followed by `G1 X140.618 Y102.994 E.00049` after the outer-wall role setup. Values must be derived from aligned seam candidates and generated perimeter geometry.

## Upstream boundary

Port the applicable behavior from OrcaSlicer 2.4.2 `src/libslic3r/GCode/SeamPlacer.cpp:1547-1563` and `src/libslic3r/ExtrusionEntity.cpp:230-315`: aligned `Vec3f` seam scaling, closest segment projection, snap comparison, fitted-polyline split, and loop rotation. The Rust destination is `project_slice::seam_placement` and its spline seam input.

## Deferred behavior

Later first-layer geometry, extrusion, lifecycle, timing, progress, and metadata differences remain outside this slice.
