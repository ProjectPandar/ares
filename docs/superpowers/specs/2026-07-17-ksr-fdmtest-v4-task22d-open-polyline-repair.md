# Task 22D: Open-Polyline Repair and Loop Return

## Status and objective

This specification is a draft. Production and test implementation may begin
only after independent upstream/Ares reviewers and the repository's
default-model review gate approve these exact bytes.

Task 22D is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` parity program. Released Task 22C commit
`8c07319a5ac1f9660324ef53172ffe95d2b53230` consumes directed raw
triangle-plane intersections and returns ordered closed integer polygons plus
tagged open polylines. Task 22D ports only the remainder of Orca's
`make_loops` open-polyline repair seam: length ordering, exact identity joins,
the reversal-enabled retry, nearest-endpoint gap repair at the fixed 2 mm
radius, and the final closed-polygon return.

The new private result owns only closed polygons. Open polylines that remain
after all four repair passes are discarded at the same source boundary as
Orca. The public project API still returns
`SliceError::ProjectSlicingIncomplete` after traversing this result. Task 22D
does not apply `slicing_mode`, infer holes, invoke Clipper, create regions, or
emit G-code.

The committed fixture enters this boundary with exactly 3,288 closed polygons,
zero open polylines, and 116,472 closed polygon points across 460 layer slots.
It is therefore a strict no-op oracle for this package. Synthetic cases, fixed
before production code, prove every joining and closing branch.

## Fixed upstream rewrite boundary

All source citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored workspace checkout
may have another HEAD; citations are verified with `git show <sha>:<path>`
without changing it.

- `src/libslic3r/libslic3r.h:38-44` defines signed 64-bit `coord_t`.
- `libslic3r.h:60-70,92-94` defines normal/large coordinate scaling and
  `scale_`.
- `src/libslic3r/Point.hpp:41-43,60-64,187-203` defines integer points and
  ordered point storage.
- `Point.hpp:495-500,502-637` defines the source spatial hash and
  `ClosestPointInRadiusLookup`.
- `src/libslic3r/MultiPoint.hpp:15-44,172-187` defines ordered storage,
  reversal, open length, and the signed-area helper used by this seam.
- `src/libslic3r/Polygon.hpp:14-17,23-62` defines closed polygon storage and
  its minimum useful size.
- `src/libslic3r/Polygon.cpp:52-78` is adjacent area/orientation behavior; the
  repair code itself calls the `Points` helper from `MultiPoint.hpp`.
- `src/libslic3r/TriangleMeshSlicer.cpp:58-145,1043-1056` defines tagged
  intersection identity, directed lines, and mutable `OpenPolyline` state.
- `TriangleMeshSlicer.cpp:1163-1175` defines unconsumed-polyline length
  ordering.
- `TriangleMeshSlicer.cpp:1177-1273` defines exact identity joining.
- `TriangleMeshSlicer.cpp:1275-1381` defines nearest-endpoint gap repair.
- `TriangleMeshSlicer.cpp:1414-1480` defines the four-pass order and returns
  only closed loops.

The semantic stop is the `return loops` statement at line 1480. The outer
per-layer/TBB function beginning at line 1483 is not part of Task 22D.

## Existing Ares input seam

Task 22D consumes only released Task 22C state:

- `geometry::Point`: request-scaled signed 64-bit XY coordinates;
- `geometry::Polygon`: an ordered closed point vector without a repeated
  terminal point;
- `mesh_slicer::OpenPolyline`: tagged start/end identities, ordered points,
  cached scaled length, and mutable consumed state;
- `mesh_slicer::ChainedLayer`: ordered closed polygons followed by ordered
  open polylines;
- `project_slice::chained_intersections::ChainedPrintObject`: print plan,
  volume occurrence/type, and exact layer-slot ownership.

The implementation may not call or adapt the legacy f64 STL
`planning`/`segments`/`contours`/`pipeline` path. It may not turn a reference
G-code file or fixture coordinates into production behavior.

## Rust destination and ownership

### Core loop result

Add private `LoopedLayer { polygons: Vec<Polygon> }` and
`make_loops(ChainedLayer, max_gap_scaled: Coord) -> LoopedLayer` below the
existing `mesh_slicer::chaining` boundary. The function consumes each input
layer once, keeps Task 22C polygons in their original order, appends newly
closed polygons in pass/seed order, then drops residual opens.

Exact joining and gap joining live in child modules of `chaining`; mutable
open-polyline fields are not made public. The spatial lookup is a request-local
child module. No general public Polyline, area, orientation, or spatial-index
API is introduced.

### Request-local scale

The `CoordinateScale` already selected from the resolved printable area while
preparing project intersections will be retained privately in
`ProjectSliceState`. `slice_project` derives exactly one internal repair radius with
`scale.checked_scale(2.0)`. This produces 2,000,000 normal units or 200,000
large-bed units mathematically. The fixed source casts floating division to an
integer, however, and binary `2.0 / 0.00001` truncates to 199,999. Ares'
existing `checked_scale(2.0)` reproduces that result, so the accepted integer
radii are exactly 2,000,000 normal units and 199,999 large-bed units. Failure
is an internal invariant violation, not a public input error.

The 2 mm value comes directly from `TriangleMeshSlicer.cpp:1459-1461`. It is
not a new Option and is unrelated to the already parsed
`slice_closing_radius`. No fixture-specific scale or geometry branch is
allowed.

### Project ownership

Add a private looped-intersection wrapper that consumes
`Vec<ChainedPrintObject>` and preserves:

- print-object order and complete `PlannedPrintObject`;
- volume order, one-based occurrence ordinal, and `ProjectVolumeType`;
- exact layer-slot count and order, including empty slots;
- polygon order produced by `make_loops`.

The production path is exactly:

`raw intersections -> chained intersections -> looped intersections ->`
`ProjectSlicingIncomplete`.

It never retains two independently mutable copies of a stage.

## Ordered pass contract

For every `ChainedLayer`, run exactly:

1. exact identity joining with reversal disabled;
2. exact identity joining with reversal enabled;
3. nearest/gap joining at 2 mm with reversal disabled;
4. nearest/gap joining at 2 mm with reversal enabled;
5. return closed polygons and discard all surviving open polylines.

The disabled source loop over multiple radii is not ported. Debug SVG output
is not core behavior.

## Sorting and deterministic ties

`open_polylines_sorted` returns only unconsumed entries. Exact passes use the
current cached lengths without recomputing them. Both gap passes recompute the
length from the current point sequence before sorting.

Orca compares only length and relies on `std::sort`; equal ordering is
unspecified. Ares freezes source-unspecified ties as:

1. descending cached or recomputed length;
2. original Task 22C open-polyline index ascending.

The original index never changes even when a polyline absorbs other entries.

Open length is the sum of adjacent Euclidean segment lengths without a closing
segment. Coordinates subtract in `i128`; the exact differences then convert to
`f64` for the Euclidean norm.

## Exact identity joining

### Lookup identity

The exact lookup reproduces Orca's signed key:

- `Vertex(n) -> +n`;
- `Edge(n) -> -n`.

The key uses Rust `i64`, so valid Ares `u32` identities cannot overflow.
`Vertex(0)` and `Edge(0)` intentionally collide at key zero because
that collision is observable fixed-source behavior. Tagged identities remain
stored on each polyline. Final closure still requires equal variants and equal
numeric IDs; a cross-variant zero-key join does not itself close a loop.

When multiple unconsumed endpoints have the same signed key, Ares chooses the
smallest original polyline index, then start before end. Reversal-disabled
mode indexes starts only. Reversal-enabled mode indexes both starts and ends.

### Attachment

For each unconsumed seed in sorted order:

1. mark the seed consumed;
2. look up its current tagged end through the signed key;
3. skip all consumed candidates;
4. when attaching a candidate start, append its points forward while omitting
   the first candidate point unconditionally;
5. when attaching a candidate end, append in reverse while omitting the
   former final candidate point unconditionally;
6. add the candidate's cached length to the seed length;
7. clear the candidate points, set its length to zero, and leave it consumed;
8. continue until closure or no candidate exists.

The source does not validate candidate coordinate equality in this function;
Task 22D must not add an error, assertion, gap point, or fallback there.

### Fixed non-reversed-pass quirk

In reversal-disabled mode, appending points does not update the seed's tagged
end identity. Every subsequent lookup therefore repeats the original seed-end
key. This may absorb several candidates sharing that start key, but cannot
close an admitted Task 22C open whose start/end tags differ. This is fixed
source behavior and must be ported literally, not corrected.

In reversal-enabled mode, attaching a candidate updates the seed end to the
candidate's opposite tagged endpoint and updates the lookup record used if the
expanded seed later becomes available to another seed.

If no candidate exists, the seed becomes unconsumed again; already absorbed
candidates remain consumed and empty.

### Exact closure

Closure requires tagged start/end equality. When closed:

- pop the final point unconditionally, without a coordinate check;
- discard fewer than three remaining points;
- in reversal-enabled mode only, reverse a negative-signed-area point vector;
- append every retained polygon after existing polygons in pass/seed order;
- clear the seed points and leave it consumed.

Area zero is not reversed. Reversal-disabled exact joining never normalizes
winding.

## Nearest-endpoint gap repair

Each gap pass builds a deterministic request-local spatial lookup from the
currently unconsumed polylines. Reversal-disabled mode indexes starts only;
reversal-enabled mode indexes starts and ends. Equal-distance candidates are
chosen by squared distance, original polyline index, then start before end.

The lookup must be spatial, not a full all-pairs rescan. A deterministic
integer cell grid may differ from Orca's unordered container while preserving
the nearest-within-radius semantics. Cell arithmetic uses a wider signed
domain and supports negative/extreme admitted coordinates. With cell size
equal to the radius, each query visits the complete fixed 3-by-3 neighborhood,
which covers every point strictly inside the radius.

Distance arithmetic first subtracts signed coordinates in `i128`. A strict
radius query rejects a candidate when either exact absolute component is at
least the radius; otherwise it computes the exact `u128` squared distance.
Nearest ordering, closure eligibility, closure-vs-candidate ordering, and
radius equality therefore remain exact even when the absolute coordinates are
near `i64::MIN` or `i64::MAX`. Only an already bounded squared distance is
converted to `f64` for the square root in the 30% heuristic.

The threshold is strict: candidates and closure endpoints at exactly 2 mm are
excluded. Squared-distance comparisons are performed before square roots.

For every seed:

1. remove its mutable end record in reversal-enabled mode, then mark it
   consumed;
2. find the deterministic nearest unconsumed indexed endpoint strictly inside
   the radius;
3. compute the seed's current end-to-start closing distance;
4. initially allow closure only when that distance is strictly inside the
   radius;
5. if a candidate exists, closure is allowed, and closure is strictly closer
   than the candidate, additionally require
   `closing_distance < 0.3 * current_point_sequence_length`;
6. if distances are equal, the 30% heuristic is not entered and the loop
   closes; if the candidate is closer than closure, the source also closes;
7. if no candidate exists, a sub-radius closure closes without the 30%
   heuristic.

On gap closure:

- remove the seed start from the lookup;
- pop the final point only when closing distance is exactly zero;
- retain both endpoints for a nonzero closing bridge;
- discard fewer than three points;
- reverse negative area only in reversal-enabled mode and only when more than
  one source polyline was joined;
- append the retained polygon, clear the seed, and leave it consumed.

If the loop is not closed and no candidate exists, restore the seed to
unconsumed and, in reversal-enabled mode, reinsert its changed end.

When attaching a candidate start, append forward; when attaching its end,
append in reverse. Omit the joining endpoint only if it is coordinate-equal to
the seed's current final point. For a nonzero gap, retain both endpoints so the
bridge is explicit. Remove all candidate records, clear its points, and leave
it consumed. Do not update cached length within the pass and do not update the
tagged end identity.

## Arithmetic, complexity, and portability

- Coordinate and identity storage remains signed 64-bit points plus tagged
  `u32` identities.
- Open length first subtracts coordinates in `i128`, then converts the exact
  differences to `f64` for the Euclidean norm.
- Radius admission and nearest/closure distance ordering use exact bounded
  `u128` squared distances as described above.
- Signed area first performs each coordinate sum/difference in `i128`, then
  converts those safe intermediates to `f64` for the source-shaped product and
  accumulation.
- Grid coordinates and neighbor arithmetic use a wider signed integer domain.
- Exact indexing is `O(n log n)` with `O(n)` auxiliary memory.
- The spatial grid uses `O(n)` entries and avoids global `O(n^2)` scanning;
  dense single-cell inputs remain the source-equivalent candidate-density
  worst case and receive a bounded stress test.
- No filesystem, native thread, Rayon, `unsafe`, platform branch, or mutable
  global state enters this path. The same code builds for WASM, Windows,
  macOS, and Linux.

## Lifecycle and public behavior

Load, typed 3MF config, config-block output, Task 22A planning, Task 22B raw
intersection error precedence, and Task 22C chaining remain unchanged.
Repair is infallible over admitted internal state and introduces no public
error variant.

`slice_project` must traverse looped object/volume/layer/polygon state so the
new boundary is production-reachable, then return
`SliceError::ProjectSlicingIncomplete`. CLI and WASM behavior therefore stays
explicitly incomplete at this package boundary. No legacy or approximate
G-code fallback becomes observable.

## TDD acceptance

Every new test name begins with `task22d_`. Tests live in separate test
modules, not production files. Hand-built expected points, order, orientation,
and consumed/discard behavior are fixed before production functions exist.

### Exact-join RED contracts

1. Length sorting is descending with original-index ties; exact passes retain
   cached rather than recomputed lengths.
2. Reversal-disabled joining appends starts forward and preserves the fixed
   stale-end quirk across multiple candidates.
3. Reversal-enabled joining can attach a start or an end, advances the live
   end, and preserves lookup reachability after a failed expanded seed.
4. Equal exact candidates use original index then start-before-end.
5. `Vertex(0)`/`Edge(0)` collide for lookup while tagged closure remains
   variant-strict.
6. Exact joining omits the candidate joining point without checking coordinate
   equality.
7. Exact closure pops the terminal point unconditionally and drops results
   shorter than three points.
8. Only reversal-enabled exact closure reverses negative signed area.
9. Failed seeds become unconsumed while absorbed candidates remain cleared and
   consumed.

### Gap/spatial RED contracts

10. Gap passes recompute surviving lengths before each sort.
11. Nearest lookup accepts integer distances strictly below the source-scaled
    radius and rejects equality under both scale modes: 2,000,000 normal units
    and 199,999 large-bed units.
12. Negative and extreme coordinates use correct cells/distances without
    overflow; an extreme absolute base plus a difference exactly equal to each
    scale's radius is rejected rather than rounded inside.
13. Equal distances use original index then start-before-end.
14. Same-direction and reversal-enabled attachment retain nonzero-gap joining
    endpoints but omit coordinate-equal endpoints.
15. Zero-distance closure pops the duplicate; nonzero closure keeps both
    endpoints.
16. The closure-vs-candidate branch, equality branch, candidate-closer branch,
    no-candidate branch, and strict 30% test all match the source order.
17. Reversal-enabled gap closure normalizes negative area only after more than
    one source polyline was joined.
18. Failed reversed seeds reinsert their changed end; residual opens are
    discarded by `make_loops`.
19. A dense-cell deterministic stress case remains repeatable and does not
    allocate per query.

### Project and fixture RED contracts

20. The project wrapper preserves plan, occurrence ordinal/type, object,
    volume, empty-layer-slot, layer, and polygon order while consuming each
    stage once.
21. Both scale modes derive the physical 2 mm threshold from the retained
    request scale, not from an Option.
22. Production `slice_project` reaches the looped stage and still returns
    `ProjectSlicingIncomplete`.
23. The real fixture remains 460 layer slots, 3,288 polygons, zero Task 22C
    opens, 116,472 points, and produces unchanged face/semantic encodings.
24. Repeated fixture repair is byte-identical and the config-block hash is
    unchanged.

No test may read, hash, parse, or execute Orca `.cpp`/`.hpp` files. No Task 22D
test may read the reference G-code. Source pinning belongs in reviewed design
documents, not executable tests.

## Frozen real-fixture oracle

The Task 22C independent encoding remains authoritative:

- layers: 460;
- closed polygons: 3,288;
- open polylines before Task 22D: 0;
- closed points: 116,472;
- encoded bytes: 2,190,993;
- face/seed-order SHA-256:
  `6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`;
- normalized semantic SHA-256:
  `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`;
- config block: 49,004 bytes with its already released Task 22C SHA.

Face-order encoding remains `L<layer>\n` followed by
`C;<count>;<x1>,<y1>;...;<xn>,<yn>\n`. Semantic encoding rotates to the
earliest lexicographically smallest point without reversing and sorts polygon
point vectors per layer. Task 22D applies the same encoder to `LoopedLayer`.
Expected constants may not be regenerated from Ares output.

## Exact code and test scope

Expected production modifications:

- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/chaining.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/state.rs`.

