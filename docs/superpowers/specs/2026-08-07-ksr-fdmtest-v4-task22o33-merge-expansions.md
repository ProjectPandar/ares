# Task 22O.33 — Merge region expansions into source ExPolygons

## Status and source boundary

Locally implemented and fully verified. The complete documented-candidate and
rollback gates pass, and repaired final independent/default-model OpenCode
rereviews approve; commit/push and exact-SHA release gates remain. Exact predecessor O32 is released as
commits `2e7168f`/`699f02b`; exact-SHA Tier-1 run `31213611275` passed format,
WASM/browser twice, Linux, Windows, and macOS at
`699f02b2bbc3d797f53edf5f8c65dd2614830ecb`. The pinned rewrite target remains
OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only `Algorithm/RegionExpansion.cpp:536-587` and its declaration at
`Algorithm/RegionExpansion.hpp:110-111`:
`merge_expansions_into_expolygons`. The function consumes source ExPolygons and
O27/O29 `RegionExpansion` records, groups records by source ID, joins each
expanded source with its original contour and holes through Orca's safety
offset, and retains the source-connected component. This is a source-cited,
crate-private geometry prerequisite, not an Ares-owned pipeline.

The implementation also consumes already-cited upstream primitives rather than
inventing alternatives:

- `ClipperUtils.hpp:17-23,362-365` and `ClipperUtils.cpp:260-316,366-414` for
  `ClipperSafetyOffset`, default miter configuration, per-path raw offset, and
  `union_safety_offset_ex(Polygons)`;
- `RegionExpansion.cpp:159-193` and
  `AABBTreeIndirect.hpp:223-232` for AABB sampling, `SCALED_EPSILON`, source
  centroid arithmetic, and first matching ExPolygon containment;
- `ExPolygon.cpp:107-116` for border-inclusive contour and hole-boundary
  containment.

Deferred: `expand_merge_expolygons` at `RegionExpansion.hpp:113` /
`RegionExpansion.cpp:589-594`; `LayerRegion` and `PrintObject` external-surface
orchestration; Options, lifecycle, checkpoints, cancellation, persistence,
CLI/WASM/browser exports, fill, toolpath, seam, motion, serialization, G-code,
post-processing, and normalized KSR parity.

## Frozen crate-private API

Add only:

```rust
pub(crate) fn merge_expansions_into_expolygons(
    src: Vec<ExPolygon>,
    expanded: Vec<RegionExpansion>,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError>;
```

The two by-value vectors model upstream rvalue-reference ownership. Explicit
`CoordinateScale` replaces Orca's mutable global scale and is used only by the
existing AABB sampler's `SCALED_EPSILON` behavior. It must not rescale polygons,
the safety offset, or any input value. The return becomes `Result` only because
Ares's safe Clipper kernel reports coordinate-range failures.

Reexport only through `geometry::region_expansion` and the crate-private
`geometry` facade, with function-pointer type assertions. No public, adapter,
or lifecycle export. Do not add an overload, parameter object, generic helper,
or public wrapper.

## Exact grouping, merge, selection, and ordering

Implement the source operation in a dedicated `region_expansion/merge.rs`
module:

1. build a `Vec<usize>` permutation over `expanded`, sort that `Copy` index
   vector by referenced `src_id` only through ARD-0024's existing fixed MSVC
   STL 14.44 comparator control flow, then move every non-`Copy`
   `RegionExpansion` exactly once through temporary `Option` slots into the
   resulting order; do not use the host stable/unstable sort, clone records,
   add a boundary-ID or geometry tie-breaker, or sort `src` or final output;
2. reserve final output for `src.len()` and walk source ownership in increasing
   source-index order;
3. before each expanded source ID, move every missing leading or interior
   source ExPolygon directly to output unchanged;
4. clear the polygon accumulator for each source-ID group, then move every
   grouped expansion polygon into it; ignore `boundary_id` completely;
5. move exactly one matching source ExPolygon, first capturing its contour's
   first point as the sample; an empty contour remains a trusted internal
   invariant and panics rather than becoming a `ClipperError`;
6. append the moved source contour and then its holes in their existing order,
   after every expansion polygon;
