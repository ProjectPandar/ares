# Spec: Task 22o.121 timelapse retraction, wipe, and eager lift

## Observable contract

With `retract_when_changing_layer`, the KSR layer tail retracts before its timelapse. The first layer emits the source-compatible `.11429` pre-wipe retraction, `.28571` wipe retraction at the current role speed, and static `G3 Z.6 I1.217 J0 P1 F60000` eager spiral lift before `M625`.

## Upstream boundary

Rewrite `OrcaSlicer/src/libslic3r/GCode.cpp:313-361,5527-5546` and `GCodeWriter.cpp:650-682`. Retraction during wipe is limited by retraction speed, available wipe distance, and active role speed. The timelapse eager lift uses the source static positive-I center and `atan(travel_slope_radians)` radius calculation.
