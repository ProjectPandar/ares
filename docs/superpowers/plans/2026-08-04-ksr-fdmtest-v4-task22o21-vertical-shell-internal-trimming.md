# Task 22O.21 — Single-region vertical-shell internal trimming Plan

Spec: `docs/superpowers/specs/2026-08-04-ksr-fdmtest-v4-task22o21-vertical-shell-internal-trimming.md`

## Status

Implemented and locally validated from approved baseline `9b2fc431f697ce3fbbf8f07b6a9ff0f9fe76cff0` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Frozen O21 evidence: parent-bound checksum `-86220837291247746226319093859583939318`, totals `[1, 460, 0, 460, 7704, 104680]`, ordered events `[460, 460, 460, 460, 259]`, 42 focused tests, 386 O10-O21 regressions, and 5,717 workspace passes with 2 skipped. Final independent implementation reviews, commit/push, and exact-commit Tier-1 CI are parent-owned ship steps.

## Validation contract

Port only pinned `PrintObject::discover_vertical_shells` release behavior at `PrintObject.cpp:2334-2342`: stable reachable internal-surface flattening, clip-only path-by-path safety intersection, ordinary hole difference append, empty continue, and final existing-solid append. Stop before regularization at `2344+`. Preserve O20 exactly beside a fresh aligned crate-private trim sidecar; do not add `InternalVoid`, regularization, fill-surface mutation, public API, dependencies, fallback, or an Ares-owned pipeline.

## Gate 0

1. The approved spec is authoritative. The independent spec reviewer and default-model OpenCode review must each have returned literal `VERDICT: APPROVE`; any spec edit returns to both.
2. Review this plan against the approved spec through an independent reviewer and default-model OpenCode. Any edit returns the complete plan to both.
3. Before RED require `HEAD == origin/main == 9b2fc431f697ce3fbbf8f07b6a9ff0f9fe76cff0`, `git -C OrcaSlicer rev-parse HEAD == 8500fcdccaa10b5099ac20d252af3a7c560046f1`, no unrelated tracked changes beyond the reviewed spec/plan, and a green exact-SHA Tier-1 run for the O20 integration repair. Do not stage `.pi-subagents/` or generated evidence.

## Task 1 — RED flat Paths and record contract

Exact production files and budgets:

- new `crates/ares-core/src/project_slice/prepare_infill/vertical_shell_trimming.rs`, at most 180 LOC;
- new children `vertical_shell_trimming/types.rs`, `stage.rs`, `trim.rs`, `cleanup.rs`, each at most 300 LOC;
- direct production-test root `vertical_shell_trimming/tests.rs` and real shards `paths.rs`, `record.rs`, `transaction.rs`, each at most 300 LOC;
- extend `geometry/clipper/boolean_paths.rs` and its existing real test module `geometry/tests/clipper/boolean_paths.rs`, keeping each below 300 LOC; minimally change `geometry/clipper/boolean_ex.rs` (below 200 LOC) only to expose its existing safety constants to sibling code; make re-export-only edits in `geometry/clipper.rs` and `geometry.rs`; duplicate `10.0_f32` / `3.0` safety literals are forbidden;
- declare the production module from `project_slice/prepare_infill.rs`.

RED before behavior:

1. Add compiling API/type shells only: `VerticalShellTrim { shell: Vec<Polygon> }`, an aligned per-object `Vec<Option<_>>`, and `PreparedPostVerticalShellTrim` retaining exact O20 fields plus fresh trims.
2. Add literal flat NonZero Paths difference REDs for empty subject/clip, repeated/disjoint/overlapping and contour-plus-CW-hole inputs, exact path/point order, no safety offset, and actual coordinate failure.
3. Add literal clip-safety intersection REDs that distinguish: a near-touching clip-only expansion from subject expansion and no-safety intersection; exact `10.0_f32`; miter `3.0`; shortest edge `abs(10 * 0.005)`; CCW Positive versus CW Negative cleanup/reversal; raw path-by-path expansion versus pre-union; empty operands; repeated and disjoint paths; and a winding/repetition witness whose exact ordered result distinguishes NonZero from EvenOdd/Positive. Freeze exact final path/point order and independent real offset/intersection coordinate failures. Reuse existing raw-offset evidence and implementation, never `offset_paths` or PolyTree.
4. Add direct record REDs for source collection-order filtering of reachable `Internal | InternalSolid`, contour then holes, exact `SafetyOffset -> Intersection -> Difference -> EmptyGate -> SolidAppend` events, intersection output before difference output, intentional solid duplication, empty projected shell with nonempty difference, empty holes, complete hole erasure, empty internal input, and empty-gate suppression of an existing solid append. Add an explicit reachability assertion/documentation witness that `InternalVoid` has no producer/variant in the approved O17-O20 envelope; do not synthesize it.
5. Add inactive ensure and aligned `None` REDs with empty/no trim geometry and zero events. Add stable error REDs for conceptual safety-offset, intersection, and difference sites using exact text `vertical-shell internal trimming geometry is outside the supported Clipper range`.
6. Run and save distinct RED evidence outside Git for exact filters `geometry::tests::clipper::boolean_paths`, `project_slice::prepare_infill::vertical_shell_trimming`, then rerun the identical filters GREEN after Task 3. RED must be missing O21 behavior/parent evidence, not compile failure or an unrelated test.

