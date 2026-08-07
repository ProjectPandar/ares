# Task 22O.32 Implementation Plan

## Execution state

Tasks 1-6 are complete. The compiling chronological RED failed 5/5; focused
debug/release pass 5/5 and RegionExpansion passes 74/74. Thirteen runtime
mutations are killed, two type-shape mutations are compiler-rejected, and the
final Rust LOC are 266/81/177/8/253. Initial independent and default-model
OpenCode implementation reviews both returned literal `VERDICT: APPROVE`.
Documentation now records released O31 and locally implemented O32. A test-only
helper initially exceeded the repository's five-argument Clippy threshold; it
was tuple-packed without production change, then focused tests, workspace
6,015/6,015 with 2 skipped, check, warning-denying Clippy, rustfmt, all WASM
build/export/syntax gates, static audits, and exact-O31 rollback passed. Local
Playwright cannot load Chromium because `libglib-2.0.so.0` is absent; the
exact-SHA CI browser runs remain mandatory. Final independent six-dimensional
and default-model OpenCode reviews both returned literal `VERDICT: APPROVE`.
Commit/push and exact-SHA Tier-1 remain pending.

## Goal, reviewed spec, and baseline

Port only pinned OrcaSlicer `Algorithm/RegionExpansion.cpp:522-534` and
`RegionExpansion.hpp:102-108` as the crate-private `expand_expolygons` helper.
The approved design is
`docs/superpowers/specs/2026-08-07-ksr-fdmtest-v4-task22o32-expand-expolygons.md`.
Its independent and default-model OpenCode spec reviewers returned literal
`VERDICT: APPROVE`.

Exact predecessor is released O31 at
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`; exact-SHA Tier-1 run
`31196271880` is green. Pinned Orca is
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Baseline RegionExpansion is
69/69, PolyTree 6/6, and O26 lifecycle 3/3. Public slicing remains incomplete.

One worker is the sole Rust writer. Reviewers never edit. The parent inspects
all diffs and runs authoritative RED/GREEN, mutation, full verification,
review-repair, commit, push, and exact-SHA CI gates.

Allowed Rust files:

- `crates/ares-core/src/geometry/region_expansion/propagate.rs`;
- `crates/ares-core/src/geometry/region_expansion.rs`;
- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/tests/region_expansion.rs`;
- new
  `crates/ares-core/src/geometry/tests/region_expansion/expand_expolygons.rs`.

Allowed docs are the O32 spec/plan, `docs/roadmap.md`,
`docs/architecture/option-parity-v4.md`, and O31 spec/plan release corrections.
Stop and amend/re-review the spec and plan before touching another production
or test file.

## Validation contract

Acceptance requires direct observation that:

- the API is crate-private and has the exact six arguments and nested polygon
  return shape;
- output always contains exactly `src.len()` slots after successful O29
  propagation, including leading/interior/trailing empty slots;
- O29's boundary-first records are redistributed into source-index slot order
  while per-slot polygon order and complete point topology remain unchanged;
- multiple raw polygons in one slot are neither unioned nor truncated;
- the same expansion, step, max steps, and scale reach O29 unchanged;
- builder preconditions and discovery/propagation errors preserve O29 order;
- no lifecycle, public export, Option, KSR golden, or G-code behavior changes;
- all native/release/WASM/browser/static/rollback gates and both independent
  review systems pass.

The worker handoff must list changed files, exact RED/GREEN commands it could
run, results, incomplete checks, decisions, and residual risks. Parent command
output, current files, and generated runtime artifacts—not worker prose—are the
release evidence.

## Tasks

### 1. Freeze exact predecessor and source evidence

Verify HEAD/origin and Orca SHA, exact run `31196271880`, empty staging, only
O32 docs plus the known-untracked and unstaged `.pi-subagents/` exception, and
LOC. Archive under `/tmp/task22o32-baseline-*`. Re-run RegionExpansion 69/69,
PolyTree 6/6, and O26 lifecycle 3/3. Reject every other unexpected tracked or
untracked path and any baseline failure.

