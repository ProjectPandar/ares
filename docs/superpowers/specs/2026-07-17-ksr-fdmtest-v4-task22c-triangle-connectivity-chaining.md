# Task 22C: Triangle-Connectivity Slice Chaining

## Status and objective

This specification is a draft. Production and test implementation may begin
only after an independent reviewer and the repository's default-model review
gate approve these exact bytes.

Task 22C is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` parity program. Released Task 22B commit
`455a0d12a9c6ac48f6e2796669b4300a6a6190a2` retains directed, scaled raw
triangle-plane intersection lines. Structural cleanup commit
`4180d082858696d7eacd094358787a655bfc59f4` removes the final production
`include!` source split and leaves every Rust file below 400 physical lines.

Task 22C consumes each Task 22B raw layer exactly once and ports only Orca's
triangle-connectivity greedy chaining seam. The result is an ordered list of
closed integer `Polygon` point sequences plus unrepaired `OpenPolyline`
sequences whose tagged mesh endpoint identities and scaled length are retained
for the next slice. The package does not join open chains, repair gaps, infer
holes, apply `slicing_mode`, invoke Clipper, create regions, or emit G-code.

For the committed 3MF, the source-derived result is exactly 460 chained layer
slots, 3,288 closed polygons, zero open polylines, and 116,472 closed polygon
points. The public project API still returns
`SliceError::ProjectSlicingIncomplete`, but only after consuming its raw state
through the new chaining boundary. No approximate G-code or old STL-pipeline
fallback becomes observable.

### Why this package stops before loop repair

`chain_lines_by_triangle_connectivity` is an independently testable Orca
function. It relies only on directed raw lines, tagged vertex/edge identity,
and integer points. The next upstream code adds mutable open-polyline state,
exact joining with optional reversal, nearest-endpoint search, a 2 mm gap
threshold, loop-closing heuristics, and later polygon processing. Combining
those behaviors would hide distinct ordering, distance, and repair contracts.

The Task 22C state is therefore an upstream rewrite boundary, not a new Ares
pipeline. Existing `planning.rs`, `segments.rs`, `contours.rs`, `pipeline.rs`,
and their f64 `Point2`/STL data remain outside the project path and may not be
called, adapted, or used as a fallback.

## Fixed upstream rewrite boundary

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The workspace-relative
`OrcaSlicer` checkout may have another HEAD; citations must be verified with
`git show 8500fcdccaa10b5099ac20d252af3a7c560046f1:<path>` without changing
that checkout.

- `src/libslic3r/libslic3r.h:38-44` defines signed 64-bit `coord_t`.
- `src/libslic3r/Point.hpp:41-43,60-70,187-203` defines integer `Point` and
  `Points` storage.
- `src/libslic3r/MultiPoint.hpp:15-44,172-179` defines ordered point storage
  and open point-sequence length.
- `src/libslic3r/Polygon.hpp:14-30,44-62` defines a closed ordered point
  sequence whose first point is not duplicated at the end.
- `src/libslic3r/Polygon.cpp:11-19,52-73` defines later length, area, and
  orientation behavior. Those operations are cited only to mark the stopping
  boundary and are not implemented here.
- `src/libslic3r/Polyline.hpp:14-38` defines the general polyline type. The
  core function below does not depend on it, so Task 22C does not create a
  general Ares polyline abstraction.
- `src/libslic3r/TriangleMeshSlicer.cpp:58-69` defines the vertex-or-edge
  `IntersectionReference` sentinel pair.
- `TriangleMeshSlicer.cpp:71-78` binds an integer point to that reference.
- `TriangleMeshSlicer.cpp:80-145` defines directed `IntersectionLine`,
  `SKIP`, `NO_SEED`, reversal, endpoint identities, and facet-edge metadata.
- `TriangleMeshSlicer.cpp:1043-1056` defines local `OpenPolyline`: start/end
  identities, ordered points, length, and initial unconsumed state.
- `TriangleMeshSlicer.cpp:1058-1161` is the sole production algorithm in this
  package: `chain_lines_by_triangle_connectivity`.
- `TriangleMeshSlicer.cpp:1383-1415` shows `make_loops` invoking the function.
  Task 22C stops immediately after that first connectivity result.

The fixed source contains no call to `set_no_seed`. Its disabled
`remove_tangent_edges` implementation is inside `#if 0`. For the admitted Task
22B raw vocabulary, `is_seed_candidate()` is therefore equivalent to “not yet
consumed.” Task 22C must not invent a `NO_SEED` producer or duplicate-edge
filter.

## Existing Ares input seam

Task 22C consumes only these released boundaries:

- `geometry::Point`: request-scaled signed 64-bit XY coordinates;
- `mesh_slicer::EndpointReference::{Vertex(u32), Edge(u32)}`: a tagged form
  stricter than Orca's two `-1` sentinels;
