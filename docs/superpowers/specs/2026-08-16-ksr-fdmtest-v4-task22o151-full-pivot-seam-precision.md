# Spec: Task 22O.151 full-pivot aligned-seam precision

## Observable contract

The first KSR outer-wall loop ends with `G1 X140.645 Y102.949 E.02375`. Its following wipe ends with `G1 X140.294 Y103.881 E-.1025`. These values must come from the aligned seam, loop projection, scaled seam-gap clipping, and option-derived extrusion/retraction calculations; production code must not inspect fixture names or reference G-code.

The focused `slice_project` test is the public behavior seam. All changed Rust source files remain below 400 lines and pass Clippy and rustfmt.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2:

- `src/libslic3r/Geometry/Curves.hpp:61-175` — weighted cubic B-spline fitting through `Eigen::FullPivHouseholderQR`;
- `src/libslic3r/GCode/SeamPlacer.cpp:1547-1628` and `src/libslic3r/Point.cpp:106-128` — f32 seam coordinates, scaled-coordinate truncation, and integer segment projection;
- `src/libslic3r/Polyline.cpp:683-703` — seam-gap clipping in scaled coordinates before G-code conversion.

Included behavior is full-pivot Householder least-squares fitting, source-order f32 seam depth arithmetic, integer projection/casting, and scaled path clipping. Rank-policy generalization, other seam modes, cooling, timing, and later G-code differences are deferred.
