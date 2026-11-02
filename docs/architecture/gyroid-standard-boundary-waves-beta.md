# Standard gyroid boundary waves in beta

## Status and source boundary

Investigation and public RED qualification only. No production port is included
or authorized by this record. Start from main `f4ea41332`.

Upstream revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.
The owning unit is `src/libslic3r/Fill/FillGyroid.cpp`:
`f`, `make_one_period`, `make_wave`, `make_gyroid_waves`, and the standard
parametric branch of `FillGyroid::_fill_surface_single`.
`FillGyroid.hpp` supplies `CorrectionAngle = -45`, `DensityAdjust = 2.44`,
`PatternTolerance = 0.2`, and origin alignment.

Rust destinations are `crates/ares-core/src/fill/gyroid.rs` and
`crates/ares-core/src/fill/gyroid/waves.rs`. Existing width/height-based wave
placement is a temporary compatibility shell around the upstream gyroid concept,
not an Ares-owned pattern design. A future approved rewrite would replace it,
not retain it as a second implementation or fallback.

## Source comparison

| Boundary | Beta source | Current Rust |
| --- | --- | --- |
| Grid and extent | Keep contour bounds; compute a separate grid-aligned origin with the established ten-spacing shift; expand bounds by `spacing * multiline`. | Replace the minimum with the shifted grid origin, grow the maximum by ten spacings, and derive integer-quotient width/height plus one. |
| Period sampling | Always sample the complete `2*pi` period; refine by midpoint cross-product tolerance. | Limit the initial period to `min(2*pi, width)` before refinement. |
| Longitudinal wave coverage | Start at the sample at or before `x_min`; stop at the first sample at or after `x_max`; repeat complete periods. | Start from zero, repeat to width, and sometimes append an exact-width endpoint. |
| Transverse coverage | Compute both sampled periods' extrema, then choose integer wave indices covering the actual bounds. | Start at fixed lower offsets and use width/height-based upper limits. |
| Coordinates | Preserve the wave ordinates, swap axes for vertical waves, truncate scaled points, then translate by the grid origin. | Clamp ordinates into `[0, height]` before conversion and translate the generated set afterward. |
| Tail | Multiline offset, ExPolygon crop, minimum-length removal, base connection, inverse rotation. | The analogous stages already exist; do not assume a tail defect from a wave mismatch. |

The scalar-field equation, density adjustment, tolerance cap, and adaptive
midpoint criterion are not independently redesigned. Current Rust comments
about width/height and ten-spacing bounds describe an older source algorithm;
they are not evidence of parity with the pinned beta implementation.

Static differences do not prove a public regression. In particular, many extra
samples or waves lie outside the eventual contour and are discarded by crop.
Earlier long/shifted and slanted multiline candidates did not discriminate and
are not evidence for a production repair. A generic full-output failure or a
footer-only difference is insufficient for this investigation.

## Discriminating real fixture

Use a real two-island cylindrical-prism 3MF, not a rectangle or a mocked fill
surface. The archive's historical `cube10.stl` member name does not describe its
geometry: each closed 32-sided cylinder has 66 vertices, diameter 12 mm and
height 6 mm. Copy that actual mesh twice, retaining its facet topology per
copy, into one object/volume: local offsets `(53, 47)` and `(-53, -47)` give
132 vertices and 256 facets. Freeze standard gyroid at 1%, one sparse line,
one wall, no top/bottom shells, no support, no arc fitting, and no infill
boundary anchors.

`PrintObject.cpp:158-159,177-179,885-890` centers the transformed mesh for slicing
and carries the offset in instance shifts. Changing instance placement alone
does not select a filler-local bounding box. The first single-cylinder
candidate, moved only through its instance transform, produced no Native
sparse paths: it failed its Native premise and is explicitly not a RED.
Preserve that rejected input and Native replay separately.