## Task 2 — RED whole-project transaction, ownership, and lifecycle

Exact integration files and budgets:

- new `project_slice/tests/prepare_infill/vertical_shell_trimming.rs` root;
- real shards `fixture.rs`, `ksr.rs`, `options.rs`, `ownership.rs`, `ownership/snapshots.rs`, `lifecycle.rs`, `cleanup.rs`, and `metamorphic.rs`, each at most 300 LOC;
- declare from `project_slice/tests/prepare_infill.rs`;
- `project_slice.rs` lifecycle wiring only; reuse/delegate iterative O20 disposal.

RED before production wiring:

1. Add a fixture that reaches O20 from 3MF bytes and calls only the O21 preparation entry. Add complete pre-geometry alignment negatives for outer object/sidecar counts, record counts, `Some`/`None`, source/transform, region/compatibility, plan/layer/current/input identities, and retained O18/O19/O20 slot relations.
2. Extend recursive allocation snapshots over both classic tree families and every retained O18 surface, O19 cache, O20 projection outer/object/record/path/point allocation. Require exact identity/content after O21 success and fresh nonaliasing trim buffers.
3. Add active two-object/later-slot failure after earlier successful O21 events. Require source object/slot order, no partial successor, exact stable error, predecessor drop probe, and whole-project stage-before-move.
4. Add depth-10,000 two-tree constrained-stack witnesses for each O21 conceptual failure, direct-success disposal, and public incomplete disposal. Drain trim geometry iteratively, reconstruct exact O20, and delegate to O20 cleanup; never recursively drop predecessor trees.
5. Add public lifecycle RED: O21 invoked exactly once after O20 and still returns `ProjectSlicingIncomplete`. Require zero O21 invocations and unchanged exact errors for spiral, counterbore, multi-region, interface shells, active extra bridge, O17 geometry, both O19 offset failures, and every O20 failure class.
6. Add real typed archive REDs: inactive ensure modes produce empty trims/zero events; active `EnsureAll` produces characterized trims; model-part ensure precedence; `sparse_infill_density` replacement to `100` activates reachable `InternalSolid`; ZIP reverse Stored/Unix repack and non-slicing rename preserve output; exact component X scaling changes source/trim geometry without changing option gating.
7. Add KSR parent-capture RED: independently parse twice, guard exact O19 successor checksum `148296943860974241781127169756103364063`, O19 totals `[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, O20 checksum `-106767561006193260948265111057697183253`, O20 totals `[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and O20 events `[1830, 917, 1539, 749, 0, 0, 0, 0]`; define O21 object/slot/None/shell/path/point/coordinate delimiters and ordered step totals before freezing new literals.

## Task 3 — GREEN source rewrite and transaction

1. Add ordinary `difference_polygons_paths` through existing flat `Clipper::execute_paths(Difference, NonZero, NonZero)`. Expose the existing `boolean_ex.rs` safety constants with sibling-only visibility, then add the clip-safety intersection adapter by reusing `raw_offset_paths` with those shared constants, `JoinType::Miter`, preserving per-path orientation/order and passing its un-unioned output as clip to flat NonZero intersection. Do not duplicate safety literals.
2. Implement one stable source-order surface flattener over reachable `Internal | InternalSolid`, allocating contour then holes. Implement a second solid-only scan only after the nonempty gate. Do not add `RegionSurfaceKind::InternalVoid` or any producer.
3. For populated inactive records return `Some(empty trim)` without events. Preserve aligned current `None` as `None`.
4. For active records call safety intersection first, ordinary difference second, append without union, gate emptiness, then append fresh solid paths verbatim. Preserve every input/output/path/point order and map actual Clipper errors once to the stable O21 error.
5. Validate all alignment before geometry. Stage every object/record while borrowing O20. Only after complete success destructure/move exact O20 fields beside trims. On error dispose O20 iteratively with no successor.
6. Add iterative O21 successor disposal, wire public slicing through O21 once, and retain `ProjectSlicingIncomplete`.
7. Rerun every Task 1/2 RED filter GREEN, then focused O20/O19/O18 regressions.

## Task 4 — Freeze real provenance and KSR

1. Freeze exact real-archive active/inactive, 100%-density solid append, model-part precedence, ZIP/name invariance, and component-scale facts without fixture identity/name/hash/layer-count/geometry branching.
2. Freeze parent-bound O21 checksum, totals `[objects, slots, none, some, shell_paths, shell_points]`, and ordered conceptual event totals only after clean implementation output. Delimit all structure so operation reordering cannot pass a final-only checksum.
3. Parse independently twice and remove temporary capture diagnostics. Reassert exact O19/O20 parent evidence before every O21 literal.

## Task 5 — Full gates, six-dimensional review, docs, and ship

1. Update architecture, roadmap, spec, and plan with exact boundary, reachable-kind limitation, Paths ordering, ownership, checksum/totals/events, test counts, and next boundary `PrintObject.cpp:2344`. Because the spec/plan diff changes, rerun both required spec reviewers and both required plan reviewers to literal approval before the final implementation review.
2. On that complete documentation-inclusive diff, run focused O21 filters; an explicit O10-O21 regression expression covering Classic, layer-region, O17/O18/O19/O20/O21; and `cargo nextest run --workspace --no-fail-fast`.
3. On the same diff, run `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, default and `task22n-browser-oracle` checks for both `ares-core` and `ares-wasm` on `wasm32-unknown-unknown`, `cargo fmt --all -- --check`, and `git diff --check`.
4. On the same diff, run the LOC audit requiring every Rust source/test `<400 LOC` and every new O21 shard `<=300 LOC`. Audit added Rust for no `unsafe`, `include!`, `include_bytes!`, broad `allow`, reference-G-code access, fixture identity/hash/layer/geometry branch, Orca command/FFI, or fallback. Require no dependency diff, no source-text/hash/line pinning tests, and no `.pi-subagents/` or `target/parity/` staged files.
5. Audit rollback mechanically and document it: restore `project_slice.rs` terminal consumption to O20; remove only O21 module/state/wiring/tests/docs and the two Paths adapters; revert only the sibling visibility change to the pre-existing safety constants; retain all O20 code and behavior. The existing O20 direct preparation/disposal and public-incomplete focused regressions must pass under that boundary without actually deleting reviewed work.
6. Dispatch an independent review-only thread over requirements completeness, logic correctness, edge cases, code quality, test coverage, and actual execution, plus the same final diff/evidence through default-model OpenCode. Both inspect all code, tests, architecture, roadmap, spec, and plan. Main thread fixes all blocking findings; then rerun all affected and full tests/checks/audits on the documentation-inclusive diff, rerun spec/plan review gates if those docs changed, and return one identical revised final diff to both implementation reviewers until each emits literal `VERDICT: APPROVE`.
7. Create small Conventional Commits, push `main`, verify clean `HEAD == origin/main`, and require the exact commit's full Tier-1 native matrix and optimized browser-WASM/Playwright job to pass. Any CI repair updates evidence/docs, reruns all full gates, and returns through every affected spec/plan and implementation review gate before final approval.

## Frozen local execution evidence

The three compiling RED filters are preserved at
`/tmp/task22o21-red-boolean-paths.txt`, `/tmp/task22o21-red-record.txt`, and
`/tmp/task22o21-red-integration.txt`; their failures were missing O21 adapter,
record, lifecycle, and parent evidence rather than compile or unrelated
failures. Post-review strengthened-suite mutation REDs at
`/tmp/task22o21-red-final-boolean-paths.txt`,
`/tmp/task22o21-red-final-record.txt`, and
`/tmp/task22o21-red-final-integration.txt` use compiling adapter-only stubs to
prove behavior-sensitive failures across all final 11/10/21 filters, followed
by byte-exact production restoration. Identical GREEN filters pass, and the
combined focused command passes 42 tests. The explicit O10-O21 expression passes 386 tests; workspace Nextest
passes 5,717 with 2 skipped. Native workspace all-target check and strict
all-target/all-feature Clippy pass.

Two clean KSR captures independently reconstruct the exact O19 and O20 parent
evidence before freezing O21 checksum
`-86220837291247746226319093859583939318`, totals
`[1, 460, 0, 460, 7704, 104680]`, and ordered events
`[460, 460, 460, 460, 259]`. The implementation stops before regularization;
the next boundary is `PrintObject.cpp:2344`. Rollback restores O20 terminal
consumption, removes O21 state/wiring/tests/docs and the two O21 flat-Paths
adapters, and reverts only the sibling visibility change on existing safety
constants.

## Stop condition

Stop O21 only when internal trimming is transactionally derived from typed aligned O20 state with exact flat Paths safety/difference/append/gate behavior, O20 allocation identity and deep cleanup are proven, KSR evidence is parent-bound and repeatable, public slicing reaches O21 once and remains incomplete before regularization, all local and Tier-1 gates pass, both final reviewers approve, docs are synchronized, and commits are pushed.
