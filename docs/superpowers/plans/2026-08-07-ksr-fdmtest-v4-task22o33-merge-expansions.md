# Task 22O.33 Implementation Plan

## Execution state

Tasks 1-7 are complete. The chronological stub RED compiled and ran 11 tests:
10 meaningful failures and one behaviorally equivalent pass. After independent
review exposed a missing zero-result witness and an oracle-harness output-buffer
bug, the test suite gained a true one-point zero-result vector, the oracle was
rebuilt from one corrected source with byte-identical debug/`NDEBUG` output,
and a dedicated zero-arm mutation was killed. The repaired exact candidate
passes focused debug/release 13/13 and complete RegionExpansion 87/87.
Thirteen runtime mutations are killed, one signature mutation is compiler
rejected, and structural/equivalent survivors remain explicitly disclosed.
All Rust files remain below 400 lines and new shards remain below 300. Repaired
independent and default-model OpenCode initial implementation reviews both
returned literal `VERDICT: APPROVE`. The first complete run then found a
Clippy-only complex test-constant type; two narrow aliases repaired it without
production change, affected checks passed, and the complete suite was rerun.
Focused debug/release 13/13, AABB 8/8, O32 5/5, RegionExpansion 87/87,
PolyTree 6/6, offset 58/58, lifecycle 3/3, workspace 6,028/6,028 with 2 skipped,
check, warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
export and JavaScript syntax audits all pass. Two local browser attempts fail
before test code only because `libglib-2.0.so.0` is unavailable; exact-SHA CI
must pass both runs. Disposable exact-O32 rollback proves candidate and primary
byte identity, restores a clean baseline, and passes RegionExpansion 74/74,
PolyTree 6/6, and lifecycle 3/3. Final review found exact oracle-input and stale
status defects; both were repaired, the entire suite and rollback were
refreshed, and final independent/default-model OpenCode rereviews both returned
literal `VERDICT: APPROVE`. Tasks 1-10 are complete; Task 11 remains pending.

## Goal, reviewed spec, and baseline

Port only pinned OrcaSlicer
`Algorithm/RegionExpansion.cpp:536-587` and
`RegionExpansion.hpp:110-111` as the crate-private
`merge_expansions_into_expolygons` helper. The approved design is
`docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o33-merge-expansions.md`.
Its revised independent and default-model OpenCode spec reviews both returned
literal `VERDICT: APPROVE` after replacing an impossible direct non-`Copy` sort
with O28's fixed-MSVC index permutation and unique `Option`-slot moves.

Exact predecessor is released O32 at
`699f02b2bbc3d797f53edf5f8c65dd2614830ecb`; implementation/documentation
commits are `2e7168f`/`699f02b`, and exact-SHA Tier-1 run `31213611275` passed
all five jobs, including both browser runs. Pinned Orca is
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Baseline RegionExpansion is
74/74, PolyTree 6/6, and O26 lifecycle 3/3. Public slicing remains incomplete.

One worker is the sole Rust writer. Reviewers never edit. The parent inspects
all diffs and runs authoritative RED/GREEN, oracle, mutation, full
verification, review-repair, commit, push, and exact-SHA CI gates.

Allowed production files:

- new `crates/ares-core/src/geometry/region_expansion/merge.rs`;
- `crates/ares-core/src/geometry/region_expansion.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/clipper/boolean_ex.rs`;
- `crates/ares-core/src/geometry/clipper.rs`;
- `crates/ares-core/src/geometry/region_expansion/wave_seeds.rs`;
- `crates/ares-core/src/geometry/region_expansion/wave_seeds/aabb.rs`.

Allowed tests:

- `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- new `crates/ares-core/src/geometry/tests/region_expansion/merge_expansions.rs`;
- new ordinary shards under
  `crates/ares-core/src/geometry/tests/region_expansion/merge_expansions/`.

Allowed docs are the O33 spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O32 spec/plan release-state
corrections. Stop and amend/re-review the spec and plan before touching another
production, test, or documentation file.

## Validation contract

Acceptance requires direct observation that:

- the API is crate-private, consumes both vectors, accepts explicit scale, and
  returns direct `ClipperError`;
- the fixed MSVC STL 14.44 source-ID permutation is reproduced with `Copy`
  indices while every non-`Copy` record/polygon is moved exactly once;
- untouched sources remain in source-index order with exact topology and point
  buffers, including leading/interior/trailing gaps;
- each active source accumulator contains all its expansion polygons, then the
  source contour, then holes, and is cleared between groups;
- each group performs exactly one fixed, unscaled 10-unit Miter/3.0/0.005
  raw-path safety offset and NonZero PolyTree union;
- zero, one, and multiple merged-output branches match source, with the latter
  reusing O28 AABB scale/partition/first-hit/full-containment behavior;
- malformed IDs/empty contours remain trusted panics, while the first sorted
  Clipper failure escapes unchanged and exposes no partial output;
- no lifecycle, public export, Option, KSR golden, or G-code behavior changes;
- all native/release/WASM/browser/static/rollback gates and both independent
  review systems pass.

The worker handoff must list changed files, RED/GREEN commands it could run,
results, incomplete checks, decisions, and residual risks. Parent command
output, current files, oracle artifacts, and runtime results—not worker prose—
are release evidence.

## Tasks

### 1. Freeze exact O32 predecessor and source evidence

Verify HEAD/origin, Orca SHA, Tier-1 run `31213611275` and exact head SHA, empty
staging, only the approved O33 spec/plan plus known-untracked/unstaged
`.pi-subagents/`, and baseline LOC. Archive under
`/tmp/task22o33-baseline-*`. Re-run RegionExpansion 74/74, PolyTree 6/6, and
O26 lifecycle 3/3. Reject every other unexpected tracked/untracked path or
baseline failure.

Read and archive the exact source body plus cited Clipper raw offset, AABB
wrapper, and ExPolygon containment. Record that upstream consumes both inputs,
sorts by source only, moves untouched sources, offsets active accumulators, and
conditionally selects the source-connected component. `expand_merge_expolygons`
and every caller remain deferred.

### 2. Freeze pinned behavioral oracle vectors under `/tmp`

Build disposable debug and `NDEBUG` C++ harnesses from the pinned Clipper source
and source-shaped O33 body. Keep all source, binaries, and output under
`/tmp/task22o33-oracle-*`. Record complete ordered contour/hole point vectors,
not counts, areas, bounds, hashes, or serialized blobs, for:

- unchanged sources with empty expansion records;
- unsorted multi-source records with untouched interior slots;
- several records and boundary IDs for one source;
- source contour plus hole safety offset;
- disconnected expansion selecting only the source component;
- any zero-result/error vector that pinned source can produce reliably.

Require debug and `NDEBUG` vectors to agree where assertions are not the tested
behavior. Reuse the already reviewed O28 fixed-MSVC comparator implementation
and >32 equal-key evidence; do not infer MSVC equal-key order from host
`std::sort`. Commit only compact human-reviewed Rust literals.

### 3. Establish a compiling RED at the approved seam

The sole writer adds:

1. new `region_expansion/merge.rs` with the exact three-argument signature and
   temporary `Ok(Vec::new())` body;
2. module registration, crate-private reexports, and function-pointer type
   assertions in both facades;
3. ordinary `mod merge_expansions;` registration and bounded test root/shards.

The RED does not yet add unused safety-offset/AABB production helpers. Use the
reviewed complete oracle vectors and consolidate tests so every new Rust file is
below 400 LOC and each test shard at most 300 LOC. Focused tests cover:

- unchanged source topology/order/moved pointer identity;
- unsorted IDs with leading/interior/trailing untouched sources;
- multiple expansion polygons and ignored mixed boundary IDs;
- exact contour/hole safety-offset topology and point order;
- disconnected multi-result sample/AABB selection;
- both coordinate scales reaching only AABB selection;
- empty contour and malformed ID panics;
- coordinate-range and sorted-group error precedence;
- move-only ownership and no predecessor cloning.

Run:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::merge_expansions
```