- `mesh_slicer::IntersectionLine`: directed A-to-B points and identities in
  original mesh face order;
- `project_slice::raw_intersections::IntersectedPrintObject`: ordered print
  plan, volume occurrence identity/type, and one raw vector per planned layer.

No coordinate is unscaled or converted through `Point2`. No endpoint is
sorted by coordinate. In particular:

- `segments::Segment2::new` is forbidden because it sorts endpoints and loses
  line direction;
- `contours::stitch_segments` is forbidden because it uses undirected XY
  adjacency and may reverse edges;
- `contours::Contour::new` is forbidden because it rotates the start and
  normalizes winding;
- `print_apply::ScaledPoint` is forbidden because it belongs to a separate
  fixed-scale printable-area diff seam;
- `project_slice/tests/raw_fixture/closed_components.rs` remains a test-only
  independent graph oracle and may not become production code.

## Rust destination and ownership

### Minimal polygon domain

Create `geometry/polygon.rs` with a private-to-crate `Polygon` that owns
`Vec<Point>`. Its constructor and accessor must preserve bytes and order.

Construction must not:

- append the first point;
- rotate the start point;
- reverse or normalize winding;
- remove duplicate or zero-length points;
- reject fewer than three points;
- calculate area, containment, simplification, or offsets.

Those behaviors belong to later cited polygon/Clipper slices.

### Chained layer domain

Create `mesh_slicer/chaining.rs` with:

- `OpenPolyline { start, end, points, length, consumed }`;
- `ChainedLayer { polygons, open_polylines }`;
- `chain_lines_by_triangle_connectivity(Vec<IntersectionLine>) -> ChainedLayer`.

`OpenPolyline.start` and `.end` retain tagged identity. `points` contains every
line start followed by the final line end. `length` is the ordered open length
in scaled coordinate units. `consumed` is initialized to `false`, matching the
upstream constructor, but Task 22C never mutates it; mutation belongs to the
next joining slice.

### Project ownership

Create `project_slice/chained_intersections.rs` with private
`ChainedPrintObject`, `ChainedVolumeIntersections`, and
`chain_project_intersections(Vec<IntersectedPrintObject>)`.

The wrapper consumes, rather than clones, raw project state and preserves:

- print-object order and complete `PlannedPrintObject`;
- volume order, one-based occurrence ordinal, and `ProjectVolumeType`;
- exact layer-slot count and order, including empty layers;
- polygon and open-polyline order produced by the core function.

`prepare_project_slice` may continue exposing raw state to the already-released
Task 22B private tests. The production `slice_project` path must move its
`intersected_objects` into `chain_project_intersections` immediately after
state preparation; it must not retain both raw and chained copies.

## Exact chaining algorithm

For one input layer:

1. Treat each raw vector index as immutable original face order.
2. Build separate flat start-reference vectors for `Edge(id)` and `Vertex(id)`.
   Each record is `(identity_id, original_raw_index)`; sort by both fields and
   build flat equal-ID ranges with one advancing cursor per range. The same
   numeric ID in different variants is never equal.
3. Within one identity, the range cursor retains original raw index as the
   deterministic FIFO tie-break. This removes the fixed C++ `std::sort`
   equal-key instability without changing which identity may connect.
4. Keep consumption in a function-local side table. Do not mutate or extend
   Task 22B records with request-shared state.
5. Scan raw indices from zero for the first unconsumed seed. Consume it and
   initialize the candidate point list with only `seed.a.point`.
6. Look up a successor only by `last.b.reference == candidate.a.reference`.
   Never connect by XY, inspect candidate B, or reverse a candidate.
7. Choose the first unconsumed FIFO candidate. The internal coordinate
   invariant is `last.b.point == candidate.a.point`; enforce it with
   `debug_assert_eq!`, not a public error or fallback.
8. Append `candidate.a.point`, consume the candidate, and repeat.
9. If no successor exists, compare `seed.a.reference` with
   `last.b.reference`:
   - equal tagged identity first requires
     `debug_assert_eq!(seed.a.point, last.b.point)`, matching Orca's separate
     closed-loop coordinate invariant, then produces `Polygon(points)` without
     appending the repeated final point;
   - unequal identity appends `last.b.point` and produces `OpenPolyline` with
     the seed A and last B identities.
10. Continue with the next unconsumed raw index until every line is consumed.

Every input line contributes exactly one edge. Therefore:

`sum(polygon.points.len()) + sum(open.points.len() - 1) == input.len()`.

The function retains one-line closed loops and one-line open paths. It does not
repair, drop, merge, sort, or reorient components.

### Open length arithmetic

For each adjacent point pair, convert each signed 64-bit coordinate to `f64`
before subtraction, then accumulate
`sqrt(dx * dx + dy * dy)` in point order. This avoids signed integer
subtraction overflow and matches the source's Euclidean norm shape. There is no
closing segment in open length.

