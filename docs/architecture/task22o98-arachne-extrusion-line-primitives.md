# Task 22O.98 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port the rendering-neutral Arachne extrusion-line primitives from pinned
OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/Arachne/utils/ExtrusionJunction.hpp`;
- `src/libslic3r/Arachne/utils/ExtrusionLine.hpp`;
- `src/libslic3r/Arachne/utils/ExtrusionLine.cpp:21-275`.

The crate-private `ares-core::arachne` boundary owns scaled junction position,
width and perimeter identity; line inset/odd/closed metadata; source-order
mutation, length, polygon and thick-polyline conversion; clockwise contour and
signed-area classification; and source simplification including five-micron,
shape-error and extrusion-area guards. Operations take the active
`CoordinateScale` where upstream relies on its global scale.

This milestone is an inactive prerequisite for `WallToolPaths` and
`FillConcentricInternal`. It does not port the C++ `ExtrusionPaths` adapters,
the free closed-line `to_polygon` endpoint-stripping helper, the free
`to_points` helper, Arachne half-edge/skeletal trapezoidation, beading
strategies, wall toolpaths, variable-width entity conversion, concentric fill
materialization, motion, or G-code. No fixture branch or legacy fallback is
introduced.

Ten focused tests cover payload mutation, per-segment integer length and width
layout, orientation/area, integer weighted-width deviation, both coordinate
scales, closed spill/closure repair, replacement intersections, overflow
rejection, and simplification guards. Focused Nextest, rustfmt, strict core
Clippy, diff, macro, and LOC gates pass. Implementation and test shards are
ordinary 336/208-LOC modules.
