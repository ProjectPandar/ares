# Beta multiline pre-offset crop and outline start order

## Upstream boundary

Rewrite source is Orca beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.
The four selected complete-output failures configure tri-hexagon with count 2,
regardless of their historical test module names. `FillStars::fill_surface`
(`src/libslic3r/Fill/FillRectilinear.cpp:3586–3601`) dispatches these to
`FillRectilinear::fill_surface_trapezoidal(..., 2)`, not
`fill_surface_by_multilines` (`3004–3054`).

The included contiguous block is `FillRectilinear.cpp:3429–3447`, in order:

1. `smooth_polylines_corners(polylines, params.corner_smoothing())`.
2. Intersect centerlines with `offset(expolygon, float(d1 / 2))`.
3. `multiline_fill(polylines, params, spacing)` (`FillBase.cpp:2947–3000`).
4. For each closed outline of more than three points, remove its repeated end,
   rotate to the first vertex minimizing the dot product with `row_dir`, then
   append the first point again. Strict comparison preserves the first tied
   minimum, matching `std::min_element`.

`d1` at `3111` is `coord_t(scale_(this->spacing)) * params.multiline`.
`row_dir` is `(cos(angle), sin(angle))` for triangular-family patterns, where
`angle = (infill_layer_id % 3) * 2.0 * M_PI / 3.0` (`3134–3137`), not the
base angle used to rotate the surface. Grid uses the Y unit vector on odd
infill layers and the X unit vector on even layers. Carry this direction from
the row producer into the shared tail; do not infer it from an outline.

The later half-spacing contraction and outline intersection (`3450–3457`) are
separate operations and do not replace the earlier crop or outline reanchoring.

Rust destination is `crates/ares-core/src/fill/trapezoidal_tail.rs::finish`.
Its existing multiline-offset-first scaffold is a temporary compatibility shell
around this upstream tail, not an independently designed Ares fill pipeline.
Use existing geometry wrappers; do not alter their rounding or implementation.

## Localization and non-negotiable decisions

On the unmodified `9f6ee320d` base, the frozen S2 input's complete CLI output
first differs at line 15109: Native `X94.578`, Rust `X94.579`, with the same Y
and E. No coplanar prerequisite is needed to reproduce this first difference.

A temporary Ares-only, non-mutating parallel calculation of the missing source
crop changes the target post-outline-intersection point from
`(-10741733, -32551026)` to `(-10741732, -32551027)`. Existing rotation by
`3.9269908229496817` and translation by `(110,110)` put these at approximately
X `94.578500844` and `94.578499430`, respectively, on opposite sides of the
existing three-decimal formatting threshold. This is operation-order evidence,
not a captured Native intermediate or proof that all 13 failures share a root.
The probe did not change actual output and must not be committed.

Preserve source arithmetic: truncate scaled spacing before multiplying by the
multiline count; divide integer `d1` by 2 before converting the crop delta to
float. Keep multiline float spacing/ring arithmetic, rotation, and final output
formatting unchanged. Do not add epsilon, coordinate snapping, decimal rounding,
feed/M73 filtering, or a new clipping backend to fit the witness.

## Included and deferred behavior

The initially attempted crop-only port failed all four complete-output tests
and changed the first sparse motion. It was rejected and restored, not committed
as GREEN. The approved replacement ports the contiguous block above as one unit.

The user explicitly limited this slice to the verified `smooth_factor = 0`
path: `FillBase.hpp:101–104` defaults the factor to zero and
`FillCornerSmoothing.cpp:296–302` returns immediately for a sanitized zero factor.
All four frozen inputs use that default. Smoothing is therefore a source no-op
before the crop in this slice, not a stub implementation of an enabled smoother.
Nonzero smoothing, its reached helper and configuration projection require a
separate source-cited rewrite and public RED; this slice does not claim support.

Existing row generation, source-local coverage, origin/center handling, and
connection traversal remain outside the slice. Do not bundle adjacent upstream
differences. A remaining complete-output failure must be reported rather than
hidden by modifying the comparator or speculative fixes.

## Verification gate

1. Commit this ARD independently.
2. Freeze the four exact inputs with SHA assertions and run public API/CLI full
   ordered-output tests with deterministic double Native replay. Commit RED
   before production changes.
3. Port the approved contiguous block on the zero-factor path. If all four
   complete outputs still fail, report the first evidenced divergent step rather
   than guessing another repair. The accepted ruling below records the explicit
   exception for the separately owned footer prerequisite; it does not change
   the complete-output tests or comparator.
4. Rerun the same 13-case residual inventory before/after and report each
   transition; extend to the surrounding multiline family to measure breadth.
   Require zero PASS-to-FAIL in the measured inventory. Inherited failures remain
   failures; four related fixtures do not establish 13-case common causality.
5. Run rustfmt and all-target Clippy with warnings denied. Keep all diagnostic
   artifacts external. No additional Native debugger observations are allowed;
   standard Native parity tests remain authorized.

## Accepted GREEN ruling and outstanding complete-output RED

The user accepted the contiguous block port as GREEN after reviewing the four
cases' exact ordered raw motion and M73 equality. This is an explicit acceptance
of this source rewrite slice, **not** a claim that the four full-output tests
pass. Those tests remain enabled and RED until the preset-refresh slice supplies both footer keys:
`compatible_process_expression_group` and `pellet_flow_coefficient`.

For all four frozen inputs, Native and Ares have identical ordered raw G0/G1/G2/G3
and M73 records, with no feed, coordinate, role, order, or M73 normalization.
The three S2 cases each have 16,566 such records on both sides; H2 has 17,407.
Ares public API and CLI outputs are byte-identical in all four cases.
Exhaustive raw line comparison finds only the existing permitted generator-line
replacement and these two absent config-footer records:

```text
; compatible_process_expression_group = ""
; pellet_flow_coefficient = 0.4157
```

The unchanged full comparator first fails at line 17051 for the three S2 cases
and line 17980 for H2. No fill-block/body difference remains before the footer.
The same 100-case breadth inventory is 5 PASS / 95 FAIL before and after, with
zero PASS-to-FAIL, zero FAIL-to-PASS, and all 13 selected complete-output cases
still FAIL. These are complete-output verdicts, not motion-only acceptance
substitutes. Rustfmt and all-target Clippy with warnings denied pass.

Do not add a footer overlay, weaken the comparator, ignore tests, or implement a
local footer workaround. Keep this change on the multiline branch; the approved
bridge-limiting-offset branch is independent and unchanged. The separately
owned footer prerequisite must land before rerunning and claiming full-output
GREEN for these tests. Nonzero smoothing remains outside this accepted slice.