### Complexity and resource boundary

The implementation must be `O(n log n)` overall with `O(n)` auxiliary memory:
sorting the two flat identity/index vectors and binary-searching flat identity
ranges may cost logarithmic time, while every candidate index is examined or
advanced past at most once by its range cursor. Do not allocate one queue per
identity and do not repeatedly rescan an equal-key group.

Task 22B already enforces independent one-million raw-line and dense-layer-slot
budgets. Closed output owns exactly one point per raw line; open output owns at
most one additional point per component. Task 22C adds no public-input option,
budget, validation, or error variant.

## Lifecycle and public behavior

The existing load, typed config, config block, Task 22A planning, and Task 22B
raw error precedence remains unchanged. Chaining is infallible over admitted
internal state and happens only after all raw intersections are available.

`slice_project` must traverse the chained state so production reachability is
compiler-checked, then continue returning
`SliceError::ProjectSlicingIncomplete`. CLI and WASM therefore preserve their
current incomplete-project behavior in this package.

Task 22C consumes no new Option. That is source-faithful: the cited function
does not read configuration. The fixture's `slicing_mode=regular` and
`slice_closing_radius=0.049` stay typed in the 3MF-derived config but are not
consumed until their owning upstream stages.

## TDD acceptance

Every new test name begins with `task22c_`. Test modules are separate from
production modules.

### Required synthetic REDs

Before production implementation, tests must fail for all of these contracts:

1. Three edge-referenced directed lines form one polygon in exact seed order,
   without a repeated first point.
2. A vertex-referenced cycle follows the same rule.
3. Two directed lines form one open polyline with exact start/end identities,
   three points, Euclidean length, and `consumed == false`.
4. Equal XY with unequal identities does not connect.
5. `Vertex(7)` and `Edge(7)` do not collide.
6. A line whose B matches the current B is not reversed into the chain.
7. In debug builds, matching successor identity with unequal coordinates and
   matching closure identity with unequal seed-A/last-B coordinates each
   trigger their source-derived internal assertion.
8. Multiple components retain original seed/output order.
9. Multiple candidates with the same start identity use original-index FIFO.
10. Empty, single-open, and single-closed layers preserve the source behavior.
11. Extreme signed coordinates prove conversion precedes subtraction in open
    length.
12. The edge-contribution conservation equation holds for mixed open/closed
    input.
13. The project wrapper preserves plan, ordinal, type, empty layer slots, and
    object/volume/layer order.

Hand-built synthetic expected points and lengths must be fixed before the
production function exists. Tests must not call legacy stitching helpers.

### Independent KSR fixture oracles

A read-only independent implementation parsed the committed 3MF and first
reproduced Task 22B's released facts: 6,109 vertices, 12,234 triangles, 18,351
mesh edge IDs, 460 layers, and 116,472 raw lines. It then applied only the
algorithm above and fixed these Task 22C topology values before Ares
implementation:

- 3,288 closed polygons;
- zero open polylines;
- 116,472 closed polygon points;
- every polygon omits a repeated first point;
- representative `(layer, raw lines, closed, open)` values:
  `(0,1046,12,0)`, `(2,932,12,0)`, `(12,1265,12,0)`,
  `(17,1138,12,0)`, `(37,880,15,0)`, `(46,3011,41,0)`,
  `(230,38,1,0)`, `(459,72,9,0)`;
- sorted polygon lengths on layer 0:
  `[67,68,69,70,71,80,80,80,80,80,88,213]`;
- layer 230: `[38]`;
- layer 459: nine polygons of length 8.

Encode each layer as the ASCII bytes `L<zero_based_layer>\n`. Encode each
closed polygon as `C;<point_count>;<x1>,<y1>;...;<xn>,<yn>\n`, using base-10
signed integers and LF bytes. A semicolon precedes every point, semicolons
separate adjacent points, and there is no semicolon after the final point.

The first pre-implementation coordinate oracle mistakenly subtracted the
3MF provenance fields `source_offset_x = source_offset_y = 128.5` from an
already-centered mesh. The first Package D run exposed the mismatch without
changing its constants. Two read-only independent replays then removed only
that invalid provenance offset, reproduced Task 22B's released first raw line
and `[-37.5,37.5]` / `[-35,35]` XY bounds, and independently produced the
same corrected bytes below. The build-item XY translation remains excluded at
this object-local slicing stage.

Two corrected independent digests are frozen:

- face/seed-order encoding, with no rotation or sorting:
  `6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`;
- semantic encoding, rotating each polygon to the earliest occurrence of its
  lexicographically smallest numeric `(x, y)` point without reversing it, then
  sorting the resulting `Vec<Point>` values per layer by numeric
  lexicographic sequence order:
  2,190,993 bytes and
  `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`.

