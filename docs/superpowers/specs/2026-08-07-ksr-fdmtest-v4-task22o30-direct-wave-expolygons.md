# Task 22O.30 — Direct RegionExpansionEx wave output

## Status and source boundary

Approved implementation specification. Independent and default-model OpenCode
spec and plan reviewers returned literal `VERDICT: APPROVE`. Exact predecessor
O29 is released at `118f6a72b33926efe41ced1c931f9a51b26b2945`;
exact-SHA Tier-1 run
`31168584784` passed format, WASM/browser, Linux, Windows, and macOS. The rewrite
target remains OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone ports only:

- `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.hpp:85-92`, the
  `RegionExpansionEx` result record and supplied-seed declaration; and
- `OrcaSlicer/src/libslic3r/Algorithm/RegionExpansion.cpp:480-503`, the direct
  supplied-seed `propagate_waves_ex` implementation.

The multi-polygon branch also follows the existing upstream default
`union_ex(const Polygons &, pftNonZero)` declared at `ClipperUtils.hpp:548`.
O27 owns supplied-seed propagation; O30 consumes it unchanged and reuses the
single ARD-0024 indexed `union_ex` implementation. O30 is a crate-private
geometry prerequisite, not an Ares-owned pipeline or public slicing stage.

Deferred: the source/scalar `propagate_waves_ex` overload at
`RegionExpansion.cpp:506-520`; `expand_expolygons` at lines 522-534; merge
helpers at lines 536-594; all `LayerRegion`/`PrintObject` external-surface
orchestration; Options; lifecycle/checkpoints/cancellation/persistence; CLI,
WASM, and browser exports; fill, toolpath, seam, motion, serialization, G-code,
and post-processing behavior; normalized KSR G-code parity.

## Implemented status and bounded evidence

O30 is implemented locally in the reviewed six-file Rust allowlist. The direct
entry calls unchanged O27 once, then applies the debug-only nondecreasing
`(boundary, src)` assertion, groups only adjacent expanded records by both IDs,
wraps singleton contours directly, and sends multi-polygon groups to the
existing `union_ex` with `FillRule::NonZero`. `RegionExpansionEx` and the entry
remain crate-private and absent from lifecycle and adapter surfaces.

The disposable pinned-source oracle freezes complete singleton, natural
one-seed/two-expanded-contour hole, multi-island, adjacent-ID,
boundary-before-source comparator-conflict, and release-unsorted vectors. It
also shows that Positive and NonZero are equivalent for the tested valid
positive-clipped hole and that singleton-through-union is byte-identical for
open one-step, open staged, and closed singleton candidates. Those two
behaviorally equivalent mutations are recorded as honest survivors and their
source branches are instead fixed by structural review. Sixteen other runtime
mutations are killed, one result-field type mutation is compiler-rejected, and
the final restored focused shard is green.

The real chronological RED ran six tests against the compiling empty stub: five
nonempty/error/assertion expectations failed and the legitimate zero-output
case passed. The final shard passes 6/6 in debug and 6/6 in release; complete
RegionExpansion passes 64/64, existing PolyTree passes 6/6, O26 lifecycle
passes 3/3, and workspace Nextest passes 6,005/6,005 with 2 skipped. Native
all-target check, warning-denying Clippy, rustfmt, four WASM checks, two
optimized WASM builds, export/syntax audits, and two 11/11 Playwright runs are
green. Final physical LOC are 74 (`types.rs`), 218 (`propagate.rs`), 62
(`region_expansion.rs`), 156 (`geometry.rs`), 6 (test root), and 263 (new
shard). Exact allowlist/LOC/visibility/forbidden-pattern static audit and the
disposable exact-predecessor rollback are green. Final independent
six-dimensional and default-model OpenCode implementation reviews both return
literal `VERDICT: APPROVE`. O30 was released as implementation commit `0a19939` and documentation commit
`6ccb145`; exact-SHA Tier-1 run `31184069746` passed format, WASM/browser,
Linux, Windows, and macOS at
`6ccb145dbb1867e5724538fb071795a7fd4179f0`.

Public slicing still consumes O26 and returns `ProjectSlicingIncomplete`. O30
adds no Option, checkpoint, lifecycle state, KSR golden change, G-code byte, or
ARD change.

## Crate-private API and representation

