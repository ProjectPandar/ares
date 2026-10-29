# SPEC — ClipperLib 6.4.2 open-subject `ctDifference` boundary-collinear parity

## Status

RESOLVED — see "Root cause (proven)" below. The parity gap is in
`inside_internal_surfaces`'s mm→scaled roundtrip, not the clipper engine.
With the fix applied, `ironing-pattern-triangles` native↔ares gcode is
byte-identical except known wrapper artifacts (7940/7940 wipes).

## Root cause (proven)

`inside_internal_surfaces` (`motion/travel.rs`) accepted mm `arc::Point`s
and re-scaled them via `checked_scale(point - offset)` — a truncating
cast — while upstream `travel_inside_internal_regions`
(`RetractWhenCrossingPerimeters.cpp:9-51`) consumes the coord_t travel
polyline `[m_last_pos, destination]` directly (no mm roundtrip).

The mm roundtrip `coord → unscale → +offset → −offset → ×1e6 → truncate`
loses a unit when the sub-ulp error lands below the integer (e.g.
`-26974622 → mm → −26974621.9999… → truncate → −26974621`). For the
witness travel this made both endpoints 1 unit off
(`-26974621,-5898596` / `-23006254,-5898596` vs native
`-26974622,-5898597` / `-23006254,-5898597`), placing the travel 1 unit
*inside* the internal island instead of boundary-collinear → `diff_pl`
empty → `inside=true` → `can_skip_retraction` → missing wipe/retract.

Fix: `inside_internal_surfaces` takes coord_t `Point`s directly (caller
already has `cursor` / `first_scaled` coord_t), matching the upstream
`const Point&` contract. Verified: triangles leg byte-identical G-code
(all 7940 wipes) except wrapper artifacts.

## Evidence

Probe: `tests` leg `ironing-pattern-triangles` (3mf `/tmp/w4-gap-probe/ip-triangles.3mf`),
travel `(-26974622,-5898597) → (-23006254,-5898597)` in native Clipper
scaled coords (same plate offset in both — offset `(133.039205, 115.992105)`).

gdb on seed-replay Native (`ClipperUtils.cpp:906` entry + `finish`):

- `diff_pl(travel, clipped)` where `clipped = [(-4499996,3408597),
  (-27500000,-244999), (-27500000,-5898597), (-2500000,-5898597)]`
  (the 4-pt `clip_clipper_polygons_with_subject_bbox` output for
  `m_internal_islands[4]`, contour of the 27-pt internal ExPolygon at the
  ironing layer).
- The travel segment is exactly collinear with the clip polygon's bottom
  edge `(-27500000,-5898597) → (-2500000,-5898597)` (y = -5898597).
- **Native returns `Polylines` of length 1: `[(-23006254,-5898597) →
  (-26974622,-5898597)]`** — the open subject kept (reversed), i.e.
  ClipperLib treats the boundary-collinear open segment as *not
  contained* for `ctDifference`.
- Ares `difference_open_polylines` on ITS inputs returns `[]` →
  `inside_internal_regions` reports inside → `can_skip_retraction` fires →
  the wipe/retract pair is skipped. Native emits `;WIPE_START … ;WIPE_END`
  + `G1 E.4 F1800`.

## Corrected diagnosis (engine ruled out)

Step-2 gdb proof on seed-replay (corrected TEdge offsets: PolyTyp@56,
Side@60, WindDelta@64, WindCnt@68, WindCnt2@72, OutIdx@76):

- 104 `IntersectEdges` hits involving open edges (wd==0) at y=-5898597,
  NONE for our travel edge — upstream never `IntersectEdges` the
  coincident pair. Open-edge contribution is decided at local-minima
  insertion (`InsertLocalMinimaIntoAEL` → `SetWindingCount` +
  `IsContributing`, clipper.cpp:1488–1515).
- Ares instrumentation (`ARES_CLIP_DBG` in
  `clipper/winding.rs::set_winding_count`/`is_contributing`): the open
  travel edge enters as `wd=0 wc=1 awc=0` (contributing) in most diff
  calls, but in the failing call `awc=1` → `!fill_inside(1)` → not
  contributing → dropped → diff empty → inside=true.
