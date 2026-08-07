# Task 22O.32 — Group expanded polygons by source ExPolygon

## Status and source boundary

Approved implementation specification; the bounded Rust implementation and
initial implementation reviews are locally complete. Exact predecessor O31 is released at
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`; exact-SHA Tier-1 run
`31196271880` passed format, WASM/browser, Linux, Windows, and macOS. The pinned
rewrite target remains OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

Port only `Algorithm/RegionExpansion.cpp:522-534` and its declaration at
`Algorithm/RegionExpansion.hpp:102-108`: `expand_expolygons`. O29 owns the
source/scalar `propagate_waves` composition used by this function; O32 consumes
that result unchanged and groups its polygons into one output slot per source
`ExPolygon`. This is crate-private geometry, not an Ares-owned pipeline.

Deferred: `merge_expansions_into_expolygons` and `expand_merge_expolygons` at
`RegionExpansion.hpp:110-113` / `RegionExpansion.cpp:536-594`;
`LayerRegion`/`PrintObject` external-surface orchestration; Options, lifecycle,
checkpoints, cancellation, persistence, CLI/WASM/browser exports, fill,
toolpath, seam, motion, serialization, G-code, post-processing, and normalized
KSR parity.

## Frozen crate-private API

Add only:

```rust
#[expect(
    clippy::too_many_arguments,
    reason = "the source helper keeps the upstream argument order"
)]
pub(crate) fn expand_expolygons(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    expansion: f32,
    expansion_step: f32,
    max_nr_steps: usize,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, ClipperError>;
```

The first five arguments retain upstream order. Explicit `CoordinateScale`
replaces Orca's mutable global scale and must be forwarded unchanged. Expansion
values are already scaled and must not be scaled again. `Vec<Vec<Polygon>>`
represents upstream `std::vector<Polygons>` without a new wrapper type.

Reexport only through `geometry::region_expansion` and the crate-private
`geometry` facade, with function-pointer type assertions. No public, adapter,
or lifecycle export. The narrow reasoned Clippy expectation is allowed only on
this six-argument source-shaped function.

## Exact composition, cardinality, and ordering

Implement the source operation directly:

```rust
let mut output = vec![Vec::new(); src.len()];
for expansion in propagate_waves_from_sources_with_steps(
    src,
    boundary,
    expansion,
    expansion_step,
    max_nr_steps,
    scale,
)? {
    output[expansion.src_id as usize].push(expansion.polygon);
}
Ok(output)
```

Requirements:

1. allocate exactly `src.len()` output slots before propagation, including when
   no source discovers a wave;
2. call O29's scalar source entry exactly once with all six arguments unchanged
   and in source order;
3. wait for complete successful propagation before moving any polygon into
   output; a discovery or propagation error returns directly and exposes no
   partial result;
4. consume O29 records once in their returned order;
5. index the output only by unchanged `src_id`, discard `boundary_id`, and move
   each polygon into that source slot;
6. preserve the relative order of all polygons assigned to the same source;
7. preserve empty slots for sources with no expansion and return slots in
   original source-index order, even when O29's boundary-first stream order
   differs;
8. trust O29/O28's internal source IDs; add no range check, remap, sort,
   deduplication, canonicalization, cloning, retry, fallback, validation, or
   partial output;
9. add no empty shortcut, alternate parameter overload, duplicated parameter
   construction, seed discovery, direct O27 call, or `RegionExpansionEx`
   conversion.

O29 remains authoritative for builder assertion order, sorted seed discovery,
scaled `f32` behavior, Clipper errors, polygon topology, and propagation order.
O32 neither merges the returned polygons with `src` nor unions polygons within
a slot.

## Behavioral evidence and TDD

Add the frozen signature/reexports with an `Ok(Vec::new())` stub, then write
behavior tests that compile and fail before replacing the stub. Archive the
chronological RED separately from later mutation evidence.

Use O29's already reviewed complete Rust literals and, only if needed, a
disposable pinned-source C++ oracle under `/tmp`. Commit only compact,
human-reviewed, behavior-named polygon literals; never commit oracle source,
output blobs, hashes, or generated code.

The focused shard must freeze complete ordered vectors for:

- empty source and nonempty boundary returning zero slots;
- nonempty sources and empty boundary returning exactly one empty slot per
  source;
- invalid expansion, step, and max-step inputs asserting before empty-input
  completion through O29;
- a single source with all complete expanded contours in its one slot, equal to
  explicit `propagate_waves_from_sources_with_steps` grouping;
- several source indices with leading, interior, and trailing empty slots;
- a boundary-first O29 stream whose records are redistributed into source-index
  slots while preserving per-slot polygon order;
- one source receiving multiple polygons, proving no union or truncation;
- Normal and LargeBed scale witnesses with distinct complete vectors and the
  same explicit scale forwarded to O29;
- direct discovery error and valid discovery followed by propagation error,
  both escaping unchanged and before any result is returned.

Function-pointer assertions prove only type shape. Complete literals and
cross-checks prove cardinality, topology, source-slot order, and per-slot order.
Do not add a production injection seam for impossible IDs or allocation errors.

## Mutations and structural audit

Kill and restore applicable mutations one at a time:

- output length zero, propagation-result length, or nonempty-source count;
- allocation after propagation or early empty return;
- skip, duplicate, or replace the O29 scalar call;
- swap `f32` arguments, substitute max steps, rescale a value, or change scale;
- group by propagation position or `boundary_id` instead of `src_id`;
- sort records or slots, flatten output, omit empty slots, or compact indices;
- overwrite instead of append, reverse per-slot order, retain only one polygon,
  or union polygons in a slot;
- clone/change polygon topology or IDs;
- map/swallow discovery or propagation errors;
- change signature or return shape.

Record compiler rejections and behaviorally equivalent survivors truthfully.
Structurally prove one O29 call, source-sized preallocation, direct source-index
append, no O27/O28/O30 call, no union, and no mutation residue. Mutations are
post-hoc recurrence evidence, not the chronological RED.

## Files, limits, and prohibited changes

Allowed production edits:

- `crates/ares-core/src/geometry/region_expansion/propagate.rs`;
- `crates/ares-core/src/geometry/region_expansion.rs`;
- `crates/ares-core/src/geometry.rs`.

Allowed tests:

- register one ordinary module in
  `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- add
  `crates/ares-core/src/geometry/tests/region_expansion/expand_expolygons.rs`.

Allowed documentation: this spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O31 release-state corrections in
its spec/plan. No ARD change.

Every Rust file remains below 400 physical lines and the new shard at most 300.
No manifest/lock/dependency change, lint `allow`, broad expectation, `unsafe`,
FFI, filesystem/native thread, platform branch, `include!`, `include_bytes!`,
source concatenation, fixture identity/name/hash/layer-count/geometry branch,
reference-G-code access, binary oracle, public hook, fallback, or source
text/hash/line pinning test.

## Verification, review, release, and rollback

Require focused debug/release, complete RegionExpansion, O29 focused, O31
focused, PolyTree, O26 lifecycle, workspace Nextest, all-target check,
warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
export/syntax audit, two Playwright runs, exact allowlist/LOC/visibility and
forbidden-pattern audits, and disposable rollback to exact O31.

Independent six-dimensional and default-model OpenCode reviewers must both
return literal `VERDICT: APPROVE`. Any requested repair is followed by affected
and complete exact-candidate verification, refreshed evidence, and both reviews
against the repaired diff. Commit and push only after approval. O32 is released
only when Tier-1 `headSha` equals the exact pushed documentation SHA and all
five jobs pass.

Chronological RED compiled and failed 5/5 against the empty stub. Focused
debug/release pass 5/5, complete RegionExpansion passes 74/74, thirteen runtime
mutations are killed, two type-shape mutations are compiler-rejected, and two
behavioral equivalence classes are fixed structurally. Final LOC are 266, 81,
177, 8, and 253 across the approved Rust files. Initial independent and
default-model OpenCode implementation reviews both returned literal
`VERDICT: APPROVE`. After tuple-packing the test-only explicit grouping helper
to satisfy the repository's five-argument Clippy threshold, the exact candidate
passes workspace Nextest 6,015/6,015 with 2 skipped, all-target check,
warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
wasm-bindgen export and JavaScript syntax audits, static audits, and disposable
exact-O31 rollback. The local browser launch stops before test code because the
host lacks `libglib-2.0.so.0`; exact-SHA Tier-1 must install browser dependencies
and pass both Playwright runs. Final independent six-dimensional and
default-model OpenCode reviews both returned literal `VERDICT: APPROVE`; only
commit/push and exact-SHA Tier-1 release gates remain pending.

Public slicing must still consume O26 and return `ProjectSlicingIncomplete`;
the golden KSR test remains unchanged and incomplete. Rollback removes only the
O32 function, private reexports/assertions, test shard/registration, O32 docs,
and O31 release-state corrections, retaining released O27-O31. The next source
boundary is `merge_expansions_into_expolygons` at
`RegionExpansion.cpp:536-587` / `RegionExpansion.hpp:110-111`.
