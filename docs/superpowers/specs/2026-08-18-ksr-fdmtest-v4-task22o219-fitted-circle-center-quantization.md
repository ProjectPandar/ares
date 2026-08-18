# Spec: KSR FDM Test V4 task219 fitted-circle center quantization

## Observable contract

Arc fitting computes a circle from three path points in the active scaled-coordinate space. The fitted center is converted to the integer point representation with truncation toward zero before polar, deviation, and emitted I/J calculations; radius remains the pre-conversion floating result, matching OrcaSlicer. Positive and negative fractional center coordinates therefore follow C++ integer conversion rather than nearest-grid rounding.

The calculation uses generated path points and `enable_arc_fitting` tolerance/radius rules. It does not depend on fixture identity or known G-code coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:16-55`, especially `new_circle.center = Point(center_x, center_y)` where `Point` owns integer `coord_t` values while `radius` retains the floating result. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/arc.rs::circle_from_three`.

Included: scaled fitted-center conversion semantics. Deferred: remaining arc candidate/range differences, monotonic sub-micron geometry, retraction/wipe parity, timing/M73, and later normalized G-code differences.