Add the exact source-shaped result record beside the existing O27 record:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionExpansionEx {
    pub(crate) expolygon: ExPolygon,
    pub(crate) src_id: u32,
    pub(crate) boundary_id: u32,
}
```

Add only this direct entry:

```rust
pub(crate) fn propagate_waves_ex(
    seeds: &[WaveSeed],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
) -> Result<Vec<RegionExpansionEx>, ClipperError>;
```

Reexport both only through `geometry::region_expansion` and the crate-private
`geometry` facade. Add function-pointer assertions there for arity, argument,
and return shape. Do not export through `lib.rs`, another crate, CLI, WASM, or
browser JavaScript. This direct entry performs neither seed discovery nor
parameter construction and therefore takes no `CoordinateScale`.

## Exact operation and assertion order

The function first completes the unchanged O27 call:

```rust
let expanded = propagate_waves(seeds, boundary, params)?;
```

Only after successful propagation, apply the C++ `assert(std::is_sorted(...))`
as a debug-only assertion over the original `seeds`. Ordering is lexicographic
nondecreasing by `(boundary, src)`. Equal adjacent keys are valid and must not
panic. The check must not inspect paths, expanded output, or IDs in source-first
order.

This sequence is observable:

- a propagation `ClipperError` escapes before an unsorted-input debug panic;
- successful unsorted input panics only in debug builds;
- release builds do not validate, sort, regroup, reject, repair, or fall back;
- empty input still calls O27 before the assertion and returns empty.

The trusted supplied-seed contract remains unchanged. Do not add boundary-index,
path-shape, parameter, range, or sortedness validation beyond the debug
assertion. O27 assertion/panic and error behavior remains authoritative.

## Adjacent expanded-result grouping

Consume the complete ordered `Vec<RegionExpansion>` returned by O27 in one
forward pass. Group adjacent records only while both are unchanged:

```text
next.boundary_id == first.boundary_id && next.src_id == first.src_id
```

Requirements:

1. group the expanded records, not the seeds;
2. compare boundary ID first and source ID second, while requiring equality of
   both for group membership;
3. never sort, globally regroup, canonicalize, deduplicate, filter, or merge
   nonadjacent records;
4. branch on the number of expanded polygons in the current group, not seed
   count or output count;
5. retain the first expanded record's unchanged `src_id` and `boundary_id` for
   every output from that group;
6. preserve outer group order and every existing union output order.

A group cannot be empty because it starts from one expanded record. A seed
group may produce no expanded records; O30 emits no placeholder for it.

## Singleton and multi-polygon branches

For exactly one expanded polygon, move it directly into
`ExPolygon::new(polygon, Vec::new())` and emit one `RegionExpansionEx`. Do not
invoke any Clipper operation. This bypass is semantic: unioning a singleton can
change point start/order, topology normalization, and failure behavior.

For two or more expanded polygons, call the existing indexed kernel exactly as:

```rust
let expolygons = union_ex(&polygons, FillRule::NonZero)?;
```

Then emit every returned `ExPolygon`, in returned order, with unchanged group
IDs. Do not use `Positive`, `EvenOdd`, or `Negative`; do not use safety offsets,
`union_expolygons`, a new union engine, a second clipping pass in O30, hole
flattening, largest-contour retention, area sorting, or point canonicalization.
An empty union result emits nothing; multiple islands emit multiple records.

The upstream `reserve_more_power_of_2` call affects allocation capacity only,
not output bytes or ordering. Rust may use normal `Vec` reserve/extend behavior;
it must still move owned polygons and expolygons rather than defensively clone
them.

## Error behavior

The Rust boundary returns the existing `ClipperError` because both reused
operations are fallible. Propagation errors and reachable multi-union errors
escape directly with `?`, without mapping, `SliceError`, retry, fallback,
partial output, or catch-and-continue behavior. The debug sorted assertion runs
only after propagation success and before grouping/union.

Do not add a production injection seam merely to force a post-propagation union
error. If no natural end-to-end union-error witness exists, evidence is the
literal `?`, source/diff review, mutation where constructible, and the existing
focused `union_ex` range-error regression. Do not overclaim that a direct
`union_ex` test proves an otherwise unreachable O30 error path.

## Files, ownership, and LOC

Allowed production edits:

- `crates/ares-core/src/geometry/region_expansion/types.rs`;
- `crates/ares-core/src/geometry/region_expansion/propagate.rs`;
- `crates/ares-core/src/geometry/region_expansion.rs`;
- `crates/ares-core/src/geometry.rs`.

Allowed tests:

- register one ordinary module from
  `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- add
  `crates/ares-core/src/geometry/tests/region_expansion/expolygon_output.rs`.

Allowed documentation: this spec, its reviewed plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and truthful O29 release-state
corrections in its spec/plan. ARD-0024 does not change because O30 reuses its
only indexed Clipper kernel and introduces no architecture decision.

One implementation worker is the sole writer during implementation and repair;
all reviewers are read-only. Stop and amend/re-review the spec if another
production or test file is needed.

Every Rust source file must remain below 400 physical lines; the new test shard
must remain at most 300 physical lines. Use ordinary `mod`, never `include!`,
`include_bytes!`, source concatenation, or generated Rust splitting. Add no
manifest/lockfile change, dependency, lint `allow`, broad lint expectation,
`unsafe`, FFI, filesystem/native-thread behavior, platform branch, fixture
identity/name/hash/layer-count/geometry-identity branch, reference-G-code
access, binary oracle payload, fallback, or public test hook.

## TDD and oracle evidence

Create the record, frozen signature/reexports, and an `Ok(Vec::new())` function
stub so tests compile. Before the production body, add behavior assertions that
fail against the empty stub; unresolved imports or compilation failure do not
count as RED. Archive the real chronological RED under `/tmp`; later mutations
must be reported separately and never relabeled as original RED.

