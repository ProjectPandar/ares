# Spec: Task 22O.246 arc-center coordinate conversion

## Observable contract

Fitted arc centers use OrcaSlicer’s integral `Point` conversion: finite circle-center coordinates are truncated toward zero on the scaled coordinate grid before they are retained for G2/G3 emission. The conversion is geometry-generic and does not inspect fixture names, reference G-code, or known output coordinates.

## Upstream boundary

Rewrite OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:16-53`, specifically `Circle::try_create_circle` assigning the computed `double` center through integral `Point(center_x, center_y)`, at `project_slice::gcode_emit::motion::arc::circle_from_three`.

Included: scaled integral center conversion after the three-point circle calculation. Deferred: the remaining KSR fill-path coordinate divergence, timing/M73, and complete byte parity.

## Acceptance

A focused arc-fitting test distinguishes truncation from nearest-coordinate rounding for positive and negative center coordinates. Changed Rust files remain below 400 lines and pass focused nextest, clippy, and rustfmt before this slice is committed and pushed independently.
