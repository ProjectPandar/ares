# Spec: Task 22O.127 forward stored-path wipe

## Observable contract

When `filament_wipe` is enabled, retraction follows the stored extrusion path from its beginning after replacing that beginning with the actual clipped endpoint. For a seam-gapped loop this continues across the seam start in extrusion order, matching Orca instead of traversing the just-emitted path backward. `filament_wipe_distance`, `retract_before_wipe`, role-based wipe speed, and retraction speed continue to determine distance and E distribution.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:426-497` (`Wipe::wipe`) and `GCode.cpp:5978-5991` (stored loop path). The Ares destination is `crates/ares-core/src/project_slice/gcode_emit/motion/travel.rs::wipe_moves`; the emitter already stores its clipped path in source order.

Included: actual endpoint replacement, forward traversal from stored point one, distance clipping, and per-segment retraction distribution. Deferred: aligned seam placement, object IDs, cooling markers, timing, and later exact G-code differences.

## Acceptance

A focused wipe-path test fails against reverse traversal and passes with Orca's forward stored-path traversal. The first KSR outer-loop wipe moves toward the loop's first emitted segment rather than its terminal predecessor. Rust files remain below 400 LOC; rustfmt and strict `ares-core` Clippy pass.
