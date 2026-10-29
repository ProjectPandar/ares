# S-C0: connect_infill touching-grid admission

## Status and priority

SPEC-REVIEW-REQUEST, docs only, based on main
98a2bfc325b2a06ba4e8b95b9719acd827e2f8f1. No production or test change.
This slice precedes the blocked object-region spacing slice. SPEC approval
and a separately reviewed public RED are required before production edits.
The accepted STOP report is commit 3be5e7af1, in
`docs/architecture/adaptive-object-region-spacing-preflight-stop.md` on
`worker3/adaptive-object-region-spacing-spec`; that branch remains unchanged.

## Established public failure and frozen input

Reuse the exact authored S-C0 input from
`/tmp/worker3-object-spacing-native-1tgobnaj/inputs/S-C0.3mf`:
SHA256 `9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1`.
It is a single-region, no-modifier SupportCubic control: 20x20x10 object,
Classic two walls, 0.2mm layers, density30%, sparse width0.45mm, multiline1,
three top/bottom layers, anchors400%/20mm, without support/brim/skirt.
Its profile and packaging provenance and generator are in the STOP archive;
no input tuning or regenerated variant replaces these frozen bytes.

On exact main98a2, the public `slice_project_plate_validated` API helper
and the real `ares slice` CLI both exit101 at
`crates/ares-core/src/fill/connect/touching.rs:48`:
`assertion failed: grid_contains(&grid, trace_start)`.
The helper calls the public API with default metadata/overrides, plate None,
and SlicerContext seed0. Its exit101 is a process panic, not an API error.
Native official and seeded both exit0 and produce complete S-C0 G-code;
each has 1323 Sparse-infill XY/E moves. These are preflight observations,
not a newly submitted RED or a claim of Ares output parity.

Raw evidence remains under the root above: `public-official/S-C0/` has both
stderr streams and `official/S-C0/`, `seeded/S-C0/` have Native outputs.
Durable archive directory:
`/home/indexyz/ares-evidence/worker3/adaptive-object-region-spacing-preflight/`.
`native-and-public-stop-73e1956c6.tar.gz` SHA256:
`825bd9b59b735f40cb0e44580609820f84e8bb76e37204fa60db2b94da62d1a3`.
`STOP-evidence.json` SHA256:
`40d65bc0921a4118f827d8794bafdd1385c7cd1af3dd7e8fbe38d2584f8d81f6`.
Only authored input/generator may enter git; Native outputs remain runtime evidence.

## Upstream owner and Rust destination

All source references are to `OrcaSlicer/src/libslic3r/`:

- `Fill/FillBase.cpp:1580-1594`, `Fill::connect_infill`, constructs the
  boundary graph via `create_boundary_infill_graph`.
- `Fill/FillBase.cpp:1432-1565`, `create_boundary_infill_graph`, owns the
  ordered boundary/intersection working copy. Lines1553-1561 invoke the
  touching visitor with clip_distance=1.7*scale_(spacing) and collision
  distance=0.8*scale_(spacing).
- `Fill/FillBase.cpp:996-1241`, `mark_boundary_segments_touching_infill`,
  owns consumed boundary intervals. Lines1027-1031 construct the grid with
  bbox inflation1.43*collision distance and cell resolution
  coord_t(max(clip_distance, distance_colliding)+scale_(10.)).
- `Fill/FillBase.cpp:1164-1168` clips the infill by Euclidean distance;
  lines1195-1196 choose the segment endpoints. Lines1209-1222 construct
  both offset traces from the normalized direction and perpendicular,
  cast to coord_t, assert bbox containment, then visit grid cells.
- `EdgeGrid.hpp:172-250`, `Grid::visit_cells_intersecting_line`, asserts
  endpoint containment, translates by bbox.min, divides into cells, visits
  the first cell, and terminates on an out-of-range stepped cell.
