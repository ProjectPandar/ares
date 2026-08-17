# Spec: Task 22O.145 forward stored-path wipe traversal

## Observable contract

When the 3MF enables filament wiping, Ares starts the first KSR post-loop wipe at the current clipped extrusion endpoint and then traverses the stored loop path forward from its second point. The first two wipe moves are exactly `G1 X140.618 Y102.994 E-.02125` and `G1 X140.353 Y103.632 E-.27626`, followed by a partial move to `X140.294 Y103.881`.

The behavior derives from the emitted loop geometry and typed `wipe`, `wipe_distance`, retraction, and wipe-speed options. Production code does not inspect fixture identity or reference G-code.

## Upstream boundary

Port OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:426-492,5978-5990`. `Wipe::wipe` replaces the stored path's first point with the current position and traverses the remaining points in their original forward order. Source integer endpoint precision, the following spiral-lift target, and subsequent exact G-code differences remain deferred source-cited slices.
