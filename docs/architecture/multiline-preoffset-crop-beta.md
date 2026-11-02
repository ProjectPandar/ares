# Beta multiline pre-offset centerline crop

## Upstream boundary

Rewrite source is Orca beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.
The four selected complete-output failures configure tri-hexagon with count 2,
regardless of their historical test module names. `FillStars::fill_surface`
(`src/libslic3r/Fill/FillRectilinear.cpp:3586–3601`) dispatches these to
`FillRectilinear::fill_surface_trapezoidal(..., 2)`, not
`fill_surface_by_multilines` (`3004–3054`).

The included operation is `FillRectilinear.cpp:3433–3437`: intersect generated
centerlines with `offset(expolygon, float(d1 / 2))` **before** calling
`multiline_fill(polylines, params, spacing)`. `d1` is computed at `3111` as
`coord_t(scale_(this->spacing)) * params.multiline`. `multiline_fill`
(`FillBase.cpp:2947–3000`) then expands the cropped centerlines. The later
half-spacing surface contraction and outline intersection (`3450–3457`) are
separate operations and do not replace this earlier crop.

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

Include only the source-ordered centerline crop in the shared trapezoidal tail,
using its already-rotated surface. Existing row generation, smoothing behavior,
outline start selection, source-local coverage, origin/center handling, and
connection traversal are outside this slice. Adjacent upstream differences are
not authorization to bundle changes. A remaining complete-output failure must
be reported rather than hidden by modifying the comparator or speculative fixes.

## Verification gate

1. Commit this ARD independently.
2. Freeze the four exact inputs with SHA assertions and run public API/CLI full
   ordered-output tests with deterministic double Native replay. Commit RED
   before production changes.
3. Port the single source-order crop, then require the four full outputs to pass
   before calling the change GREEN.
4. Rerun the same 13-case residual inventory before/after and report each
   transition; extend to the surrounding multiline family to measure breadth.
   Require zero PASS-to-FAIL in the measured inventory. Inherited failures remain
   failures; four related fixtures do not establish 13-case common causality.
5. Run rustfmt and all-target Clippy with warnings denied. Keep all diagnostic
   artifacts external. No additional Native debugger observations are allowed;
   standard Native parity tests remain authorized.
