# Line fill adjusted spacing at the base extrusion boundary

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`Fill/FillLine.cpp:32–42`, `Fill/FillBase.cpp:131–149,174–215`.
Rust destination: `fill/line.rs::LineFill` and
`project_slice/fill_entities/line.rs::append`.

## Source argument and included behavior

At sparse density 100%, `params.density` is 1.0f. When `dont_adjust` is
false, `FillLine::_fill_surface_single` first computes the integer scaled
spacing, calls `_adjust_solid_spacing` on the rotated domain width, and
writes `this->spacing = unscale<double>(this->_line_spacing)` (line 42).
The assignment occurs independently of whether clipping produces paths.
`Fill::fill_surface` offsets once using the entry spacing, then calls the
same filler for every offset component. Each component therefore observes
the spacing left by the preceding component; the final spacing survives
the call. For density <= 0.9999f or dont_adjust=true, spacing is unchanged.

`Fill::fill_surface_extrusion` reads that final filler spacing both for
flow materialization and for the `2 * this->spacing` close-end join radius
at 213–215. Joining with the original group parameter is not equivalent:
solid-spacing adjustment can inflate spacing by up to 20%, admitting a
gap between the old and new radii. No claim is made that every 100% Line
surface contains such a discriminating gap.

The Rust result carries both raw polylines and final f64 spacing. Offset
components retain the sequential spacing update, and the base extrusion
adapter uses the returned spacing for joining and materialized flow.
Geometry-only callers consume the returned polylines without materializing
or adding close-end joins. This replaces the incomplete vector-only return
shell around the upstream mutable filler state, not a new fill policy.

## Deferred and verification

No changes to Line oscillation, connector predicates, angle selection,
LockedZag/concentric overrides, or complete-output comparators. This is a
source argument rather than a claimed new public 100% Line RED witness.
Validate the final tip against main `1cf2968d5` with the same authorized
421 fill/support selectors, reporting every FAIL-to-PASS and requiring
zero PASS-to-FAIL; also retain the five ordered witnesses, rustfmt and
zero-warning all-target Clippy. External evidence retained outside the
repository.
