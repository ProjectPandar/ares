# Spec: KSR FDM Test V4 task224 rectilinear offset quantization

## Observable contract

Monotonic bottom-surface fill quantizes its overlap-derived outer and inner offsets to scaled integer coordinates before invoking polygon offsetting. The values remain derived from `infill_wall_overlap` and the selected extrusion spacing; fixture identity and reference G-code are not inputs.

For the KSR first-layer bottom surface, the source-order conversions produce outer and inner offsets `-22853` and `-228539` scaled units. This corrects task O220's fractional-offset interpretation. It does not by itself resolve the first KSR G-code divergence: adjacent upstream surface geometry still changes fitted lengths and centers.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillRectilinear.cpp:2756-2775` and `386-425`. `fill_surface_by_lines` converts each scaled overlap expression to `float`, then `ExPolygonWithOffset` receives it as `coord_t`; polygon offsetting therefore receives an integral scaled distance. The Rust destination is `crates/ares-core/src/fill/rectilinear/surface.rs::scaled_offsets`.

Included: source-order `f32` conversion followed by integer-coordinate truncation, and removal of the superseded fractional-offset assertion. Deferred: the adjacent surface-geometry difference responsible for the first KSR path mismatch, later rectilinear geometry, travel/wipe ordering, ant topology, cooling, timing/M73, and all later normalized G-code differences.