The code must compile and meaningful topology/cardinality/assertion tests must
fail against the stub. Archive true chronological output at
`/tmp/task22o33-red-focused.txt` before replacing the stub. A compile failure is
not RED; later mutations remain separate.

### 4. Implement the literal source merge

Add only the approved shared seams:

- `clipper::union_safety_offset_ex(&[Polygon])`, implemented by
  `offset_paths_tree(paths, SAFETY_OFFSET, JoinType::Miter,
  SAFETY_MITER_LIMIT)?.into_expolygons()` and reexported only crate-privately;
- a narrow production `sample_in_expolygons(expolygons, point, scale)` wrapper
  in O28's AABB module, reusing `BoundaryAabb::build(..., scale).sample(...)`
  with no tree/containment change.

Replace the stub with source-shaped code:

1. build `Vec<usize>` order and call `fixed_msvc_sort_by` with only referenced
   `src_id < src_id`;
2. move records through unique `Option` slots in the resulting order without
   clone;
3. reserve `src.len()`, reuse one accumulator with `clear()`, and walk a source
   owning iterator plus monotonic source index;
4. move missing sources unchanged; group all equal source IDs; ignore boundary
   IDs;
5. capture the source contour's first point, then append contour and holes after
   expansion polygons;
6. call the safety-offset helper once; emit none/one/source-sampled one for
   zero/one/multiple results;
7. use `debug_assert!` for missing multi-result sample but conditionally emit in
   release; append trailing untouched sources;
8. return the first Clipper error directly through `?`.

Do not call O29/O30/O31/O32, rescale the safety offset, pre-union, use host sort,
add ID validation, clone geometry, select largest/first fallback, retry, map
errors, or activate `expand_merge_expolygons`/lifecycle.

Run focused debug/release, complete RegionExpansion, direct safety-offset/
PolyTree, O28 AABB, O32 focused, and lifecycle regressions. Diagnose the first
point/order/topology mismatch without canonicalizing output.

### 5. Mutation and structural proof

Apply and restore one mutation at a time in a disposable candidate copy:

1. host/no/secondary-key/reversed sort, wrong fixed comparator, duplicate/omit
   permutation index, or clone instead of unique slot move;
2. missing/reordered/cloned leading/interior/trailing sources;
3. accumulator not cleared, missing/duplicate/reversed/first-only expansions,
   or boundary-ID grouping;
4. source-before-expansions, missing contour/hole, reversed hole order, or
   independent ExPolygon wrapping;
5. plain union, wrong delta/sign/scale/join/miter/shortest-edge/fill, second
   offset, or ExPolygon overload;
6. wrong zero/one/multi branch, retain all/first/largest, wrong sample, local
   scan, contour-only containment, hole-boundary rejection, or wrong scale;
7. ID validation/remap, swallowed/mapped/retried error, partial output, original
   geometry fallback, or signature/visibility/return mutation.

Record killed runtime mutations, compiler rejections, and genuine equivalent
survivors in `/tmp/task22o33-mutation-manifest.txt`. Finish with restored
focused GREEN. Structurally prove fixed-MSVC index permutation, unique moves,
one helper call per group, exact append order/constants/AABB reuse/error escape,
and absence of mutation residue.

### 6. Initial implementation review gate

Before milestone-state documentation, provide the approved spec/plan, exact O32
base, Rust diff, chronological RED, oracle vectors, focused debug/release,
mutation manifest, and structural audit to a fresh independent reviewer and
default-model OpenCode. Both must return literal `VERDICT: APPROVE` for spec
implementation.

Reviewers never edit. Synthesize every requested repair into a concrete fix
list for the sole writer. After any fix, rerun affected checks plus the complete
applicable implementation-candidate verification, refresh every impacted
oracle/mutation/structural artifact while retaining the original chronological
RED only as archival evidence, and send both reviewers the repaired diff and
fresh evidence. Both initial implementation reviews must approve that exact
repaired candidate; no pre-repair GREEN or review verdict may satisfy the gate.

### 7. Update bounded documentation

