# Spec: KSR FDM Test V4 task218 clipped-arc sweep containment

## Observable contract

Loop clipping retains a fitted G2/G3 segment only when the shortened polyline endpoint lies strictly inside the original directed arc sweep. A candidate behind the start, beyond the original end, at the start, or at the fitted center cannot expand or reverse the retained arc; that fitting range falls back to linear emission. A valid interior candidate continues through source-grid endpoint projection from task217.

The decision uses generated path points, retained arc center/radius/length/direction, and the project `enable_arc_fitting` option. It does not depend on fixture identity or output coordinates.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Circle.cpp:230-260`, where `ArcSegment::clip_end` calls `is_point_inside` before projection, and `Polyline.cpp:80-90`, where a failed clipped arc becomes `Linear_move`. The Rust destination is `crates/ares-core/src/project_slice/gcode_emit/motion/arc/retained.rs::clip_end`.

Included: strict directed-sweep containment before retained-arc endpoint projection and linear fallback. Deferred: remaining arc candidate/range differences, monotonic sub-micron geometry, retraction/wipe parity, timing/M73, and other normalized G-code divergences.
