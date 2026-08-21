# Spec: Task 22O249 arc simplification coordinate and direction parity

## Observable contract

Arc fitting simplifies retained wall paths on OrcaSlicer's integral coordinate grid and applies the same strict polar-angle direction predicates. A wrapped arc whose middle point lies exactly at polar angle zero remains linear instead of being accepted as a counter-clockwise arc. The KSR fixture's first-layer aligned seam advances to `G1 X122.022 Y94.872 Z.6` after progress commands and dynamic object IDs are excluded.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/ArcFitter.cpp:108-137`, `MultiPoint.cpp:164-205`, `Line.hpp:73-82`, and `Circle.cpp:326-390`. The Rust destination remains the private `project_slice::gcode_emit::motion::arc` module and its normal submodule; no fixture coordinates or output-specific branches enter production code.

## Included behavior

- Evaluate Douglas-Peucker tolerance and point-to-segment distances in the scaled integral coordinate domain used by OrcaSlicer paths.
- Preserve OrcaSlicer's strict zero and $2\pi$ bounds while deriving clockwise or counter-clockwise direction from start, middle, and end polar angles.
- Keep rounded fitted-circle centers and verify the resulting first-layer seam destination through `slice_project`.

## Deferred behavior

Wipe extrusion rounding, dynamic object IDs, progress/time processing, later-layer geometry, and unrelated fill-path ordering remain unchanged.