Expected production additions:

- `crates/ares-core/src/mesh_slicer/chaining/exact.rs`;
- `crates/ares-core/src/mesh_slicer/chaining/gaps.rs`;
- `crates/ares-core/src/mesh_slicer/chaining/gaps/spatial.rs`;
- `crates/ares-core/src/project_slice/looped_intersections.rs`.

Exact, gap, and spatial unit tests live in separate `tests.rs` child modules
below their production modules. End-to-end core-loop tests live below
`mesh_slicer/tests/chaining`, and project ownership/fixture tests live below
`project_slice/tests`, including a separate looped fixture encoder.

Every Rust file remains below 400 physical lines. Production source splitting
uses `mod`; `include!`, `include_bytes!`, `include_str!`, or related macros may
not split Rust source. Test fixture embedding may continue using
`include_bytes!` only for non-source 3MF data.

## Explicit deferrals

Task 22D intentionally defers, without authorizing a fallback:

- `TriangleMeshSlicer.cpp:1483-1532` TBB scheduling, cancellation, and
  `slicing_mode` selection;
- slab-loop behavior beginning at line 1535;
- contour/hole ownership around line 1664 onward;
- Clipper union/closing/offset and `ExPolygon` construction around line 1738;
- `slice_closing_radius` consumption;
- general public polyline/spatial/area/containment/orientation APIs;
- negative/modifier volume booleans, regions, surfaces, perimeters, fill,
  support, extrusion entities, path ordering, G-code generation, filters, and
  final metadata;
