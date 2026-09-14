# Orca Lattice Parity Milestone (4096/mm SCALING_FACTOR)

Status: PROPOSED — planned, not yet scheduled. This spec records the
investigation results that motivate the milestone and the staged plan.

## Motivation (evidence, 2026-09-14/15)

Three divergent buckets trace to coordinate-lattice differences between
ares and OrcaSlicer 2.4.2:

1. `top_surface_pattern=octagramspiral` fill ORDER: the emission reorder
   (validated as an exact port of upstream's v1 greedy) flips tie
   decisions because ares's geometry ints (1e6/mm, object-centered)
   differ from orca's (4096/mm, world-absolute) by ±1-2 units. The
   greedy compares squared distances on exact ints; sub-µm input noise
   flips equal/near-equal comparisons (probe: 4096-rounded inputs match
   orca's first six fragment picks, then flip at position 9).
2. Loop geometry ±1 scaled-int cases (smoke sweep "other" bucket).
3. Elephant-foot ±1 µm corner cases (E3NG family).

## Root findings

- `libslic3r.h:94-95`: `#define scale_(val) ((val) / SCALING_FACTOR)` —
  a pure division MACRO returning double. `SCALED_EPSILON =
  scale_(1e-4) = 0.4096` stays FRACTIONAL; ClipperLib offset deltas are
  doubles. Orca coordinates are ints (Point ctor rounds), but OFFSET
  DELTAS and epsilon amounts are unrounded doubles in scaled units.
- Ares quantizes every mm→scaled conversion to lattice ints
  (`CoordinateScale::checked_scale`), so `closing_ex(ε)` gets delta=100
  at 1e6/mm (same geometry) but delta=0 at 4096/mm → assertion failure
  (verified by the lattice-probe branch, since deleted).
- Ares `CoordinateScale`: Normal=1e-6/mm, LargeBed=1e-5/mm (dynamic).
  Orca: fixed `SCALING_FACTOR = 0.000244140625` (1/4096).
- Independent µm quantizations exist outside CoordinateScale:
  `contours.rs:238`, `segments.rs:173`, `extrusions.rs:255`
  (`(value * 1_000_000.0).round()`), `fuzzy_skin_noise.rs`,
  `shell_layers.rs`, `support_ironing.rs`, `mesh_slicer.rs`. Orca's
  mesh slicing runs in f64 with no µm quantization.

## Audit inventory

- ~126 `checked_scale` sites + ~30 unscale sites + 3
  `scaled_offset_coordinate` sites: classify each as COORDINATE (round,
  Point-ctor semantics) or DELTA/EPSILON (unrounded `mm / factor` as
  f32/f64, orca's double-delta semantics).
- The hardcoded ×1e6 sites above: remove the quantization or route
  through the scale.
- Tests with hardcoded 1e6-scale ints (classic_clip, seam placement,
  timelapse, etc.) need regeneration on the new lattice.

## Staged plan (each stage gated by golden + targeted parity cases)

1. **Spec + inventory table**: classify all conversion sites
   (coordinate vs delta) in a checked-in audit table.
2. **Delta semantics fix** (independent of the lattice switch!):
   convert DELTA/EPSILON sites to unrounded scaled doubles. At 1e6/mm
   this is nearly a no-op (100 vs 100.0); it unblocks the lattice
   switch and matches orca exactly.
3. **Mesh-slice quantization removal**: keep f64 where orca keeps f64.
4. **Lattice switch**: NORMAL_SCALE = LARGE_BED_SCALE = 1/4096 (the
   dynamic distinction becomes vacuous with i64 coords — consider
   collapsing the enum). Fix int-overflow guards and test fixtures.
5. **Full sweep + golden validation**; expected to resolve the three
   buckets above and possibly part of the M73 knife-edge family
   (sub-ms timing shifts from ±1 geometry).

## Risks

- The 866 PASS baseline moves: outputs recompute on the new lattice.
  Cases where ares-1e6 happened to round to orca's printed mm but
  ares-4096 differs would newly fail — mitigated by the fact that
  ares-4096 should equal orca-4096 exactly wherever the algorithm ports
  are faithful.
- WASM/Windows i64 arithmetic unchanged (coords stay i64).
- Effort: the inventory classification is mechanical; the fixture
  regeneration is the bulk.

## Probe results (2026-09-15, continuation #81)

- Fourth lattice family confirmed: the Flashforge Adventurer X4.464
  group (6+ cases) — ares emits a duplicate travel waypoint whose two
  positions differ by sub-4096 amounts at the 1e6 lattice (both print
  as X4.464 Y4.464), while orca's 4096 lattice collapses them to one
  integer and the router's consecutive-duplicate skip
  (`router.rs` `to_polyline` emulation) drops the hop.
- Second lattice probe (lattice-m2, deleted): after the stage-2 delta
  fix, the 4096 switch no longer panics on the Flashforge case, but the
  output diverges WORSE (5485 vs ~988 lines) starting at the priming
  anchor (`G1 X-9.007` → `X-9.009`, a 0.002 mm shift = 8 lattice
  units). At least one upstream-faithful stage still computes
  differently on the new lattice — the switch requires the full audit
  table (stage 1) plus per-stage anchoring before flipping.

## Stage-1 audit findings (2026-09-15, continuation #84)

119 `checked_scale` sites inventoried. The sub-unit epsilon class
(1e-4 mm = 0.41 units at 4096) that would round to zero:

- `perimeters/classic/materialize/path.rs:64` (fuzzy bbox, upstream
  `GCode.cpp:7559 instance_bbox.offset(scale_(EPSILON))`)
- `prepare_infill/bridge_over_infill/candidates.rs:58,142` (spacing
  multipliers / offset expansion)
- `fill/multiline.rs:96` (line-width bbox margin)
- `perimeters/arachne/top_surface.rs:59` (clip bbox)

Two further semantic mismatches surfaced during the audit:

1. **Rounding mode**: `CoordinateScale::checked_scale` truncates
   (`quotient as Coord`), but orca's `Point(double,double)` constructor
   ROUNDS (`Point.hpp:197 coord_t(std::round(x))`) — every coordinate
   conversion that flows through a Point construction upstream rounds.
   At 1e6/mm the quotients are integral (no observable difference); a
   4096 switch must switch to rounding to stay faithful.
2. **BoundingBox offsets**: upstream `BoundingBoxBase::offset` takes
   `coordf_t` (double) and constructs `Point(delta, delta)` — the
   double ctor rounds each component. So
   `bbox.offset(scale_(EPSILON))` at 4096 offsets by round(0.4096)=0,
   not by a fractional amount and not by truncation.

Sweep re-baseline at this point: 866 PASS / 107 DIVERGENT / 18
ORCA_ERROR (comment-trail work introduced no regressions; the churn
stayed inside the known oracle-crash families).

## Stage-2 completion (2026-09-15, continuation #85)

The remaining sub-unit epsilon sites fixed per their upstream cast mode
(three modes exist: fractional doubles into clipper, `Point(double)`
ctor rounding, explicit `coord_t()` truncation):

- `bridge_over_infill/candidates.rs` (both): `PrintObject.cpp:2527
  closing(area, float(SCALED_EPSILON))` — fractional clipper delta →
  `scaled_delta`.
- `perimeters/classic/materialize/path.rs` (fuzzy bbox): `GCode.cpp:7559
  instance_bbox.offset(scale_(EPSILON))` — the bbox offset rounds the
  delta through the Point(double,double) ctor →
  `scaled_delta(..).round()`.
- `perimeters/arachne/top_surface.rs`: `PerimeterGenerator.cpp:160
  bbox.offset(SCALED_EPSILON)` — same rounded-bbox mode.
- `fill/multiline.rs` (x_margin): `FillRectilinear.cpp:3017
  line_width + coord_t(SCALED_EPSILON)` — an EXPLICIT truncating cast;
  the existing `checked_scale` (truncating) is already faithful. No
  change.

Behavior-neutral at 1e6/mm (all values integral): octagram and H2D
fixtures byte-stable, golden green, bridge_over 244 tests green.

## Stage-3 audit (2026-09-15, continuation #86)

The mesh-slice µm quantization is SMALLER than the milestone assumed:
`contours.rs scaled()` quantizes only the stitching IDENTITY keys
(PointKey/SegmentKey for dedup and adjacency); the contour geometry
stays f64 end-to-end. Orca's stitcher compares floats directly, so the
µm keys are an ares-internal merge tolerance, not an output-geometry
lattice.

The switch-blocking µm hardcodings are in the G-code emit layer:

- `motion/arc.rs`: circle fitting runs at a hardcoded 1e6/mm scale
  (`SCALE = 1_000_000`, collinearity area `1e8` µm², `quantize`
  truncation). The physical values match upstream
  (`Circle.cpp:14-27`: `scale_(scale_(Parallel_area_threshold))` =
  1e-4 mm²), but the constants bake in µm units — the CoordinateScale
  must thread through for any lattice switch.
- `motion/arc/simplify.rs`: `COORDINATE_UNITS_PER_MILLIMETER = 1e6`
  in the tolerance² distance test.
- `avoid_crossing/boundary.rs`: `unit(1.0).unwrap_or(1_000_000)` grid
  resolution fallbacks.

None of these affect the current 1e6 outputs; they join the stage-4
precondition list alongside the m2 priming-anchor drift.

## Stage-3 code completion (2026-09-15, continuation #86)

All live µm hardcodings now follow the active CoordinateScale:

- path simplification (`3fe01d57`): Douglas-Peucker tolerance² and point
  comparisons take `units_per_mm` from the caller's scale
  (path_simplification.rs, classic materialize path).
- arc circle fitting (`2740ecab`): try_arc/fit_circle/circle_from_three/
  deviation_sum/quantize take `units_per_mm`; the parallel-area threshold
  is now derived as 1e-4 mm² in lattice units² (matching Circle.cpp's
  `scale_(scale_(0.0001))`) instead of the literal 1e8 µm². The center
  reconstruction keeps `x * (1.0/SCALE)` — the `/SCALE` form differs by
  an ulp and flips the zero-axis arc split (pinned by the arc test).
- avoid_crossing/boundary.rs `unit()` was already scale-aware; its
  `unwrap_or(1_000_000)` is an unreachable overflow fallback.

Everything is behavior-identical at 1e6 (arc tests 83/83, fixtures
byte-stable, golden green). Stage-4 preconditions remaining: the m2
priming-anchor drift hunt and the checked_scale rounding-mode audit
(truncation today vs the Point(double) constructor's rounding).

## m3 probe (2026-09-15, continuation #87)

Post stage-2/3, the 4096 probe's Sermoon diff drops 5485 → 3851 lines
(the epsilon/delta and scale-threading fixes helped). The priming-anchor
"drift" is now understood: skirt_loops=0, so the first travel targets the
outer wall SEAM — the 0.002 mm shift is seam-placement lattice noise (the
Loop ±1 family), not a distinct unfaithful stage. The remaining
divergence taxonomy at 4096:

1. E-value 5th-decimal flips (E.40045 vs E.40044) with IDENTICAL printed
   XY — the extrusion chain (length in mm from the lattice ints ×
   mm3_per_mm) differs sub-3-decimal.
2. Wipe/intermediate points off by 0.001.
3. Comment placement following route differences (the first-travel
   comment lands on a different hop than upstream's because the route
   differs; the comment logic itself matches
   `GCode.cpp:7488-7504`, including the middle-hop `travel_to_xy(point,
   comment)`).

Conclusion: stage 4 is now a pure geometry-lattice-chain audit — every
mesh→lattice→perimeter→extrusion intermediate must reproduce orca's
4096 ints exactly (the clipper ports are verified faithful; the inputs
are not yet). The two-point scale rounding audit (checked_scale
truncation vs `Point::new_scale` truncating / `Point(double,double)`
rounding — both ctors exist upstream) remains the mechanical
prerequisite.

## Rounding-mode audit resolved (2026-09-15, continuation #87)

`Point.hpp:662-668`: the dominant conversion `scaled<coord_t>(v)` is
`Tout(v / SCALING_FACTOR)` — an explicit TRUNCATION; the rounding
variant is commented out upstream with a TODO. `Point::new_scale` also
truncates; only the direct `Point(double,double)` constructor rounds.
ares's `checked_scale` (truncating) is therefore faithful to the
dominant pattern — the rounding precondition for stage 4 is already
satisfied. The only per-site check left is where upstream constructs
Points directly from doubles (the bbox-offset family already handled in
stage 2). Stage 4 reduces to: flip the constants, fix the fallout
empirically (the E 5th-decimal / sub-3-decimal chain), validate golden
+ sweep.

## Stage-4 kickoff: mesh-slicing chain verified faithful (2026-09-15, #88)

The mesh→intersection chain is architecturally identical to upstream:

- `TriangleMeshSlicer.cpp:1827-1833 make_trafo_for_slicing`: the
  transform is pre-scaled by (1/SCALING_FACTOR, 1/SCALING_FACTOR, 1) and
  applied in f32 with NO vertex rounding; ares's `prescaled_xy(factor)`
  (project/transform.rs:71) matches.
- Intersections: `TriangleMeshSlicer.cpp:277-278` =
  `coord_t(floor(b + (a-b)*t + 0.5))`; ares `interpolate_coordinate`
  (mesh_slicer/intersection.rs:168) is the same formula.
- On-layer (inherited) points: upstream assigns the float-scaled vertex
  into a coord_t (truncation); ares `inherited_point` casts the same
  f32-scaled vertex `as i64`.

Exact-coordinate meshes (the parity cube) slice identically at any
lattice. The 4096 E 5th-decimal flips therefore originate DOWNSTREAM —
in the perimeter/infill offsetting chain (float spacing deltas into the
clipper ports), not in slicing. Stage 4's remaining work is the
offsetting-chain int comparison at 4096.
