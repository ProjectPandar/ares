# Spec: Task 22O.147 lifted-travel modal feedrate

## Observable contract

After the first KSR spiral lift, the consuming move is exactly `G1 X145.539 Y95.848 Z.6`. It omits `F60000` because the immediately preceding spiral-lift command already established that modal feedrate. Ordinary travels that did not just emit a retract lift continue to carry their required feedrate.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/GCodeWriter.cpp:713-769`, where `travel_to_xyz` emits the spiral lift followed by an XYZ formatter move without redundantly resetting an unchanged feedrate.

Deferred behavior: fitted-arc grouping beginning at this perimeter, cooling, timing, and later exact G-code differences.

## Acceptance

The focused KSR inter-path test asserts the exact modal XYZ line. The existing next-layer lifted-travel test remains green, proving that a lift retained from lifecycle G-code is not treated as a new retract lift.
