# Spec: KSR FDM Test V4 task221 rectilinear rotation half-up rounding

## Observable contract

Rectilinear input contours and emitted fill polylines convert rotated floating coordinates to integer points with OrcaSlicer's `fast_round_up` rule: `floor(value + 0.5)`, including ties toward positive infinity (`-0.5 -> 0`, `-1.5 -> -1`). The source special case `0.49999999999999994 -> 0` is retained. Rust's default `f64::round`, whose negative ties round away from zero, is not used at these two rotation seams.

Coordinates derive from generated surface points and effective infill direction options. Production code contains no fixture-specific coordinates or output branches.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Point.cpp:50-57` and `src/libslic3r/libslic3r.h:403-420`, called by `Fill/FillRectilinear.cpp:399-405,2909-2913`. The Rust destinations are `crates/ares-core/src/fill/rectilinear/segments.rs::rotate_expolygon` and `surface.rs::rotate_polyline`, sharing the rounding primitive in `rectilinear.rs`.

Included: exact source rounding for forward and reverse rectilinear rotation. Deferred: remaining ant/rectilinear topology, arc numeric/range parity, retraction/wipe parity, timing/M73, and later normalized G-code differences.
