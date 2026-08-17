# Spec: Task 22O.142 inner-path aligned seam projection

## Observable contract

For `seam_position=aligned`, an extrusion loop whose first path has the `Perimeter` extrusion role is projected from the selected external seam onto that inner-wall loop regardless of its nesting/orientation loop role. The first KSR inner-wall start travel is exactly `G1 X140.158 Y102.797 F60000`; the following outer-wall seam remains unchanged.

All coordinates derive from generated loops, seam candidates, typed project options, and model geometry. Production code does not inspect fixture identity or reference G-code.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode/SeamPlacer.cpp:1562-1600` and `src/libslic3r/ExtrusionEntity.hpp:485`. `ExtrusionLoop::role()` returns its first path's extrusion role; only `erPerimeter` receives inner-wall projection. Concave-corner projection already exists in Ares and remains active through this corrected role seam. Exact loop-end clipping, staggered inner seams, non-aligned seam modes, cooling, timing, and later G-code differences remain deferred.
