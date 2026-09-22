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

## FillLine port prerequisites (2026-09-22)

`FillLine::_fill_surface_single` (FillLine.cpp:38) is the simplest remaining
fill generator but needs geometry primitives that do not yet exist in
`ares-core::geometry`:
- `ExPolygon::rotate(angle, Point center)` — rotate contour+holes.
- `Polygon::bounding_box()` — the rotated outer-contour bbox.
- `ExPolygon` → `Polygons` extraction (`affine_transform`/`out`).
- `intersection_ln(Lines, Polygon)` — clip open lines to a polygon.
- `offset(Polygon, delta) → Polygons` — polygon (not expolygon) offset.
- `ExPolygon::contains(Line)` — does the expolygon fully contain a segment.

Upstream flow: rotate expolygon so infill is vertical → shrink the rotated
outer contour by `min_spacing` → build the bounding box → `intersection_ln` a
vertical `Lines` grid against the shrunk contour (with `x0` centered on the
bbox and `scale_(params.reference_point)` for the `center_infill` anchor
exactly like the rectilinear/monotonic anchor) → un-rotate each surviving
segment → drop any segment whose line is not contained in the original
expolygon → extend endpoints by `min_length/2` → `_can_connect` chain.

The `_fill_surface_single` returns a `ThickPolyline` per run of connected
vertical segments; `line.zigzag`/`line` reuse the existing
`fill::checked_rotate` helpers for the point rotations, but the polygon-level
rotate/bbox/offset/intersection_ln/contains primitives must be ported first.
The existing `rectilinear::prepare_rectilinear_contours` produces
`MonotonicRegion`s, not raw `Lines`, so it cannot be reused for the raw
segment list `FillLine` consumes.
