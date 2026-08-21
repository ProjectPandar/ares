# Spec: Task 22o250 fitted-arc split projection arithmetic

## Observable contract

When an option-driven fitted perimeter is split at an aligned seam, Ares projects the split point onto the retained fitted circle with OrcaSlicer's integral coordinate arithmetic. The KSR first-layer second inner-perimeter wipe therefore emits `G1 X122.305 Y95.287 E-.04452`; the retraction remains derived from the projected path length and the 3MF `retraction_length`, `retract_before_wipe`, and `wipe_distance` options.

## Upstream boundary

Port `OrcaSlicer/src/libslic3r/Circle.hpp:24-27` (`Circle::get_closest_point`) as consumed by `Circle.cpp:221-236` (`ArcSegment::clip_start` / `clip_end`) and `Polyline.cpp:939-1005` (fitted-result splitting). Orca normalizes the integral point-to-center vector, truncates the scaled radius vector to `coord_t`, and only then adds the integral circle center. Ares must preserve that order rather than rounding the final floating-point coordinate.

## Included behavior

- Integral-coordinate fitted-circle endpoint projection during seam splitting.
- Exact first-layer KSR wipe retraction formatting through the existing G-code seam.

## Deferred behavior

Later geometry, extrusion, lifecycle, timing, and metadata differences remain outside this slice.