7. call the raw-polygon `union_safety_offset_ex` equivalent exactly once for
   that group: independently offset each oriented path by fixed `10.0_f32`
   coordinate units using `JoinType::Miter`, miter limit `3.0`, shortest-edge
   factor `0.005`, then union with `FillRule::NonZero` into ExPolygons;
8. if the merge returns zero ExPolygons, emit none for that source; if it
   returns one, move it to output; if it returns more than one, build the
   already-ported O28 AABB with the same explicit scale and move only the first
   ExPolygon whose full containment includes the original sample;
9. in the multi-result branch, preserve upstream `assert(id != -1)` as a
   `debug_assert!`; in release, a missing match emits nothing rather than
   choosing index zero, the largest component, or the original source;
10. after all groups, move every trailing untouched source ExPolygon directly
    to output.

The source-ID sort must mirror O28's ownership-safe permutation pattern:

```rust
let mut order = (0..expanded.len()).collect::<Vec<_>>();
fixed_msvc_sort_by(&mut order, |left, right| {
    expanded[*left].src_id < expanded[*right].src_id
});
let mut source = expanded.into_iter().map(Some).collect::<Vec<_>>();
let expanded = order.into_iter().map(|index| {
    source[index].take().expect("sort permutation is unique")
});
```

The iterator may be collected or consumed directly by the grouping loop, but
its fixed comparator permutation and move-once behavior may not change. The
temporary index vector and `Option<RegionExpansion>` slots are approved
ownership machinery, not defensive copies or validation. The safety-offset
helper must reuse ARD-0024's existing `offset_paths_tree` path. It must not pre-union the accumulator, call the ExPolygon overload,
offset only expansions, use a plain union, or perform a two-sided closing.
Expose the existing O28 `BoundaryAabb` sampling path narrowly for production
reuse; do not create a second AABB implementation or linear-scan fallback.

Output is source-index ordered. Unexpanded sources keep exact topology and
relative order. Each valid expanded source contributes at most one ExPolygon.
`expanded` record order among different source IDs is discarded by the source
sort; equal-key ordering follows the fixed MSVC permutation even when the final
union makes that ordering behaviorally equivalent. Other than the approved
index/`Option` move machinery, no clone, deduplication, canonicalization,
reorientation, remap, retry, or alternate component selection is allowed.

## Trusted invariants, errors, and scale

Valid absent IDs mean no expansion and preserve their source unchanged.
Malformed internal `src_id >= src.len()`, expansions with empty `src`, and
empty expanded-source contours remain trusted-invariant panics/invalid internal
states. Do not add range validation, error variants, skipped records, remapping,
or a production injection seam. The `u32` source ID converts directly to
`usize`; `boundary_id` is never consumed.

Only Clipper failures return `Err`. Groups are attempted in sorted source-ID
order, so the first safety-offset failure escapes unchanged and no partial
output is observable. Unexpanded sources are moved without Clipper validation.
No error is swallowed, mapped, retried, or replaced with original geometry.

The fixed safety offset is ten stored coordinate units for both
`CoordinateScale::Normal` and `CoordinateScale::LargeBed`; it is never divided
or multiplied by `scale.factor()`. The scale reaches only AABB leaf inflation:
100 stored units for Normal and 10 for LargeBed, preserving the upstream
`SCALED_EPSILON` replacement and existing O28 comparator/traversal behavior.
Full containment remains outer interior/boundary accepted, hole interior
rejected, and hole boundary accepted.

## Behavioral evidence and TDD

Add the frozen signature/reexports with an `Ok(Vec::new())` stub, then add
behavior tests that compile and fail before replacing the stub. Archive this
chronological RED separately from later mutation evidence.

Use compact, human-reviewed complete literals generated from a disposable
pinned-source C++ oracle under `/tmp`. Validate oracle vectors in debug and
`NDEBUG`; do not commit oracle source, raw output, serialized blobs, hashes, or
generated code. Comparator-sensitive cases must avoid claiming equal-key
ordering that upstream `std::sort` does not guarantee.

The focused ordinary test modules must freeze complete ordered ExPolygon point
vectors for:

- empty `expanded` preserving all nonempty sources exactly, including contour,
  holes, source order, and moved point-buffer identity;
- unsorted records for several source IDs, with leading, interior, and trailing
  unexpanded sources moved unchanged and output still in source-index order;
- multiple records for one source, mixed `boundary_id` values, accumulator
  clearing, and one safety-offset merge;
- source contour then hole ordering, fixed 10-unit miter expansion, hole
  contraction, NonZero topology, and complete output point order;
- a disconnected expansion producing multiple merged ExPolygons, proving the
  AABB/sample branch keeps only the source-connected component;
- zero-, one-, and multi-result branches where source-supported vectors can be
  constructed without invalid hardcoding;
- Normal and LargeBed calls using the same fixed safety offset and explicit
  scale forwarded only to the existing AABB sampler;
- an empty expanded-source contour panic and malformed source-ID panic as
  internal invariant witnesses;
- direct safety-offset coordinate-range failure, plus two source groups proving
  sorted source-ID error precedence and no exposed partial output;
- untouched-source ownership moves and merged-source output allocation without
  cloning predecessor point buffers.

Function-pointer assertions prove only type shape. Complete literals, pinned
oracle vectors, pointer witnesses, and explicit helper cross-checks prove
ordering, topology, safety offset, selection, errors, and ownership.

## Mutations and structural audit

Kill and restore applicable mutations one at a time:

- skip sorting, use the host stable/unstable sort, reverse source order, add an
  equal-key tie-breaker, or sort/group by `boundary_id`;
- omit leading/interior/trailing unchanged sources, clone them, or reorder final
  output;
- fail to clear the accumulator, omit/duplicate/reverse expansions, overwrite
  instead of append, or retain only the first expansion;
- append source before expansions, omit contour or holes, reverse hole order,
  or wrap raw paths as independent ExPolygons;
- replace safety offset with plain union, zero/negative/rescaled delta, a second
  offset, wrong join/fill/miter/shortest-edge behavior, or ExPolygon overload;
- change zero/one/multi-result branches, retain all components, pick first or
  largest, sample an expansion point, use contour-only containment, reject hole
  boundaries, linear scan, or wrong/missing scale;
- validate/remap IDs, consume `boundary_id`, map/swallow Clipper errors, expose
  partial output, retry, or keep original geometry after failure;
- sort records directly despite their non-`Copy` payload, duplicate/omit an
  index, clone a record instead of moving it from its unique `Option` slot, or
  otherwise change ownership, visibility, signature, or return shape.

Record compiler rejections and behaviorally equivalent survivors truthfully.
Structurally prove one fixed-MSVC source-ID index-permutation sort, a unique
move from every referenced temporary slot without record/polygon cloning, one
safety-offset call per active source group, source contour/holes appended after
expansions, reuse of the O28
AABB sampler, fixed unscaled offset constants, direct error escape, and no
mutation residue. Mutations are post-hoc recurrence evidence, not chronological
RED.

## Files, limits, and prohibited changes

Allowed production edits:

- add `crates/ares-core/src/geometry/region_expansion/merge.rs`;
- `crates/ares-core/src/geometry/region_expansion.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/clipper/boolean_ex.rs` and
  `crates/ares-core/src/geometry/clipper.rs` only for a source-shaped
  `union_safety_offset_ex(&[Polygon])` helper/reexport;
- `crates/ares-core/src/geometry/region_expansion/wave_seeds.rs` and
  `wave_seeds/aabb.rs` only to expose the existing production sampler with no
  algorithm change.

Allowed tests:

- register one ordinary `merge_expansions` module in
  `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- add a bounded
  `crates/ares-core/src/geometry/tests/region_expansion/merge_expansions.rs`
  root and ordinary shards under `merge_expansions/` as needed.

Allowed documentation: this spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O32 spec/plan release-state
corrections. No ARD change.

Every Rust file remains below 400 physical lines; each new test shard is at
most 300 lines. No manifest/lock/dependency change, lint `allow`, broad
expectation, `unsafe`, FFI, filesystem/native thread, platform branch,
`include!`, `include_bytes!`, source concatenation, fixture identity/name/hash/
layer-count/geometry branch, reference-G-code access, binary oracle, public
hook, legacy fallback, or source text/hash/line pinning test.

## Local implementation and evidence

The compiling chronological RED ran 11 tests against the empty stub: ten
meaningful failures and one behaviorally equivalent pass. The literal helper
uses the existing fixed-MSVC comparator to sort source-ID indices, moves each
non-`Copy` record once through unique `Option` slots, moves untouched sources
unchanged, accumulates expansion polygons then source contour and holes, and
reuses a fixed unscaled 10/Miter/3 safety-offset union plus the O28 AABB sampler.

Initial review found two test-evidence defects, not production defects: the
zero-result arm lacked a direct witness, and the temporary C++ oracle reused a
moved output buffer. A one-point source/expansion now proves the true zero
result; a dedicated panic mutation is killed 0/1. The corrected oracle creates
a fresh output buffer per input and produces byte-identical debug/`NDEBUG`
outputs, including the acute-miter contour `(50,68),(-23,-10),(123,-10)`.
The repaired exact candidate passes focused debug/release 13/13 and complete
RegionExpansion 87/87. Thirteen runtime mutations are killed, one signature
mutation is compiler-rejected, and accumulator-clear, scale, equal-key host-sort,
and union-order equivalences are disclosed as structural survivors. Final LOC
are 184/210/169/89/65/295/266/9/60/102/119/75 across the approved Rust files.
Repaired independent and default-model OpenCode initial implementation reviews
both returned literal `VERDICT: APPROVE`.

The first full verification exposed only a test-constant Clippy
`type_complexity` finding; splitting the function-pointer type into two narrow
test aliases changed no production behavior. After that repair, the complete
exact documented candidate was rerun: focused debug/release 13/13, AABB 8/8,
O32 5/5, RegionExpansion 87/87, PolyTree 6/6, offset 58/58, O26 lifecycle 3/3,
and workspace Nextest 6,028/6,028 with 2 skipped all pass. All-target check,
warning-denying all-feature Clippy, rustfmt, four WASM checks, two optimized
WASM builds, exact export audit, JavaScript syntax, and diff checks pass. Both
local Playwright attempts stop before test code because Chromium cannot load
`libglib-2.0.so.0`; the workflow installs browser dependencies and both exact
pushed-SHA CI browser runs remain mandatory.

Disposable exact-O32 rollback proves the copied candidate is byte-identical,
removes only O33 plus O32 release-state corrections, restores a clean exact-O32
tree, passes RegionExpansion 74/74, PolyTree 6/6, and lifecycle 3/3, and leaves
the primary candidate byte-identical and unstaged. After exact oracle-input and
stale-status repairs, the complete suite was rerun and final independent and
default-model OpenCode rereviews returned literal `VERDICT: APPROVE`.
Commit/push and exact-SHA Tier-1 remain release gates. O33 is not released.

## Verification, review, release, and rollback

Require focused debug/release, complete RegionExpansion, direct offset/PolyTree
regressions, O28 AABB, O32 focused, O26 lifecycle, workspace Nextest, all-target
check, warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
export/syntax audit, two Playwright runs, exact allowlist/LOC/visibility and
forbidden-pattern audits, and disposable rollback to exact O32.

Independent six-dimensional and default-model OpenCode reviewers must both
return literal `VERDICT: APPROVE`. Any requested repair is followed by affected
and complete exact-candidate verification, refreshed evidence, and both reviews
against the repaired diff. Commit and push only after approval. O33 is released
only when Tier-1 `headSha` equals the exact pushed documentation SHA and all
five jobs pass.

Public slicing must still consume O26 and return `ProjectSlicingIncomplete`;
the golden KSR test remains unchanged and incomplete. Rollback removes only the
O33 helper, narrow shared-helper exports, test modules, O33 docs, and O32
release-state corrections, retaining released O27-O32. The next bounded source
boundary is `expand_merge_expolygons` at `RegionExpansion.cpp:589-594` /
`RegionExpansion.hpp:113`; external-surface orchestration remains deferred.