The fixture test also proves repeat-run equality, preserves the existing
49,004-byte config-block SHA, and confirms the public API still returns
`ProjectSlicingIncomplete`. The committed reference G-code is not a raw-loop
oracle and must not be read by these tests.

Tests may embed the 3MF fixture with `include_bytes!`; that is test-data
embedding, not Rust source splitting. No test may open, hash, parse, or assert
the contents of an Orca `.cpp`/`.hpp` file.

## Exact code and test scope

Expected production paths:

- modify `crates/ares-core/src/geometry.rs`;
- create `crates/ares-core/src/geometry/polygon.rs`;
- modify `crates/ares-core/src/mesh_slicer.rs`;
- create `crates/ares-core/src/mesh_slicer/chaining.rs`;
- minimally widen only test/construction visibility in
  `mesh_slicer/intersection.rs` if the synthetic tests require it;
- modify `crates/ares-core/src/project_slice.rs`;
- create `crates/ares-core/src/project_slice/chained_intersections.rs`.

Expected test paths:

- modify `geometry/tests.rs` and create `geometry/tests/polygon.rs`;
- modify `mesh_slicer/tests.rs`;
- create `mesh_slicer/tests/chaining.rs` and, if needed for the line limit,
  `mesh_slicer/tests/chaining/{identity,open}.rs`;
- modify `project_slice/tests.rs`;
- create `project_slice/tests/chained_intersections.rs`;
- create `project_slice/tests/chained_fixture.rs` and
  `project_slice/tests/chained_fixture/{encoding,oracles}.rs`.

Every Rust source file must remain below 400 physical lines. Production source
splitting must use `mod`; `include!`, `include_bytes!`, `include_str!`, or
similar macros may not split Rust source. Do not grow the 359-line
`project_slice/raw_intersections.rs`; consume it through existing methods.

## Explicit deferrals

Task 22C intentionally defers, without authorizing a fallback:

- `TriangleMeshSlicer.cpp:1163-1175` open-polyline length sorting;
- `1177-1273` exact open joining, reverse-allowed retry, and closing direction
  heuristic;
- `1275-1381` endpoint search and 2 mm gap repair;
- `1428-1462` all remaining `make_loops` repair passes;
- `1484-1532` `slicing_mode` selection and positive/largest winding policy;
- `1664-1736` simple contour/hole ownership;
- `1738-1824` Clipper union, closing, offset, and `ExPolygon` construction;
- polygon area/contains/orientation/repair;
- `slice_closing_radius`, negative/modifier volume Boolean assembly, regions,
  surfaces, perimeters, infill, supports, extrusion entities, path ordering,
  G-code, filters, and final metadata;
- general `Polyline`, arc fitting, or any migration of the old STL pipeline.

## Verification and review gates

Before release, run at minimum:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22c_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22b_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22a_[^:]*$/)'
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check --workspace --all-targets
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown --tests
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown --tests
```

Also scan all Rust files for `<400` LOC, production source-splitting macros,
native threads/Rayon/unsafe/platform branches in the new path, source-pinning
tests, old project-pipeline fallback calls, fixture hash drift, and unexpected
tracked paths.

After implementation, one independent read-only review thread must assess the
same candidate across requirement completeness, logical correctness, edge
cases, code quality, test coverage, and actual execution results. It returns
one exact fix list. The main thread fixes that list and sends the identical
reviewer the revised bytes and fresh results. Repeat until all six dimensions
pass or a concrete blocker is recorded.

Architecture and roadmap documentation are updated only after production/test
approval. Then rerun the complete local matrix, commit conventionally, push,
and require a green exact-SHA Tier-1 run before marking Task 22C released.

## Acceptance criteria

Task 22C is complete only when:

1. production project slicing consumes Task 22B raw lines through the cited
   triangle-connectivity function and never enters the legacy STL pipeline;
2. tagged identity, direction, seed order, FIFO tie-break, closure, open
   endpoint, length, and conservation contracts pass synthetic tests;
3. the committed fixture produces the exact counts, point-length oracles, and
   both independently frozen digests above;
4. no new Option, public error, repair behavior, or hardcoded fixture branch is
   introduced;
5. every Rust file is below 400 LOC and production source splitting uses real
   modules only;
6. focused, workspace, strict lint, native, WASM, fixture, and audit gates pass;
7. the required independent six-dimensional review/fix/re-review loop passes;
8. tracked architecture/roadmap docs describe only implemented behavior and
   honest deferrals;
9. the conventional commit is pushed and exact-SHA Tier-1 is green.

Passing Task 22C does not complete the user-visible goal. The API still returns
`ProjectSlicingIncomplete`; the next source-cited package must start with
Orca's open-chain joining and gap-repair boundary.