Use the separate local islands and low density to make a component's current
longitudinal extent shorter than a full period while its contour crosses the
truncated final sampling interval. A complete beta period and the truncated
Rust period can then refine different samples inside the contour, rather than
merely creating different discarded outer margins. This is a source-guided
candidate, not a qualified witness until runtime evidence exists.

The public test must load the frozen 3MF bytes, run Native twice at runtime,
assert standard gyroid and actual sparse infill, then use the existing complete
ordered Native/API/CLI comparator without changes. Retain input, producer,
Native replay, API/CLI outputs and exhaustive differences as repeatable external
artifacts. Preserve feeds, M73, comments, metadata and footer in the verdict.

Diagnostics must establish a sparse-infill geometric difference, not just the
first global M73 byte difference or the separately owned footer residuals.
API/CLI raw equality and Native replay equality remain independent gates.
If the candidate is nondiscriminating, record that result rather than promote
it to a boundary-specific RED. No production edit or diagnostic replacement of
the standard algorithm is permitted during qualification.

## Smoothing and ownership exclusions

The standard `_fill_surface_single` path calls `multiline_fill` and
`Fill::chain_or_connect_infill`, not `FillCornerSmoothing`.
`FillBase.cpp::multiline_fill` uses multiline offsets; its count-one branch
returns immediately. `chain_or_connect_infill` selects chaining or connection,
not the corner-smoothing unit. No corner-smoothing change belongs to this slice.
If a further source call edge to `FillCornerSmoothing` is found, stop and flag it
before expanding scope.

Do not modify `Fill3DHoneycomb`. Adaptive gyroid, `gyroid_optimized`, marching
squares and optimized raster margins are adjacent, deferred behavior. Their
existing shared setup must not be silently changed by a future standard-branch
port. Also deferred: multiline offset internals, base joining/connection,
Clipper internals, surface preparation, flow, writer/time calculation, and
preset/footer supply. Any remaining independent difference requires its own
source boundary and review.

## Measured first difference and approved port

The paired-cylinder public RED first differs in sparse extrusion at line 105.
A read-only Native capture of `make_gyroid_waves` records complete odd/even
period vectors; a temporary Rust observation records the corresponding vectors.
Their phase and tolerance match exactly, as do both period prefixes through π.
Native ends the period at 2π; Rust ends it at 4. The first even knots after π
are respectively `π + π/32` and `π + (4 - π)/8`. Applying the source vertical
swap, coordinate cast, grid origin and instance shift maps those measured knots
to the differing public endpoints. Thus period truncation explains the first
difference; no flow or corner-smoothing repair is implicated. Optimized-out
Native variables are not treated as captured values.

The approved rewrite is limited to the standard branch's separate grid origin
and spacing/multiline-inflated contour bounds, full-period construction,
longitudinal bracketing without ordinate clamping, and extrema-indexed wave
selection. Preserve the optimized branch's current bounds and behavior; do not
extend this approval to its adjacent upstream changes. Keep the scalar field,
multiline/crop/length-filter/connection tail and inverse rotation unchanged.
Remove temporary observations and verify rebuilt complete-byte restoration.
Freeze the existing gyroid test IDs before implementation, then rerun exactly
that inventory. Full-output failures remain authoritative, including any
remaining metadata/footer dependency.

The paired-cylinder fixture passes the unchanged complete ordered Native/API/CLI
comparison after this port, with sparse extrusion counts matching at 176.
The frozen three-test gyroid inventory improves from zero passes to one, with
zero PASS-to-FAIL transitions. Both existing ironing gyroid tests still fail;
this is not an all-gyroid parity claim. An inventory with no prior passes does
not demonstrate preservation of positive coverage.

## Review gate

Submit this ARD and a genuinely discriminating public RED before a production
port. Name the exercised upstream functions, distinguish measurements from
hypotheses, and disclose all remaining complete-output failures. Future GREEN
requires approved scope, source-local implementation, identical before/after
breadth with zero PASS-to-FAIL transitions, rustfmt and strict all-target Clippy,
changed Rust files below 400 lines, and full-output qualification without
borrowing another slice's limited GREEN exception.
