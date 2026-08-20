# Spec: Task 22O242 rounded fitted-circle centers

## Observable contract

Arc fitting converts a floating fitted-circle center back to the scaled integer coordinate grid using nearest-integer rounding. KSR G2/G3 centers and segmentation therefore use the same integer centers as OrcaSlicer rather than centers biased toward zero.

## Upstream boundary

Port the conversion in `OrcaSlicer/src/libslic3r/Circle.cpp:43-53` through the `Point(double, double)` constructor in `Point.hpp:197`: both center coordinates use `std::round`. Destination: `project_slice/gcode_emit/motion/arc.rs`. Deferred: arc endpoint geometry, path ordering, and later G-code differences.
