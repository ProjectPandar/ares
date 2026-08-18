# Spec: KSR FDM Test V4 task220 rectilinear offset float scaling

## Observable contract

Monotonic/rectilinear contour offsets preserve the fractional scaled-coordinate value produced by the overlap and spacing formulas when passing the offset distance to polygon offsetting. The outer offset is `scale(overlap - (0.5 - 0.45) * spacing)` and the inner offset is `scale(overlap - 0.5 * spacing)`, each converted directly to `f32`; they are not first truncated to an integer coordinate. Generated endpoints therefore follow the same sub-micron offset geometry as OrcaSlicer.

Values derive from the effective fill flow spacing, overlap option, density, and generated surface. Production code does not inspect fixture identity or reference coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillRectilinear.cpp:2751-2775`, where `ExPolygonWithOffset` receives `float(scale_(...))` distances. The Rust destination is `crates/ares-core/src/fill/rectilinear/surface.rs::fill_monotonic_surface`.

Included: direct scaled-float conversion for rectilinear outer/inner offsets and resulting monotonic endpoints. Deferred: remaining ant/rectilinear topology, arc range differences, retraction/wipe parity, timing/M73, and later normalized G-code differences.