- **AREs' travel edge is `bot=(-26974621,-5898596)
  top=(-23006254,-5898596)` — y=-5898596, while the clip quad's bottom
  edge is y=-5898597 in BOTH ares and native.** Native's travel edge is
  `(-26974622,-5898597)→(-23006254,-5898597)` — exactly collinear.
- Therefore in ares the travel is 1 unit ABOVE the clip boundary =
  strictly inside the quad interior. `diff=∅` → inside=true is
  geometrically CORRECT for ares' inputs. Keeping a strictly-inside open
  segment to force the wipe would be NON-faithful to ClipperLib.
- Endpoint diff: x -26974621 vs -26974622, y -5898596 vs -5898597 (1
  unit each). These are travel polyline endpoints (prev extrusion end /
  next extrusion start). The 1-unit divergence is upstream of
  `travel_inside_internal_regions` — in the endpoint producer (fill
  line/extrusion endpoint emission), not `clipper.cpp`.
- Ordering mechanism (now moot for this witness): with travel.y ==
  clip_bottom.y, native's minima share one `botY` batch; AEL-prefix
  `WindCnt2` accumulation then sums the clip bound pair (+1,-1) to 0 →
  contributing. In ares the travel's minimum lands in a LATER scanbeam
  batch (different y) → clip edges already in AEL → `awc=1`.

Conclusion: the parity gap is a 1-unit coordinate divergence in the
travel endpoints produced by the upstream fill/extrusion pipeline — NOT
a clipper engine logic gap. The `into_open_polylines` +
`inside_internal_surfaces` rewrites remain valid faithful mirrors
(prerequisites), but an engine `WindCnt`/`WindCnt2` patch is NOT
warranted by the evidence. Next step: trace the endpoint producer for
the -26974622/-5898597 vs -26974621/-5898596 diff (fill line clip
endpoint or vertex emission).

## Earlier (superseded) analysis

The clip polygon fed to `diff_pl` was previously believed byte-identical
in both engines (clip side is: verified via gdb dump vs `W4DBG-D`
instrumentation). The subject side was NOT identical — that was the
hidden 1-unit divergence.

## Upstream source boundary (superseded — kept for the record)

`deps_src/clipper/clipper.cpp` (ClipperLib 6.4.2 Orca fork):

- `Clipper::SetWindingCount` (clipper.cpp:1195–1280) — for an open edge
  (`WindDelta == 0`) with `m_ClipType != ctUnion`:
  `edge.WindCnt = 1; edge.WindCnt2 = e->WindCnt2` (lines 1213–1217), where
  `e` is the preceding same-polytype edge in the AEL; `WindCnt2` is the
  *other* polytype's winding count.
- `Clipper::IsContributing` (clipper.cpp:1297–1390) — `ctDifference`,
  `ptSubject`, `pftNonZero`: contributes iff `abs(WindCnt) == 1 &&
  WindCnt2 == 0` (lines 1314, 1344–1352).
- `Clipper::AddLocalMinPoly` / `outRec->IsOpen = (e->WindDelta == 0)`
  (clipper.cpp:2007) — an emitted remainder of an open subject is marked
  `IsOpen`, which is what `PolyTreeToPolylines`
  (`ClipperUtils.cpp:207–221`) then returns as a `Polyline`.

## Ares destination boundary

`crates/ares-core/src/geometry/clipper/` (Clipper1-style port):

- `winding.rs::is_contributing` (`edge.wind_count`/`alternate_wind_count`
  equivalents of `WindCnt`/`WindCnt2`; Difference arm at `winding.rs` — subject
  requires `!fill_inside(alternate, alternate_wind_count)`).
- `winding.rs` `first_winding`/`nonzero_winding` (equivalents of
  `SetWindingCount`).
- `intersections/open.rs::intersect_different_role_open` — open-vs-closed
  intersection handling (`add_out_point`/`unassign_if_output`), the path that
  decides whether an open segment survives a boundary touch.
- `polytree.rs::into_open_polylines` — already fixed (unstaged) to mirror
  `PolyTreeToPolylines` (all contours recursively, open AND closed);
  prerequisite, rides with the GREEN.
- `motion/travel.rs::inside_internal_surfaces` — already rewritten
  (unstaged) to the upstream `travel_inside_internal_regions` algorithm
  (bbox-clip + `difference_open_polylines`); prerequisite, rides with the
  GREEN.

## Included

- Make `difference_open_polylines` (open subject, `ClipOperation::Difference`,
  `FillRule::NonZero` both sides) keep a subject segment that is collinear with
  a clip-polygon boundary edge, matching ClipperLib 6.4.2's `WindCnt`/`WindCnt2`
  accounting (`SetWindingCount` open-edge branch 1213–1217 + `IsContributing`
  Difference arm 1344–1352).
- The unstaged `polytree.rs` + `travel.rs` rewrites ship in the same GREEN
  (they are the faithful upstream mirrors the fix needs to be observable
  through).

## Deferred

- Other open-subject operations (`Intersection`, `Union`, `Xor`) beyond what
  the Difference parity fix touches incidentally.
- Any wholesale port of ClipperLib scanbeam internals; the fix is bounded to
  the winding/contribution accounting for the boundary-collinear open-subject
  Difference case.
- `intersection_open_polylines` parity beyond what the shared code path yields.

## RED / GREEN plan

- RED: the already-failing ironing legs (`alignedrectilinear`, `cubic`,
  `grid`, `lateral-honeycomb`, `lateral-lattice`, `quartercubic`, `tpmsfk`,
  `tri-hexagon`, `triangles` in `option_coverage/ironing_pattern.rs`, RED
  `8df488c1f`) are the witness — no new synthetic engine unit tests.
- GREEN: engine fix + the two kept rewrites; FULL both modes (engine change
  has wide blast radius — seeded + official `cargo nextest run --workspace`
  with `test-threads = "num-cpus"` per `.config/nextest.toml` ... under CPU15,
  `CARGO_BUILD_JOBS=1`).
