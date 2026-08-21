# Spec: Task 22O.252 proportional wipe retraction arithmetic

## Observable contract

For the first KSR layer's three-segment wipe after the matching rounded outer wall, Ares emits retractions `E-.20877`, `E-.18094`, and `E-.01028` at the fixture-derived endpoints. Retraction is distributed from the configured retraction length, wipe speed, wipe distance, retained fitted-circle radius, and generated segment lengths; no fixture constants enter production code.

## Upstream boundary

Port the applicable behavior from OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:426-493`, especially `Wipe::wipe` line 479: `length * (segment_length / wipe_dist)`, plus `src/libslic3r/Circle.hpp:24-27` and `src/libslic3r/Polyline.cpp:939-1005`, which retain the fitted circle radius while projecting a seam-split arc endpoint. The Rust destinations are `project_slice::gcode_emit::motion::{arc,travel}` and `project_slice::seam_placement::fitting`.

## Deferred behavior

Subsequent first-layer motion, lifecycle, timing, progress, and metadata differences remain outside this slice.
