# Task 22O.28 — Crate-private ClipperZ `wave_seeds`
## Status and source boundary

Approved implementation specification. Local implementation and preliminary
six-dimensional/default-model OpenCode reviews are complete; final documented
review and exact pushed-SHA Tier-1 remain release gates. Exact predecessor
`f361bb73b558b4e50bfa4fa712afcd63df44ba9f` contains shipped O27; its Tier-1
run `31127440442` is green. O27's own exact-SHA run `31126818275` is also green.
Rewrite target: OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Goal: port pure crate-private `Algorithm::wave_seeds` by adding only the
per-vertex Z provenance required by the single ARD-0024 indexed Clipper 6
kernel; this is not an Ares-owned pipeline.
Normative sources:
- API, comparator, algorithm: `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.hpp:38-68`; `RegionExpansion.cpp:88-391`.
- Four-direction merge: `OrcaSlicer/src/libslic3r/Polyline.hpp:232-250`.
- Z conversion/visitor: `OrcaSlicer/src/libslic3r/ClipperZUtils.hpp:14-139`.
- XYZ records/callback: `OrcaSlicer/deps_src/clipper/clipper.hpp:46-47,99-135,230-279,441-479,500-533`.
- XY-only XYZ equality, `SetZ`, output points, horizontal/top Z, PolyTree flattening: `OrcaSlicer/deps_src/clipper/clipper.cpp:78-113,472-479,1617-1683,2002-2040,2284-2314,2588-2643,4121-4166`.
- Offset factor: `OrcaSlicer/src/libslic3r/ClipperUtils.hpp:33-39`.
- AABB build/traversal: `OrcaSlicer/src/libslic3r/AABBTreeIndirect.hpp:37-210,221-236,940-987`.
- Binding local constraints: `docs/architecture/ard-0024-safe-indexed-clipper6-kernel.md`; `docs/roadmap.md:4946-4993`.
## Scope
Included: expanded/opened source Z paths; boundary Z paths; NonZero open-subject/closed-clip intersection; Z-fill provenance; PolyTree Z flattening; exact split reconciliation; four ID-recovery branches; lazy AABB fallback; optional source-comparator sorting; focused differential tests.
Deferred: source-taking `propagate_waves` and scalar overloads; `propagate_waves_ex`; `expand_expolygons`; all expansion merge helpers; `clipper_round_offset_error`; O27 behavior changes; public ClipperZ/AABB APIs; `LayerRegion`/`PrintObject` external-surface orchestration; expansion zones, bridge grouping/angles/direction detection, surface rebuilding, minimum-sparse-area conversion; project lifecycle/checkpoints/cancellation/transactions/incomplete-sink movement; CLI/WASM/browser exports; fill/toolpath/seam/motion/G-code/post-processing work; full KSR seed/G-code parity.
Public slicing must retain the current O26/O27 lifecycle and `ProjectSlicingIncomplete`.
## Crate-private API
```rust
pub(crate) fn wave_seeds(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    tiny_expansion: f32,
    sorted: bool,
    scale: CoordinateScale,
) -> Result<Vec<WaveSeed>, ClipperError>;
```
- `CoordinateScale` replaces Orca’s mutable global scale only for `SCALED_EPSILON`; `tiny_expansion` is already scaled.
- Return existing `WaveSeed { src: u32, boundary: u32, path: Polygon }`.
- Add a function-pointer signature assertion in `geometry/region_expansion.rs`; expose nothing through `lib.rs`, CLI, or WASM.
## One-kernel metadata design
ARD-0024 remains non-negotiable: one safe indexed Clipper 6 engine; no alternate Boolean engine, FFI/C++, Clipper2, geometry fallback, unsafe/self-referential graph, exposed arena ID, mutable global, platform branch, hash-order dependency, or canonicalization.
Define in `geometry/clipper/z.rs` a `KernelPoint { xy: Point, z: i64 }` and
`ZPath = Vec<KernelPoint>`. Both types, their explicit full-XYZ helpers, and the
Z add/execute accessors use `pub(in crate::geometry)` visibility: wave-seed
code may inspect them, but neither the crate API nor adapters may expose them.
- Public/crate geometry `Point`, `Polygon`, and `Polyline` remain exactly 2-D.
- Ordinary `KernelPoint` equality is XY-only, matching `clipper.cpp:109-110`; Z never affects input cleanup, closure, edge equality, output deduplication, ring cleanup, or joins.
- Full `(x,y,z)` equality and `(x,y,z)` lexicographic comparison exist only as explicit helpers; do not derive an incompatible `Ord`.
- Use `KernelPoint` for `Edge.current/bottom/top`, `IntersectionNode.point`, `OutPoint.point`, and `Join`/`GhostJoin.offset`.
- Existing 2-D add-path adapters assign Z zero; existing outputs discard Z and must remain ordered-identical.
Add narrow private Z-path add/execute methods to the same `Clipper`; they share its edge/minima/scanbeam/AEL/SEL/intersection/output/free-list/join/PolyTree machinery and range checks.
Extend `PolyNodeRecord` with an optional Z vector parallel to its existing contour; normal execution stores none, not a second tree.
`geometry/clipper/polytree/z_paths.rs` must consume the existing tree root-left-to-right in preorder, emitting every nonempty open/closed contour with zipped Z values and no filtering, rotation, closure change, or canonicalization, matching `PolyTreeToPaths`.
Z execution owns and returns a fresh `Vec<(i64,i64)>` intersection table; no borrowed/global callback state.
### Exact `SetZ`
Before any `intersect_edges` open/closed branch:
1. Retain a nonzero candidate Z.
2. Otherwise compare candidate XY against first-edge bottom, first-edge top, second-edge bottom, second-edge top, in that order.
3. Copy the first matching endpoint Z; invoke Z-fill only if none matches.
Kernel audit requirements:
- horizontal reversal swaps endpoint X and Z together;
- before the closed-output `AddOutPt` at `clipper.cpp:2292-2300`, fill the
  crossing edge's complete `Curr` in place: left-to-right calls `SetZ(Curr,
  horizontal, crossing)` and right-to-left calls `SetZ(Curr, crossing,
  horizontal)`, then writes that filled point;
- the strictly-simple type-3 top touch at `clipper.cpp:2637-2643` copies the
  current candidate, applies `SetZ(candidate, previous, current)`, and uses that
  same filled complete point for both output writes and the join offset;
- scanbeam X projection preserves current Z;
- top update sets current Z to top Z when `top_y == top.y`, else bottom Z when `top_y == bottom.y`, else zero;
- edge promotion copies complete metadata;
- output allocation and `DupOutPt` copy the complete original XYZ point;
- immediate XY-only `AddOutPt` dedup retains the already stored point's Z;
- `FixupOutPolyline` and `FixupOutPolygon` perform the pinned XY-only removal
  sequence (`clipper.cpp:2694-2760`): removed nodes contribute no Z and each
  surviving node retains its own complete original XYZ point;
- join duplication/replacement follows `clipper.cpp:2927-3015` exactly: copied
  nodes copy complete XYZ, and any point assignment overwrites both XY and Z
  from that exact source point. No cleanup or join may average, reconstruct,
  prefer nonzero Z, or transfer Z independently of the pinned node operation.
### Z-fill collector
Sort the four endpoint Z values ascending with a fixed four-element insertion routine and deduplicate numerically.
- One unique value: copy it as self-intersection provenance.
- Two or more: `debug_assert` exactly two, store the first two sorted values, and set output Z to `-(table.len() as i64)`.
- Preserve pinned release behavior: if three/four distinct values bypass the debug assertion, still store the first two; do not invent an error.
Recovery computes `-z - 1` in signed `i64` before `usize`.
### Reuse
`Clipper::clear` clears Z-bearing edges, active/output state, sidecars, and collector state; a subsequent run sees no stale labels or indices.
Each Z execution starts and ends without an active prior collector.
Existing normal rerun/allocation order remains unchanged.
`ClipperOffset::clear` remains source-compatible: clear paths/lowest while retaining configuration.
## `wave_seeds` semantics
### Entry, errors, IDs
- Assert `tiny_expansion > 0.0` before emptiness; this rejects NaN.
- If either input is empty, return `Ok([])` without offsetter, Clipper, or AABB construction.
- Add/range-check boundary paths before source expansion to preserve first-error precedence.
- Only existing `ClipperError` values may escape; add no error wrapper, overflow error, validation layer, or geometry fallback.
Use signed `i64` Z IDs:
- `idx_boundary_begin = 1`; each boundary `ExPolygon` gives its contour and holes one ID, incremented once per `ExPolygon`.
- `idx_boundary_end` begins the source range; each source `ExPolygon` similarly gets one ID, incremented after all contours even if offset emits zero/multiple paths.
- Valid ranges are boundary `[1, idx_boundary_end)` and source `[idx_boundary_end, idx_src_end)`.
- Final IDs use signed subtraction followed by `as u32`, matching upstream narrowing; no new count guard.
### Boundary paths
Visit boundaries in input order, contour then holes in stored order; tag each path uniformly, keep it closed without appending a point, and add it as closed `Clip`.
### Expanded/opened source paths
Use one reusable 2-D `ClipperOffset` with `JoinType::Square` and `shortest_edge_length = f64::from(tiny_expansion) * 0.005`.
For each source `ExPolygon`, visit contour then holes:
- clear offsetter before each independent contour;
- add only that contour as `ClosedPolygon`;
- execute with `+f64::from(tiny_expansion)` for contour and the negative value for holes;
- preserve every emitted path and its Clipper output order;
- only after offsetting, assign every emitted vertex the current source Z and append one exact copy of its first `(x,y,z)` point;
- increment source Z once per `ExPolygon`.
Do not extend `ClipperOffset` with Z, union outputs, sort them, canonicalize them, or merge them.
### Split registry and clipping
For each opened source path, `debug_assert` length at least two and repeated full endpoint; store `(endpoint,-1)`.
Sort registry records by X, Y, then Z with the fixed ARD-0024 MSVC sort control flow and no tie-break.
Add sources as open `Subject`; execute `Intersection` into PolyTree with `NonZero/NonZero`; flatten all paths in exact preorder.
### Exact split reconciliation
Scan mutable flattened paths by index; `debug_assert` length at least two.
- Closure is XY-only; closed paths are untouched.
- For an open path, lower-bound the front first, then back only if front is absent; lower-bound compares `(x,y,z)`, while final endpoint match is XY-only.
- First fragment writes its vector index into the registry.
- Second fragment merges into that earlier destination using exact `polylines_merge`:
  - destination-front/source-front: reverse destination;
  - destination-front/source-back: swap destination and source;
  - destination-back/source-back: reverse source;
  - destination-back/source-front: no reversal;
  - append the entire source, retaining the duplicate junction.
- If current is last, pop and stop; otherwise move the last path into current, pop, and reprocess the same slot.
No map/group merge, stable erase, unconditional post-swap increment, or junction deduplication is conforming.
### Four recovery branches
Process reconciled paths in their current order and discard Z only after selecting IDs.
1. **Open, both endpoint Z nonnegative:** scan all points for the first source-range Z and first boundary-range Z, stopping when both exist. Missing source drops. Both known emits directly. Source-only samples front XY through lazy AABB and emits only on success; failure drops. Do not port unused `iseed`.
2. **Rare repair:** when front/back are XY-equal and `front.z < idx_boundary_end`, scan all points; for every `z >= idx_boundary_end`, replace local front and back, so the last match wins. Preserve the source’s absent upper-bound check, then continue.
3. **Intersection:** a pair is valid only if first is a boundary ID and second a source ID. Try negative front first, then negative back only if no valid front; emit `src = (second-idx_boundary_end) as u32`, `boundary = (first-1) as u32`.
4. **Closed fallback:** without a valid intersection, `debug_assert` XY closure and front source range; lazily sample front XY; `debug_assert` success and emit only on success, otherwise drop in release.
Emitted XY paths retain exact order, start point, duplicated junctions, and existing closure.
## Lazy boundary AABB
Implement only in `geometry/region_expansion/wave_seeds/aabb.rs`; instantiate on the first Branch 1/4 request.
- One leaf per boundary `ExPolygon`, using outer-contour bbox only.
- Inflate inclusively by 100 fixed units at normal scale or 10 at large-bed scale.
- Allocate implicit full tree length `2 * next_power_of_two(n) - 1`; internal nodes union their ranges.
- Choose longest bbox axis, X on ties.
- Preserve source centroid arithmetic coordinate-wise as `min + max / 2` with signed truncation toward zero; do not use `BoundingBox::center`.
- Preserve `AABBTreeIndirect.hpp:130-210` median-of-three/QuickSelect swaps and comparisons without index tie-break.
- Traverse root, prune non-containing inclusive boxes, then left before right; stop at the first containing leaf.
- Exact containment with `border_result=true`: outer point-in-polygon must be nonzero; a hole excludes only positive interior, so a hole boundary remains contained.
No generic/public AABB, R-tree, hash lookup, linear lowest-index selection, or right-first traversal.
## Output order and assertions
`sorted=false` preserves PolyTree order modified only by split swap-pop and recovery drops.
`sorted=true` uses comparator `(boundary,src)` only and the fixed MSVC STL 14.44 control flow; sort a `Vec<usize>` permutation and move non-`Copy` seeds accordingly, without index/geometry tie-break or host/stable sort.
Coordinates and Z are `i64`; Z affects no geometry predicate; offset delta remains source-ordered `f32`→`f64`.
Use unconditional assertion only for the entry precondition; use `debug_assert` for trusted path cardinality/closure, source endpoint topology, exactly-two labels, pair ranges, closed source range, and expected containment.
The only deliberate drops are Branch 1 missing source, Branch 1 failed source-only containment, and Branch 4 failed containment; malformed internal arenas receive no defensive recovery.
## Concrete file boundary
Existing files may change only where required: `geometry/clipper.rs`,
`clipper/types.rs`, `clipper/engine.rs`, `clipper/predicates.rs`,
`clipper/input.rs`, `clipper/input/path.rs`, `clipper/input/bounds.rs`,
`clipper/intersections.rs`, `clipper/intersections/open.rs`,
`clipper/intersections/top.rs`, `clipper/horizontals.rs`, `clipper/minima.rs`,
`clipper/strictly_simple.rs`, `clipper/output/rings.rs`, `clipper/output/fixup.rs`,
`clipper/output/append.rs`, `clipper/output/join_points.rs`,
`clipper/output/ownership.rs`, `clipper/polytree.rs`, `clipper/ordering.rs`,
`geometry/region_expansion.rs`, and matching test module roots. The two newly
listed sweep touchpoints permit mechanical `Point`/`KernelPoint` conversions;
`horizontals.rs` additionally permits only the exact direction-sensitive
closed-horizontal `SetZ` fill above, and `strictly_simple.rs` additionally
permits only the exact type-3 `SetZ` behavior above, both identified by
preliminary implementation review. They add no other behavior.
`active_edges.rs`, `bounds.rs`,
`winding.rs`, `output/open_fixup.rs`, `output/joins.rs`, and `output/simple.rs`
must compile unchanged through XY-only access/equality seams. Additional kernel
touchpoints require a spec amendment before editing, not silent scope expansion.
New production shards: `geometry/clipper/z.rs`, `geometry/clipper/polytree/z_paths.rs`, `geometry/region_expansion/wave_seeds.rs`, `wave_seeds/splits.rs`, `wave_seeds/aabb.rs`.
New test shards: `geometry/tests/clipper/z/{input_fill,output,lifecycle}.rs` plus module root; `geometry/tests/region_expansion/wave_seeds/{expanded,splits,recovery,aabb_order,oracle}.rs` plus module root.
No project lifecycle, adapter, public API, manifest, lockfile, or offset-kernel
metadata file may change. Documentation must amend ARD-0024 narrowly to record
the geometry-private Z extension of the same indexed engine, and must add O28
scope/exit evidence to `docs/roadmap.md` and
`docs/architecture/option-parity-v4.md`; this is not a new architecture or
second engine.
Every Rust file must remain below 400 physical lines; every new shard must be at most 300; no `include!` or source concatenation.
## TDD and differential gates
Write failing tests before production behavior.
Kernel tests must cover unchanged ordered 2-D output; XY-only Z cleanup,
closure, immediate dedup, exact fixup survivor identity, `DupOutPt`, and join
copy/overwrite behavior; horizontal reversal, both direction-sensitive horizontal-output fills,
strictly-simple type-3 fill, top/promotion metadata; all endpoint-priority
cases; callback bypass; self-intersection; negative indexing; debug-panic and
release first-two behavior for >2 labels; output-Z retention; PolyTree preorder;
and clear/reuse.
Expansion tests must cover outer/hole signs; exact shortest-edge equality; contour/hole shared ID; multiple/zero outputs; per-`ExPolygon` increment; exact repeated endpoint; emitted order.
Split tests must cover four direction combinations, front precedence, `(x,y,z)` lower-bound plus XY match, duplicate junction, last pop, middle move/reprocess, moved-fragment merge, closed no-op, and exact vector order.
Entry tests must prove the positive assertion precedes both empty cases, each
empty side short-circuits without geometry, and boundary range/add errors
precede source expansion/errors. Recovery tests must cover direct both-positive
IDs, source-only fallback success/failure, missing-source drop, last-source rare
repair, front-negative precedence, debug-valid front/back behavior, release-only
invalid-front/valid-back continuation, closed outer containment, hole interior
rejection, hole-boundary containment, release-only final containment failure
drop, and unchanged XY paths.
AABB/sort tests must cover both epsilon scales, contour-only bbox, X axis tie, negative `min+max/2`, QuickSelect order, overlapping first-hit order, lazy non-build, unsorted order, sorted order, and an equal-key group over 32 exposing fixed MSVC behavior.
A focused offline C++ oracle built from pinned sources must record ordered expanded Z paths, intersection pairs, pre/post-merge paths, and final IDs/XY paths for direct crossing, shared vertices, split contour, closed outer/hole cases, multiple IDs, overlapping fallback, and >32 equal-key sorting.
C++ is test-oracle generation only: no production C++ build, subprocess, FFI, or fixture lookup.
The KSR fixture may attest the later automatic-bridge caller branch but must not be wired or unignored.
## Mutation gate
Each independent mutation must make a named focused test fail, with evidence recorded and the mutation reverted:
1. XYZ ordinary equality; 2. removed/reordered endpoint priority; 3. callback at coincident endpoint; 4. zero-based index; 5. unsorted labels; 6. wrong fixup survivor/copy/overwrite Z; 7. junction dedup; 8. increment after moved-slot replacement; 9. grouped split merge; 10. first rather than last rare-repair source; 11. back-before-front recovery; 12. eager AABB; 13. hole-inclusive leaf bbox; 14. missing/wrong-scale epsilon; 15. mathematical midpoint; 16. right-first/lowest-index fallback; 17. wrong hole offset sign; 18. per-contour/output source increment; 19. stable/host seed sort; 20. geometry/index tie-break; 21. stale Z after clear; 22. emptiness before assertion; 23. source before boundary validation.
Record every compiling mutation, named failing test command, failure excerpt, and restored-production rerun in a `/tmp` manifest; survivors block review. This evidence supplements rather than rewrites TDD chronology.
## Validation and acceptance
Required commands:
```text
cargo fmt --all -- --check
cargo nextest run -p ares-core geometry::tests::clipper::z
cargo nextest run -p ares-core geometry::tests::region_expansion::wave_seeds
cargo nextest run --release -p ares-core geometry::tests::clipper::z::release
cargo nextest run --release -p ares-core geometry::tests::region_expansion::wave_seeds::release
cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p ares-core --target wasm32-unknown-unknown
```
Release gates: identical pure-Rust implementation on Linux/macOS/Windows/WASM; no manifest/lock change; no unsafe/FFI/filesystem/native thread/allocator/TBB/platform branch/new geometry dependency; LOC/include audit; existing O27 ordered/error/rerun tests unchanged; pinned oracle exact match. Archive focused debug/release output, full workspace output, strict Clippy, WASM checks/build/export audit, two browser runs, LOC/forbidden/dependency/lifecycle/staging audits, mutation manifest, and both literal review verdicts. After commit/push, `HEAD == origin/main` and a successful Tier-1 run whose `headSha` equals that exact commit are mandatory ship evidence.
Acceptance requires:
1. Exact crate-private signature with no public/lifecycle change.
2. Unchanged 2-D `Point`/`Polygon`/`Polyline`.
3. Only the ARD-0024 indexed kernel.
4. Provenance retained through kernel metadata, never reconstructed from XY.
5. Exact expanded/opened order, signs, labels, and repeated endpoints.
6. Exact endpoint bypass, Z-fill sorting/indexing, and release behavior.
7. Exact PolyTree preorder and point order.
8. Exact four-direction split plus swap-pop/reprocess.
9. All four recovery branches and documented drops tested.
10. Lazy source-compatible AABB inflation, partition, traversal, containment, and overlap order.
11. Optional fixed `(boundary,src)` source-comparator sorting without tie-break.
12. Discovered sorted seeds feed unchanged O27 in a focused geometry test.
13. Exact pinned-Orca ordered IDs/paths.
14. TDD, mutation, platform, lint, format, nextest, dependency, and LOC gates pass.
15. No deferred source-taking propagation, orchestration, or lifecycle behavior enters the diff.
## Rollback
Remove the new wave-seed/Z/AABB production and test modules; remove private Z APIs, record metadata, PolyTree sidecars, and widened sort visibility; restore kernel records to 2-D `Point`.
Retain ARD-0024’s existing kernel, all O27 parameters/types/direct propagation/end types/tests/docs, and the exact O26 lifecycle.
Remove the O28-only ARD-0024 amendment, roadmap entry, and
`option-parity-v4.md` entry while retaining all O27 documentation. No migration,
checkpoint, adapter, manifest, or persisted-state rollback is required.
## Review findings
- **blocker:** `crates/ares-core/src/geometry/clipper/types.rs:22-223` — current edges/intersections/output points carry only XY, making exact provenance impossible.
- **high:** `crates/ares-core/src/geometry/region_expansion.rs:1-20` — `wave_seeds` is absent after O27.
- **high:** `crates/ares-core/src/geometry/clipper/polytree.rs` — production materialization discards per-point metadata.
- **high:** `crates/ares-core/src/geometry/expolygon.rs` — current API lacks exact outer/hole point containment with hole-boundary inversion.
- **medium:** `crates/ares-core/src/geometry/clipper/ordering.rs:66-80` — fixed sorting currently accepts `Copy` values and is not visible to RegionExpansion; use an index permutation.
- **medium:** `OrcaSlicer/src/libslic3r/AABBTreeIndirect.hpp:226-232` — observable centroid arithmetic is `min + max / 2`, not a mathematical midpoint.
- **medium:** stable sorting, geometry tie-breaks, grouped split merging, or eager fallback can alter downstream O27 polygon order.
## Residual risks
- Three/four-label Z-fill behavior is debug-asserted but release-observable; retain first-two behavior and test it through an isolated helper.
- Equal-key parity depends on the ARD-0024 MSVC STL 14.44 compatibility target.
- Overlapping boundaries require a pinned oracle witness for QuickSelect/centroid/traversal order.
- Upstream-compatible `u32` narrowing may truncate impractically large collection IDs; a new overflow error is deferred.
- Full KSR seed counts remain unavailable until later source-cited orchestration.

## Local implementation evidence

The final local implementation uses the approved five production shards and
ten ordinary `mod`-based test shards. All Rust files remain below 400 physical
lines and every new shard is at most 300. Static audits report no staged file,
manifest/lockfile/dependency, public adapter, project lifecycle, forbidden
construct, or deferred-symbol change. The public project boundary remains
`ProjectSlicingIncomplete` after O26.

Focused nextest runs pass 25 Z tests, 39 wave-seed tests, 211 Clipper tests, and
53 RegionExpansion tests. Both final-state workspace runs pass 5,994 tests with
2 skipped. Release filters pass 1 Z and 3 recovery tests. Workspace all-target
check, strict all-feature Clippy, formatting, four wasm32 checks, two optimized
WASM builds, wasm-bindgen export/syntax audit, and two 11-test Playwright runs
all exit zero.

All 23 required semantic mutations and one supplemental strict-shortest-edge
mutation fail their named witness, restore production, and rerun GREEN. Offline
pinned-source debug/`NDEBUG` diagnostics record complete ordered expanded,
intersection, split, recovery-ID, and XY results for inside, crossing, hole,
split, multiple-ID, overlapping-fallback, and release-only shared-vertex
cases. O28 reuses the accepted ARD-0024 MSVC STL 14.44 comparator control flow
unchanged except for geometry-private visibility; the proprietary toolset is
not installed on the Linux host, so the exact pushed-SHA Windows Tier-1 job
remains mandatory platform evidence. Original compiling-RED chronology is not
available and was not reconstructed from post-hoc mutations.

A disposable worktree reproduced the exact O28 tracked and untracked state,
rolled back mechanically to clean predecessor
`f361bb73b558b4e50bfa4fa712afcd63df44ba9f`, and proved that the primary
worktree diff, file list, staging state, and content digests were unchanged.
After the repair/re-review loop, the preliminary independent six-dimensional
reviewer and the default-model OpenCode reviewer both returned literal
`VERDICT: APPROVE`. Final documented-state dual review, commit/push, and the
exact-SHA Tier-1 matrix remain release gates and are not claimed here.
