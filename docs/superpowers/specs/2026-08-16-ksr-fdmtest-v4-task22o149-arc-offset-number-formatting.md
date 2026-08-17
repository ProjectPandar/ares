# Spec: Task 22O.149 arc-offset number formatting

## Observable contract

Retained G2/G3 center offsets use G-code offset formatting: an exact zero remains `0`, while nonzero magnitudes below one omit the leading zero. The first affected KSR command is exactly `G2 X155.758 Y90.456 I-6.194 J.091 E.09765`.

## Upstream boundary

This slice ports the I/J word formatting used by OrcaSlicer 2.4.2 `src/libslic3r/GCode.cpp:7080-7110` and its `GCodeG2G3Formatter` path. It applies only to arc-center offsets; X/Y axes retain ordinary coordinate formatting.

Deferred behavior: later perimeter ordering and geometry, cooling, timing, and remaining byte differences.

## Acceptance

The focused KSR test asserts the exact positive sub-unit J word, existing formatter unit tests remain green, and the changed core crate passes rustfmt and Clippy.
