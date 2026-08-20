# Spec: Task 22O.247 arc-center Point constructor parity

## Observable contract

Fitted arc centers use OrcaSlicer’s two-`double` `Point` constructor semantics: each finite scaled center coordinate rounds to the nearest integral coordinate before retained G2/G3 emission. The conversion is geometry-generic and does not inspect fixture names, reference G-code, or known output coordinates.

## Upstream boundary

Rewrite OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:48-53` together with the selected `Point(double, double)` overload in `src/libslic3r/Point.hpp:194-203`, at `project_slice::gcode_emit::motion::arc::circle_from_three`.

Included: nearest-integral conversion of the computed circle center. Deferred: the KSR fill-path coordinate divergence, timing/M73, and complete byte parity.

## Acceptance

A focused arc-fitting test distinguishes nearest-coordinate rounding from truncation for positive and negative center coordinates. Changed Rust files remain below 400 lines and pass focused nextest, clippy, and rustfmt before this corrective slice is committed and pushed independently.
