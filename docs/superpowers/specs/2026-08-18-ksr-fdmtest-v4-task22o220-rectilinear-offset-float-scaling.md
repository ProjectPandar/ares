# Spec: KSR FDM Test V4 task220 rectilinear offset float scaling

## Observable contract

Monotonic/rectilinear contour offsets preserve the source call site's
`float(scale_(...))` value at Ares' polygon-offset seam. The outer offset is
`scale(overlap - (0.5 - 0.45) * spacing)` and the inner offset is
`scale(overlap - 0.5 * spacing)`. Ares must not convert either value through
`CoordinateScale::checked_scale` before the existing `f32` offset interface;
that early integer truncation moves generated endpoints by one coordinate and
regresses the KSR golden prefix.

Values derive from effective fill flow spacing, overlap, density, and generated
surface geometry. Production code does not inspect fixture identity or known
coordinates.

## Upstream boundary

This slice rewrites OrcaSlicer 2.4.2
`src/libslic3r/Fill/FillRectilinear.cpp:2751-2775`, where the call site
explicitly evaluates both scaled offsets as `float`. The upstream
`ExPolygonWithOffset:391-421` wrapper subsequently carries `coord_t`, while the
Rust destination `crates/ares-core/src/fill/rectilinear/surface.rs` directly
owns an `f32` polygon-offset seam. Preserving the call-site float at that
different seam is an intentional compatibility adaptation: a follow-up
`f32 -> i64 -> f32` experiment reproduced the old `.03178` first divergence,
while direct `f32` advanced the normalized fixture prefix from line 1,682 to
1,855. No fixture-specific branch is introduced.

Deferred: remaining ant/rectilinear topology, arc numeric parity,
retraction/wipe parity, timing/M73, and later normalized G-code differences.