After initial implementation approval, correct O32 spec/plan, roadmap, and
option parity from pending to released with commits `2e7168f`/`699f02b`, exact
SHA `699f02b2bbc3d797f53edf5f8c65dd2614830ecb`, and Tier-1 run
`31213611275`. Update O33 spec/plan, roadmap, and option parity with actual
RED/GREEN counts, oracle, mutations/survivors, LOC, review state, and still
pending release gates.

State that O33 adds no Option, public API, lifecycle/checkpoint/persistence,
adapter, KSR golden expectation, or G-code bytes. Public slicing still consumes
O26 and returns `ProjectSlicingIncomplete`. Next boundary is only
`expand_merge_expolygons` at hpp:113/cpp:589-594; external-surface and later
slicing work remain deferred. No ARD change.

### 8. Verify the exact documented candidate

Archive fresh results for:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::merge_expansions
cargo nextest run --release -p ares-core geometry::tests::region_expansion::merge_expansions
cargo nextest run -p ares-core geometry::tests::region_expansion::wave_seeds::aabb_order
cargo nextest run -p ares-core geometry::tests::region_expansion::expand_expolygons
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core geometry::tests::clipper::offset
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Run all four existing wasm32 core/adapter checks with and without the browser
oracle feature. Build optimized default/feature WASM in separate `/tmp` target
directories, run wasm-bindgen, prove default/feature exports unchanged, and
syntax-check all browser JavaScript. Run the full Playwright suite twice in the
existing supported environment. If local host libraries are unavailable,
record the exact failure and require both CI browser runs; never label an
environment failure a product pass.

Static audit proves the exact allowlist, each Rust file `<400` LOC, each new
test shard `<=300`, clean diff/staging, no manifest/lock/dependency/ARD/public/
lifecycle/adapter/golden change, no forbidden identity/G-code/oracle/pinning/
include/unsafe/FFI/thread/platform/lint/fallback patterns, no generated artifact
staged, and no export drift. Run `git diff --check`.

### 9. Rehearse exact-O32 rollback

In a disposable worktree based on exact O32:

1. apply the complete tracked O33 candidate and prove byte identity;
2. remove only O33 merge/shared-helper exports, tests, docs, and O32 release
   corrections;
3. prove retained files match exact O32;
4. run RegionExpansion 74/74, PolyTree 6/6, and lifecycle 3/3;
5. remove the worktree and prove primary diff/digest/staging unchanged.

Archive `ROLLBACK_REHEARSAL_OK` under `/tmp`.

### 10. Final six-dimensional review and repair loop

Fresh independent and default-model OpenCode reviewers inspect the same final
documented candidate and evidence for:

1. requirements completeness;
2. source/logical correctness;
3. edge cases, ordering, IDs, scale, containment, zero/one/multi branches, and
   errors;
4. code quality, ownership, performance, visibility, forbidden patterns, and
   LOC;
5. TDD/oracle/mutation/test coverage and truthful evidence;
6. actual native/release/WASM/browser/static/rollback results.

Both must return literal `VERDICT: APPROVE`. For every finding, the parent gives
a repair list to the sole writer. Any repair requires affected checks plus the
complete exact-candidate suite, refreshed docs/evidence, and both final reviews
again on the repaired diff. No stale pre-repair evidence may be reused.

### 11. Commit, push, and exact-SHA Tier-1

Load Conventional Commits guidance. Re-audit status/diff/allowlist/LOC and stage
only approved files. Keep `.pi-subagents/`, `/tmp`, `target`, oracle artifacts,
and generated bindings untracked/unstaged. Create reviewable implementation and
documentation commits, push `main`, and prove `HEAD == origin/main`.

Wait for the push-triggered Tier-1 run. Verify its `headSha` equals the exact
pushed documentation SHA, all five jobs pass, and the WASM job passes the full
Playwright suite twice. Only then mark O33 released. Do not mark the full KSR
G-code goal complete: public slicing remains intentionally incomplete and the
next reviewed milestone begins at `expand_merge_expolygons` only.
