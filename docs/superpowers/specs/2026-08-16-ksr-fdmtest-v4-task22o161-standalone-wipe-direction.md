# Spec: Task 22O.161 standalone extrusion wipe direction

## Observable contract

A standalone extrusion path loaded from the KSR project stores its completed path in reverse emission order for the next wipe. The first bottom-surface retract therefore wipes one millimetre back along the final diagonal to `X104.446 Y94.771`, and the following spiral lift uses the resulting position.

## Upstream boundary

This ports OrcaSlicer 2.4.2 `GCode.cpp:426-499,6103-6125`. `GCode::extrude_path` reverses a standalone path before `Wipe::wipe` replaces its first point with the current nozzle position and clips it to `wipe_distance`. Loop emission remains governed by `GCode.cpp:5979-5991` and retains forward loop-path storage.

Timing, object identifiers, sub-micron extrusion rounding, later path-order differences, cooling, and final statistics remain deferred parity slices.