Use a disposable C++ oracle under `/tmp`, built from the pinned
Orca/Clipper sources, to inspect representative exact result vectors. Do not
commit C++, oracle output, serialized blobs, hashes, or generated source.
Human-reviewed, behavior-named Rust literals containing complete IDs, contour
points, and holes may be committed. The committed test must never invoke Orca,
filesystem fixtures, or reference G-code.

The focused shard must cover, with complete ordered snapshots rather than only
counts, area, or bounds:

1. empty seeds and boundaries return empty;
2. a singleton expanded polygon is wrapped directly with zero holes and exact
   O27 contour identity/order;
3. equal-key seeds are accepted and a multi-polygon group is passed through
   NonZero union, including exact output order;
4. one multi group that unions to one `ExPolygon`, and one that emits multiple
   islands; retain exact IDs on every result;
5. a natural hole-producing witness if the propagated geometry can construct
   one; otherwise run and cite the existing complete `union_ex` hole/topology
   regression without manufacturing an O30 seam or claiming it as direct O30
   execution;
6. adjacent `(boundary, src)` groups remain separate and ordered, including an
   equal-key group and both source and boundary transitions;
7. a valid seed whose O27 clipping produces no polygon emits no placeholder;
8. in debug, successful unsorted seeds panic after propagation while equal keys
   do not; in release, the same successful unsorted witness follows O27's
   adjacent-group behavior without sorting;
9. an unsorted witness whose first propagation group returns
   `ClipperError::CoordinateOutOfRange` returns that error before the debug
   assertion; a sorted propagation error is forwarded unchanged.

Run the shard in both debug and release. If debug and release use conditional
assertions in one test source, archive both executions explicitly. No source
text/hash/line-number pinning assertion is permitted.

## Mutation and structural audit

Kill and restore, one at a time where behaviorally distinguishable:

- skipping or swallowing the O27 call/error;
- moving the sorted assertion before propagation;
- removing it or making it active in release;
- rejecting equal keys or comparing `(src, boundary)`;
- sorting/regrouping before conversion;
- grouping by only source or only boundary;
- branching on seed count instead of expanded group size;
- sending singleton output through union;
- wrapping multi polygons separately instead of unioning;
- changing `NonZero` fill;
- dropping holes, retaining only one island, reversing union output, or losing
  IDs;
- mapping or swallowing a reachable union error, if a natural witness exists;
- changing the result record/function signature shape.

A behaviorally equivalent ownership/iterator rewrite is structural evidence,
not a killed mutation. Record survivors and limitations truthfully. Restore
exact GREEN after every mutation and verify no mutation residue remains.

## Verification and release gates

Required local gates:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expolygon_output
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Also require:

- release-mode focused O30 execution;
- all four existing `wasm32-unknown-unknown` core/adapter checks, with and
  without the existing browser-oracle feature;
- default and feature optimized WASM builds;
- wasm-bindgen export audit proving no O30/default export drift and unchanged
  browser-oracle-only exports;
- JavaScript syntax audit and two complete Playwright executions;
- LOC, visibility/export, dependency/manifest/lockfile, lifecycle/public-state,
  forbidden-pattern, lint, staging/allowlist, and diff audits;
- disposable rollback rehearsal that removes only O30 and release-state docs,
  leaving exact released O29 behavior and passing its 58 RegionExpansion tests
  plus O26 lifecycle;
- independent six-dimensional and default-model OpenCode reviews returning
  literal `VERDICT: APPROVE`, followed by repair and renewed review as needed;
- Conventional Commits, push, and Tier-1 success whose `headSha` is the exact
  pushed O30 documentation SHA.

Public slicing must continue to use O26 and return
`ProjectSlicingIncomplete`. The golden KSR test remains unchanged and does not
pass in O30. Documentation must never call O30 full external-surface, slicing,
or G-code parity.

## Acceptance criteria

O30 is accepted only when:

1. the exact crate-private record and direct function signatures are frozen;
2. O27 completes before the debug nondecreasing `(boundary, src)` assertion;
3. adjacent expanded records are grouped by both IDs without reordering;
4. singleton output bypasses union and multi output uses the existing NonZero
   `union_ex`, preserving complete topology, IDs, and order;
5. direct errors, debug/release behavior, zero-output groups, singleton,
   one-result union, multi-result union, and ID transitions have complete
   focused evidence plus pinned-oracle inspection;
6. no O27/O28/O29, Clipper, lifecycle, public/export, Option, or G-code behavior
   changes;
7. all required native, release, mutation, WASM/browser, static, rollback, and
   dual-review gates pass;
8. exact-pushed-SHA Tier-1 passes before release is claimed.

## Rollback and next boundary

Mechanical rollback removes only `RegionExpansionEx`, direct
`propagate_waves_ex`, crate-private reexports/signature assertions, the O30 test
shard/registration, O30 docs, and release-state documentation changes. It
retains released O27 propagation, O28 seed discovery, O29 source composition,
ARD-0024, and the O26 lifecycle.

The next candidate boundary after O30 is the source/scalar
`propagate_waves_ex` overload at `RegionExpansion.cpp:506-520`. It requires
parameter construction, sorted seed discovery, and explicit retained
`CoordinateScale`; it remains out of O30 until separately specified and
reviewed.