Read the exact source body and O29 scalar wrapper. Record that upstream first
allocates `src.size()` slots, then consumes the complete scalar propagation
result and moves each polygon to `out[src_id]`. Do not build a new C++ oracle
unless a complete expected vector is missing from released O29 evidence; any
oracle remains entirely under `/tmp`.

### 2. Establish compiling RED at the approved seam

The worker adds:

1. the exact six-argument `expand_expolygons` signature with the narrow
   reasoned Clippy expectation and temporary `Ok(Vec::new())` body;
2. crate-private reexports and function-pointer type assertions in both
   facades;
3. ordinary `mod expand_expolygons;` registration and the new focused shard.

Use complete point vectors, not counts/bounds/areas/hashes. Consolidate the
approved matrix into a shard at most 300 LOC:

- empty source and source-sized empty slots for empty/no-wave boundaries;
- invalid scalar inputs before empty completion;
- one source receiving every raw polygon in exact order;
- leading/interior/trailing empty slots;
- boundary-first flat O29 order redistributed to source-index slots;
- Normal/LargeBed complete distinct vectors and explicit O29 grouping parity;
- discovery and propagation errors escaping unchanged.

Run:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expand_expolygons
```

The code must compile and nonempty/cardinality assertions must fail against the
stub. Archive the true chronological output at `/tmp/task22o32-red-focused.txt`
before replacing the stub. A compile failure is not RED. Later mutations are
recorded separately.

### 3. Implement the literal source helper

Replace only the stub body with source-shaped code:

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

Call O29 exactly once. Do not add an empty shortcut, builder, discovery, direct
O27/O30/O31 call, union, sorting, ID check/remap, clone, rescale, validation,
error mapping, retry, fallback, or partial output. Run focused debug/release,
complete RegionExpansion, O29, O31, and PolyTree filters. Diagnose the first
point/slot/order mismatch without normalizing output.

### 4. Mutation and structural proof

Apply and restore one mutation at a time in a disposable candidate copy. Test:

1. wrong output cardinality, omitted empty slots, or allocation after an early
   empty return;
2. skipped/duplicated/replaced O29 call;
3. swapped `f32`s, substituted max steps, rescaled value, or changed scale;
4. position/boundary grouping instead of `src_id` indexing;
5. sorted/flattened/compacted/reversed output;
6. overwrite instead of append, first-only retention, or per-slot union;
7. topology clone/change or wrong source slot;
8. swallowed/mapped discovery or propagation errors;
9. signature/return-shape mutation.

Record killed runtime mutations, compiler rejections, and genuine equivalent
survivors in `/tmp/task22o32-mutation-manifest.txt`. Finish with restored
focused GREEN. Structurally prove source-sized allocation before the one O29
call, direct `src_id as usize` append, and absence of O27/O28/O30/O31/union,
rescale, shortcut, or mutation residue.

### 5. Initial implementation review gate

Before milestone-state documentation, provide the approved spec/plan, exact
O31 base, Rust diff, chronological RED, focused debug/release output, mutation
manifest, and structural audit to a fresh independent reviewer and default-model
OpenCode. Both must return literal `VERDICT: APPROVE` for spec implementation.
A reviewer request is synthesized into a fix list for the sole writer. After
any fix, rerun affected checks and both implementation reviews.

### 6. Update bounded documentation

After implementation approval, update O31 spec/plan from locally implemented
to released with commits `7113f7c`/`1f89dd3`, exact SHA
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`, and Tier-1 run `31196271880`.
Update O32 spec/plan, roadmap, and option parity with actual RED/GREEN counts,
mutations/survivors, LOC, review state, and still-pending release gates.

State that O32 adds no Option, public API, lifecycle/checkpoint/persistence,
adapter, KSR golden expectation, or G-code bytes. Public slicing still consumes
O26 and returns `ProjectSlicingIncomplete`. Next boundary is only
`merge_expansions_into_expolygons` at hpp:110-111/cpp:536-587;
`expand_merge_expolygons` and later external-surface work remain deferred. No
ARD change.

