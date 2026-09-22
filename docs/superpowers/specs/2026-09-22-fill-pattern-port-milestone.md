# Fill-Pattern Port Milestone (sparse/surface infill generators)

Status: PLANNED — boundary for the remaining `sparse_infill_pattern` /
surface-pattern option-coverage rejections.

## Upstream boundary (source-cited)

Each rejected pattern needs its `Fill*.cpp` `fill_surface`/`_fill_surface_single`
generator ported AND a `sparse_anchoring` arm for the bridge-over-infill anchor
projection (`generate_sparse_infill_polylines_for_anchoring`,
`sparse_anchoring.rs:262`).

| Pattern | Upstream file | ares generator | anchoring arm |
|---|---|---|---|
| `line` | `FillLine.cpp` | `monotonic::append` exists (extend match) | `fill_monotonic_surface` |
| `alignedrectilinear` | `FillRectilinear.cpp` | `monotonic::append` exists | `fill_monotonic_surface` |
| `crosszag`/`lockedzag` | `FillRectilinear.cpp` (zigzag variants) | none (extend monotonic) | `fill_monotonic_surface` |
| `concentric` | `FillConcentric.cpp` (148 LOC, arachne wall-toolpath) | `fill_entities/concentric.rs` (needs a `fill::concentric::fill_surface` returning polylines) | concentric polylines |
| `tri-hexagon` | `FillRectilinear.cpp` (3-sweep) | `fill_multiline_surface` TRIANGLE_SWEEPS exists — wire pattern | same |
| `quartercubic` | `FillCubic.cpp` variant | `fill_entities/cubic.rs` exists — extend | cubic anchoring arm |
| `honeycomb` | `FillHoneycomb.cpp` | none — port hex grid | hex polylines |
| `lightning` | `FillLightning.cpp` | none — port lightning tree | lightning polylines |
| `tpmsd`/`tpmsfk` | `FillTpmsD.cpp`/`FillTpmsFK.cpp` | none — port TPMS sheets | TPMS polylines |
| `lateral-honeycomb`/`lateral-lattice` | `FillLateral*.cpp` | none | none |
| `hilbertcurve`/`archimedeanchords`/`octagramspiral` | `FillPlanePath.cpp` | `plane_path` exists for surface role only — extend to sparse | plane-path polylines |

## Also tracked (not generators)

- Real content divergence on `zigzag`/`cubic`/`adaptivecubic`/`supportcubic`:
  sorted-content diff is a `G1 F<feedrate>` count divergence → the
  cooling/estimator feedrate chain, same class as the M73 boundary lag.
- M73 placement is a confirmed oracle TBB race (two oracle runs place
  `M73 P10 R7` at different lines; ares matches one). M73-gated FAILs are
  races — re-run can flip them; not a content divergence.

## Exit criteria

- Every `sparse_infill_pattern` enum value slices byte-identical (modulo the
  `generated` header and M73 race) on at least one printer fixture.
- `surface`/`internal_solid`/`top`/`bottom` pattern enums likewise.
- All new generators cited to their `Fill*.cpp` source boundary; each file
  <400 LOC with tests in a separate `tests.rs` mod.

## Concentric sparse probe (2026-09-22)

Wiring `Concentric` into the `anchor_projection` whitelist + a
`sparse_anchoring` arm reusing `fill_entities::concentric` generation slices
but diverges ~3935 non-M73 lines: the emitted sparse infill uses
density-scaled spacing (`FillConcentric` spacing = `flow.spacing` at reduced
density), and the wipe/retract chain differs. The `append_configured` surface
concentric path does not carry the sparse-density semantics. A correct port
needs the sparse `_fill_surface_single` spacing/density path, not just the
anchor-projection plumbing — reverted to the honest `unsupported` rejection.

## FillLine port prerequisites (2026-09-22, corrected flow)

`FillLine::_fill_surface_single` (FillLine.cpp:38) is the simplest remaining
fill generator. Reading the actual upstream source, the flow is:
1. `expolygon.rotate(-direction.first)` so infill is vertical.
2. `_min_spacing = scale_(spacing)`; `_line_spacing = min_spacing/density`
   (or `_adjust_solid_spacing(bbox.size(0), line_spacing)` at full density).
3. Non-full-density: `bounding_box.merge(align_to_grid(bbox.min, spacing,
   direction.second.rotated(-direction.first)))` — anchor to the rotated
   reference point exactly like monotonic.
4. Build a vertical `Lines` grid `x` from `bbox.min(0)` to `bbox.max(0)+EPS`
   stepping `_line_spacing`; each `_line(i, x, ymin, ymax)`.
5. `intersection_pl(polylines_src, offset(expolygon, scale_(0.02)))` — clip
   the open lines against the *slightly-expanded* expolygon (keeps edge lines
   on skewed/vertical sides). `offset(expolygon)` returns `Polygons`.
6. Extend each surviving segment's endpoints by `extra =
   floor(min_spacing*0.3 + 0.5)` (the `INFILL_OVERLAP_OVER_SPACING` term),
   swapping so the low-Y end is extended down and high-Y up.
7. `chain_polylines(polylines)` then `_can_connect(dx, dy) &&
   expolygon_off.contains(Line(last, first))` — where `expolygon_off =
   offset_ex(expolygon, min_spacing/2)` — to append connected segments into
   one `ThickPolyline` per run.
8. `it->rotate(direction.first)` to rotate each output polyline back.

ares primitives already available: `chain_polylines`
(`project_slice/perimeters/classic/shortest_path/polyline_chain.rs`),
`adjust_solid_spacing` (`fill/rectilinear/surface.rs:198`), `align_to_grid`
(`fill/multiline.rs:272`), `rotate_expolygon`/`rotate_polylines`,
`offset_expolygons`/`offset_expolygon`, `intersection_open_polylines`
(`geometry/clipper/polyline.rs`), `open_polyline_inside_expolygon`
(= `ExPolygon::contains(Line)`). Still needed: `Polygon::rotate(angle,center)`,
`ExPolygon::rotate`, `ExPolygon::to_polygons` (contour+holes flatten for the
`offset(expolygon)` clip arg), `Polygon::first_point`, `Polyline::rotate`, and
a `Line`-grid builder + `_can_connect` (distance ≤ `_diagonal_distance` and
not over `_line_oscillation`/`_min_spacing`). These land with `fill/line.rs`
together so no dead helpers are committed.

## `line` is the next confirmed target (2026-09-22)

Confirmed: oracle slices `sparse_infill_pattern=line` cleanly (exit 0) while
ares rejects `UnsupportedProjectFeature`. The rejection gate is
`anchor_projection::validate` (`transaction/anchor_projection.rs:20`) whose
whitelist omits `Line`. To port: add `Line` to that whitelist, add a
`sparse_anchoring` arm that calls a new `fill::line::fill_surface` (the
`FillLine::_fill_surface_single` port returning `Vec<ThickPolyline>`), and add
the `fill_entities` `Line` dispatch arm. A `fill/line.rs` draft was written and
verified to compile, then removed because it was dead code — the primitives it
added (`Polygon::rotate`/`first_point`, `ExPolygon::rotate`/`to_polygons`,
`BoundingBox::merge_point`, `Polyline::points_mut`) must land together with the
wired `fill_entities`/`sparse_anchoring` arms, not orphaned.