- `EdgeGrid.cpp:142-163`, `Grid::create_from_m_contours`, merges contour
  points into the supplied bbox, pads16 coordinate units, and computes
  ceil-divided rows/columns.

Primary Ares destination:
`crates/ares-core/src/fill/connect/touching.rs::mark_boundary_segments_touching_infill`
and its `touching_grid`/`grid_contains` helpers. Supporting source-equivalent
expressions, only if localization proves a mismatch, are in
`fill/connect/collision.rs::thick_trace_line` and `clipped_infill_segments`,
`fill/connect/scale.rs::inflate_bbox_round_delta`, and
`geometry/edge_grid.rs::EdgeGrid::new_from_contours`,
`geometry/edge_grid/query.rs::EdgeGrid::visit_cells_intersecting_line`,
`geometry/edge_grid/raster.rs::visit_line`.
Paths in this paragraph are relative to `crates/ares-core/src/`.
`fill/connect/graph.rs::build_working_graph` supplies the source distances.
These existing modules are the compatibility shell for the upstream
boundary graph and grid; this slice corrects that port, adding no pipeline.

## Included behavior and localization decision

Include S-C0 public admission and the source-owned touching trace/grid
semantics responsible for its panic. First compare ordered connector inputs,
clipped segment, spacing/distances, completed trace coordinates, bbox before
and after grid creation, and visited cells against release Native. Use
bounded read-only localization on CPU12; preserve input and output hashes.

Native's release asserts are compiled out under NDEBUG, including the
FillBase and EdgeGrid containment asserts. This fact alone does not prove
that Native reaches an identical out-of-bbox trace. Ares already safely
terminates grid traversal on out-of-range cells, but the caller asserts
before entering it. Do not declare the caller assertion the sole root cause
until the actual upstream operands and release traversal have been checked.
If operands differ, port the first differing source expression. If they
match and release Native legitimately traverses beyond the asserted bbox,
report that evidence and the exact debug-assert removal scope for review.
Do not expand the bbox, clamp endpoints, skip a path, catch the panic, add
an admission guard, change spacing, or alter connector ordering to force PASS.
If the first divergence is outside this source boundary, STOP for an addendum.

## RED then GREEN and acceptance

After SPEC approval, commit the exact S-C0 authored input plus provenance
and a public E2E RED separately. Run Native official and seeded at test time;
require successful completion and compare generated output strictly using
the existing comparator. Exercise both public API and real CLI: success and
complete output are asserted, so the current exit101 remains a real failure.
Do not assert panic as desired behavior or use normalization/retries.
Record API/CLI status, stderr, raw output SHA and runtime Native result/SHA.
RED evidence must be approved before GREEN.

GREEN must make the same frozen S-C0 succeed through both public surfaces
and pass strict output comparison in both modes. Preserve existing connector,
AdaptiveCubic and SupportCubic controls; report any residual as STOP instead
of broadening production scope. Existing assertions are otherwise untouched;
any required test retirement needs separate approval. No new suppressions,
unsafe code, probes, fixture predicates or fallback. Rust files stay <=400LOC.
Run directed tests via outer taskset CPU12, jobs1/threads2, fresh Native cache
and artifact roots per mode/run, START/END in inbox. Final strict clippy,
fmt and wasm32 checks run on the exact submitted hash. Full/workspace remain
coordinator-only; no push or merge is authorized by this SPEC.

## Separate deferred findings

A-C1 and A-C2/C3/C4 show earlier Inner-wall output differences against Native,
before the intended sparse-spacing boundary; A-C0 passed. Their cause has
not been established. Preserve all original inputs/roots and the STOP report;
these differences are a separate deferred finding, with no work in this slice.

Object-region adaptive/support spacing remains BLOCKED until this S-C0 slice
lands. Afterwards re-evaluate its controls before resuming that SPEC. Neither
the observed object-spacing scalar mismatch nor the earlier Inner-wall
finding is part of this production boundary.