### 7. Verify the exact documented candidate

Archive fresh results for:

```text
cargo nextest run -p ares-core geometry::tests::region_expansion::expand_expolygons
cargo nextest run --release -p ares-core geometry::tests::region_expansion::expand_expolygons
cargo nextest run -p ares-core geometry::tests::region_expansion::composition
cargo nextest run -p ares-core geometry::tests::region_expansion::expolygon_composition
cargo nextest run -p ares-core geometry::tests::region_expansion
cargo nextest run -p ares-core geometry::tests::clipper::polytree
cargo nextest run -p ares-core project_slice::tests::prepare_infill::horizontal_shell_propagation::lifecycle
cargo nextest run --workspace
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Run all four existing wasm32 core/adapter checks with and without the browser
oracle feature. Build optimized default and feature WASM in separate `/tmp`
target directories, run wasm-bindgen, prove default/feature exports unchanged,
and syntax-check all browser JavaScript. Run the full Playwright suite twice in
the existing supported environment. If local host libraries are unavailable,
record the exact failure and require both CI browser runs in the exact-SHA WASM
job; do not call an environment failure a product pass.

Static audit must prove exact allowlist, every Rust file `<400` LOC, new shard
`<=300`, clean diff/staging, no manifest/lock/dependency/ARD/public/lifecycle/
adapter/golden change, no forbidden identity/G-code/oracle/pinning/include/
unsafe/FFI/thread/platform/lint/fallback patterns, no generated artifact staged,
and no export drift. Run `git diff --check`.

### 8. Rehearse rollback

In a disposable worktree based on exact O31:

1. apply the complete tracked O32 candidate and prove byte identity;
2. remove only O32 function/reexports/assertions, shard/registration, O32 docs,
   and O31 release-state corrections;
3. prove retained files match exact O31;
4. run RegionExpansion 69/69, PolyTree 6/6, and lifecycle 3/3;
5. remove the worktree and prove primary diff/digest/staging unchanged.

Archive `ROLLBACK_REHEARSAL_OK` under `/tmp`.

### 9. Final six-dimensional review and repair loop

Fresh independent and default-model OpenCode reviewers inspect the same final
documented candidate and evidence. They must evaluate:

1. requirements completeness;
2. source and logical correctness;
3. edge cases, cardinality, ordering, IDs, scale, and errors;
4. code quality, visibility, forbidden patterns, and LOC;
5. TDD/oracle/mutation/test coverage and truthful evidence;
6. actual native/release/WASM/browser/static/rollback results.

Both must return literal `VERDICT: APPROVE`. Reviewers never edit. For every
finding, the parent issues a concrete repair list to the sole writer. Any repair
requires affected checks plus the complete exact-candidate suite, refreshed
documentation/evidence, and both final reviews again on the repaired diff.

### 10. Commit, push, and exact-SHA Tier-1

Load the Conventional Commits guidance. Re-audit status/diff/allowlist/LOC and
stage only approved files. Keep `.pi-subagents/`, `/tmp`, `target`, oracle
artifacts, and generated bindings untracked/unstaged. Create reviewable
implementation and documentation commits, push `main`, and prove
`HEAD == origin/main`.

Capture the pushed documentation SHA. O32 is released only after a Tier-1 run
has exactly that `headSha`, `status=completed`, `conclusion=success`, and all
five format, WASM/browser, Ubuntu, Windows, and macOS jobs succeed. Do not claim
full external-surface or KSR G-code parity.

## Completion and rollback

O32 completes only after exact source grouping, complete slot/vector/error
proof, restored mutations, native/release/WASM/browser/static/rollback gates,
dual approvals, reviewed commits, push, and exact-SHA Tier-1. Mechanical
rollback retains released O27-O31 and removes only the bounded O32 artifacts.
