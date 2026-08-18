# Spec: KSR FDM Test V4 task222 fitted-circle Point constructor rounding

## Observable contract

A circle fitted from three scaled path points converts its floating center through OrcaSlicer's `Point(double, double)` constructor. Both center coordinates use `std::round` before returning to millimeters, including half-away-from-zero behavior; they are not truncated. The fitted radius remains the pre-conversion floating result.

The calculation uses generated path points and effective arc-fitting tolerance/radius rules. It does not depend on fixture identity or known G-code coordinates.

## Upstream boundary

This slice corrects task219 by reading the complete constructor boundary: OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:43-53` assigns `Point(center_x, center_y)`, and `src/libslic3r/Point.hpp:187-203` defines the selected double constructor with `std::round`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/arc.rs::circle_from_three`.

Included: nearest-integer fitted-center conversion and correction of obsolete task219 truncation documentation. Deferred: remaining arc candidate/range differences, rectilinear geometry, retraction/wipe parity, timing/M73, and later normalized G-code differences.