- any migration or fallback to the old Ares STL pipeline.

## Verification and review gates

Before release, run at minimum:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22d_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22c_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22b_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22a_[^:]*$/)'
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check --workspace --all-targets
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown --tests
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown --tests
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
```

Also run the existing real-3MF browser/WASM gate, scan every Rust file for
`<400` LOC, audit production splitting macros, unsafe/threads/Rayon/platform
branches, executable Orca-source pinning, legacy fallback reachability,
reference-G-code reads, fixture drift, and unexpected tracked paths.

After implementation, one independent read-only review thread assesses the
same candidate across requirement completeness, logical correctness, edge
cases, code quality, test coverage, and actual execution results. It returns
one exact fix list. The main thread alone fixes the list, reruns affected and
full gates, and asks the identical reviewer to revalidate revised bytes. The
loop continues until all six dimensions pass or a concrete blocker is
recorded.

Architecture and roadmap documents are updated only after code/test approval.
Then the complete matrix is rerun, the exact manifest is committed
conventionally and pushed, and the exact pushed SHA must pass Tier-1 format,
Linux, WASM/browser, macOS, and Windows jobs.

## Acceptance criteria

Task 22D is complete only when:

- the four repair passes and final open discard match the fixed source;
- the source's stale-end and branch-order quirks are tested rather than fixed;
- only source-unspecified ties receive documented deterministic ordering;
- the 2 mm constant comes from the request-local scale, not a 3MF Option;
- project ownership and production reachability are proven;
- synthetic RED/GREEN tests cover every listed branch;
- the real fixture remains byte-identical at this zero-open boundary;
- no source-pinning test, old pipeline fallback, hardcoded fixture branch, or
  cross-platform divergence is introduced;
- all local, structural, review-loop, push, and exact-SHA Tier-1 gates pass;
- the next upstream rewrite boundary is recorded without claiming final G-code
  parity.

**Status: DRAFT — implementation is forbidden until fresh independent and
default-model reviewers approve these exact spec/plan bytes.**
