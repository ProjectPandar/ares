# Task 22O.22 — Single-region vertical-shell morphology regularization Plan

Spec: `docs/superpowers/specs/2026-08-04-ksr-fdmtest-v4-task22o22-vertical-shell-regularization.md`

## Status

Implemented and locally validated from Ares baseline `7d607b4bcda5ede5d5eb1d5c513148ecf1ab25d4` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Both spec and plan received independent and default-model OpenCode literal approval before production work. Frozen O22 evidence: checksum `134936948052282121922360252649864225707`, totals `[1, 460, 0, 460, 632, 632, 128, 34557]`, ordered events `[259, 259, 259, 259]`, radii digest `-119839535044106185061007902266478724784`, 11 direct tests, 22 integration tests, 346 O10-O22 regressions, and 5,750 workspace passes with 2 skipped. Final implementation review and exact-commit Tier-1 remain parent-owned ship gates.

## Validation contract

Port only pinned `PrintObject::discover_vertical_shells` release behavior at `PrintObject.cpp:2344-2367`: aligned solid-infill spacing, exact `f32` radii, NonZero `union_ex`, Square-join `offset2_ex`, and Square-join `shrink_ex`. Stop before `object_volume` at line 2369. Preserve O21 exactly beside a fresh aligned crate-private regularization sidecar; do not add neighbor/area filtering, `fill_surfaces` mutation, new options, public API, dependencies, fallback, or an Ares-owned pipeline.

## Gate 0 — Approve the design

1. Confirm `HEAD == origin/main == 7d607b4bcda5ede5d5eb1d5c513148ecf1ab25d4`, pinned Orca checkout `8500fcdccaa10b5099ac20d252af3a7c560046f1`, clean tracked state apart from reviewed O22 docs, and green exact-SHA O21 Tier-1 run `30941387670`.
2. Send the complete source-cited spec through one independent review-only agent and a separate default-model OpenCode review. Any edit returns the full spec to both.
3. Send this complete plan through both reviewers after spec approval. Any edit returns the full plan to both. Require literal `VERDICT: APPROVE` from all four reviews before RED or production edits.

## Task 1 — RED direct morphology contract

Exact production files and budgets:

- new `crates/ares-core/src/project_slice/prepare_infill/vertical_shell_regularization.rs`, at most 180 LOC;
- new real children `vertical_shell_regularization/types.rs`, `stage.rs`, `regularize.rs`, and `cleanup.rs`, each at most 300 LOC;
- direct production-test root `vertical_shell_regularization/tests.rs` and real shards `radii.rs`, `morphology.rs`, and `transaction.rs`, each at most 300 LOC;
- minimally extend `geometry/clipper/offset/expolygon.rs` (remaining below 300 LOC) and re-exports so the existing two-stage `offset2_ex` implementation has an inter-stage observer entry used by O22, while the ordinary entry delegates through the same implementation unchanged; do not duplicate either offset stage or change production geometry semantics;
- declare the production module from `project_slice/prepare_infill.rs`.

RED before behavior:

1. Add compiling type/API shells only: `VerticalShellRegularization { regularized_shell: Vec<ExPolygon> }`, aligned per-object `Vec<Option<_>>`, and `PreparedPostVerticalShellRegularization` retaining exact O21 fields plus fresh regularizations.
2. Add exact-bit radius REDs proving one `i64 as f32` cast, `* 1.05_f32` for `min_perimeter_infill_spacing`, then left-associated `0.5_f32 * 0.65_f32`, `0.5_f32 * 1.2_f32`, and `0.2_f32` arithmetic. Freeze negation/addition/subtraction order for all offset deltas using ordinary, odd, and large-but-supported spacing values.
3. Add literal morphology REDs for empty shell, a union that becomes empty, disjoint and touching polygons, contour with hole, material narrower than `2 * narrow_ensure`, a gap crossed by the grow stage, and final shrink. Freeze exact ExPolygon/contour/hole/point order and output coordinates. At least one witness must distinguish `JoinType::Square` from Miter and Round. Snapshot every input and output allocation and require fresh regularization ExPolygon/contour/hole/point buffers with no aliasing.
4. Add direct actual coordinate-failure witnesses for union, offset2 first stage, offset2 second stage, and shrink, each proving the failure originates at that operation. Add exact event REDs for `Union -> Offset2First -> Offset2Second -> Shrink`, including O21 empty-input zero events and nonempty input whose empty union still reaches both offset2 stages and shrink. Add exact stable error REDs for each conceptual geometry site with `vertical-shell regularization geometry is outside the supported Clipper range`.
5. Add one minimal geometry entry that runs the same existing `offset2_ex` body and invokes an observer after `offset_expolygons_paths` succeeds but before `offset_paths_tree` begins; make the ordinary `offset2_ex` delegate to it with a no-op observer. O22 uses this entry to record/fail `Offset2Second` at the actual stage boundary. This is an observer seam only: no duplicated offset code, changed delta/order, test-only production branch, or alternate algorithm.
6. Run and preserve compiling behavior-failure RED output at `/tmp/task22o22-red-direct.txt` for the exact direct O22 filters. RED must fail for missing morphology behavior, not compilation or unrelated tests. Rerun the byte-identical command/filter GREEN after Task 3.

## Task 2 — RED whole-project alignment, ownership, and lifecycle

Exact integration files and budgets:

- new `project_slice/tests/prepare_infill/vertical_shell_regularization.rs` root;
- real shards `fixture.rs`, `ksr.rs`, `options.rs`, `ownership.rs`, `ownership/snapshots.rs`, `lifecycle.rs`, `cleanup.rs`, and `metamorphic.rs`, each at most 300 LOC;
- declare from `project_slice/tests/prepare_infill.rs`;
- `project_slice.rs` lifecycle wiring only; delegate iterative O21 disposal.

RED before production wiring:

1. Add a fixture reaching O21 from 3MF bytes and calling only O22 preparation. Add complete pre-geometry alignment mismatch tests for every outer relation (`objects`, `caches`, `projections`, `trims`, `predecessor.objects`), every record-count relation (`object.records`, cache/projection/trim records, inputs, prelude records, plan layers, and `lslices`), every `Some`/`None` slot relation, and every source/transform, region/compatible-region, planned-layer/layer-id/current-layer/current-region/input identity invariant.
2. Extend recursive allocation snapshots over both classic tree families and every retained O18 surface, O19 cache, O20 projection, and O21 trim outer/object/record/path/point allocation. Require exact O21 identity/content after success and fresh nonaliasing O22 ExPolygon/contour/hole/point buffers.
3. Add active multi-object/later-slot failures after earlier successful O22 events. Require object/slot order, no partial successor, exact stable error, predecessor drop probe, and whole-project stage-before-move.
4. Add depth-10,000 two-tree constrained-stack witnesses for every O22 failure class, direct-success disposal, and public-incomplete disposal. Drain O22 regularization iteratively, reconstruct exact O21, and delegate to O21 cleanup.
5. Add public lifecycle RED: O22 invoked once after O21 and public slicing still returns `ProjectSlicingIncomplete`. Require zero O22 invocations and unchanged exact earlier errors for spiral, counterbore, multi-region, interface shells, active extra bridge, O17 geometry, O19, O20, and O21 failures.
6. Add typed real-archive REDs for inactive modes, active `EnsureAll`, model-part precedence, ZIP reverse Stored/Unix repack, non-slicing rename, component X scaling, and a typed solid-infill-flow spacing mutation. The spacing mutation must prove the complete chain: changed retained `ClassicPreludeRecord::solid_infill_spacing`, exact changed `min_perimeter_infill_spacing` and three radius/delta bit patterns, and changed ordered regularization output/digest. Assert behavioral provenance rather than archive identity.
7. Add KSR parent-capture RED: independently parse twice and first guard O19 successor checksum `148296943860974241781127169756103364063` and totals `[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`; O20 checksum `-106767561006193260948265111057697183253`, totals `[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and events `[1830, 917, 1539, 749, 0, 0, 0, 0]`; and O21 checksum `-86220837291247746226319093859583939318`, totals `[1, 460, 0, 460, 7704, 104680]`, and events `[460, 460, 460, 460, 259]`. Define O22 structural delimiters and exact radii/event counts before freezing literals.
8. Run and preserve compiling post-implementation behavior-failure evidence after all Task 2 tests compile. The exact whole-project factor mutation lives at `/tmp/task22o22-red-integration.txt`; separate production mutations cover all 5 alignment classes at `/tmp/task22o22-red-integration-alignment.txt`, public lifecycle wiring at `/tmp/task22o22-red-integration-lifecycle.txt`, and genuine later-slot transaction staging at `/tmp/task22o22-red-integration-transaction.txt`. Together they must demonstrate transaction, alignment, lifecycle, typed-provenance, and KSR successor behavior rather than compile or unrelated failures. Do not present these mutations as chronological pre-implementation RED. Restore every production artifact byte-for-byte and rerun the affected tests plus the exact whole-project filter GREEN after Task 3.

## Task 3 — GREEN source rewrite and transaction

1. Implement a radius helper that casts aligned scaled spacing to `f32` once, multiplies by `1.05_f32`, and evaluates all three source radius expressions in exact order. Do not round, clamp, validate, or convert through `f64`.
2. For a nonempty O21 trim, call the existing `union_ex(shell, FillRule::NonZero)`. Pass even an empty union result through both offset2 stages and shrink exactly as the nested source expression does.
3. Call the reviewed inter-stage-observer entry over the existing `offset2_ex` body with `(-narrow_ensure, narrow_ensure + narrow_sparse, JoinType::Square, 3.0)`, recording/failing the second-stage site only after the first succeeds. Then call existing `offset_expolygons` for shrink with `-(narrow_sparse - tiny_overlap)`, `JoinType::Square`, `3.0`. Do not sort, normalize, union again, deduplicate, filter by area, or intersect with neighbors.
4. Preserve aligned `None` as `None` and populated empty/inactive trims as `Some(empty regularization)` with zero geometry. Map all actual Clipper failures to the one stable O22 error.
5. Validate all O21 alignment before geometry. Stage all regularizations while borrowing O21; only after complete success destructure/move exact O21 fields. On failure, dispose O21 iteratively and expose no successor.
6. Add iterative O22 successor disposal, wire public slicing through O22 once, and retain `ProjectSlicingIncomplete`.
7. Rerun the byte-identical direct and whole-project commands GREEN after restoring the factor mutation, rerun every supplemental alignment/lifecycle/transaction mutation target GREEN after byte-exact restoration, and then run focused O18-O21 parent regressions.

## Task 4 — Freeze real provenance and KSR

1. Freeze exact real-archive active/inactive, typed spacing mutation, model-part precedence, ZIP/name invariance, and component-scale facts without fixture identity/name/hash/layer-count/geometry branching.
2. Freeze parent-bound O22 checksum, totals `[objects, slots, none, some, expolygons, contours, holes, points]`, ordered conceptual events, and exact radius bits only after clean implementation output. Delimit every structure so operation or output-order changes cannot collide trivially.
3. Parse independently twice, reassert all O19/O20/O21 parent evidence, compare captures, remove diagnostics, and make the independent result the committed literals. Never read the reference G-code.

## Task 5 — Full gates, six-dimensional review, docs, and ship

1. Update architecture, roadmap, spec, and plan with the exact O22 seam, cast/radius/operation order, ownership, checksum/totals/events, tests, and next boundary `PrintObject.cpp:2369`. Rerun both spec reviewers and both plan reviewers after documentation changes.
2. Run focused O22 tests, explicit O10-O22 regressions, and `cargo nextest run --workspace --no-fail-fast` on the documentation-inclusive diff.
3. Run `cargo check --workspace --all-targets`, strict all-target/all-feature Clippy, default and `task22n-browser-oracle` checks for both `ares-core` and `ares-wasm` on `wasm32-unknown-unknown`, `cargo fmt --all -- --check`, and `git diff --check`.
4. Require every Rust source/test `<400 LOC` and every new O22 shard `<=300 LOC`. Audit added Rust for no `unsafe`, `include!`, `include_bytes!`, broad lint allowance, reference-G-code access, fixture identity/hash/layer/geometry branches, Orca command/FFI, or fallback. Require no dependency diff, no source-text/hash/line pinning tests, and no `.pi-subagents/` or `target/parity/` staged files.
5. Audit rollback mechanically: restore O21 terminal consumption in `project_slice.rs`; remove only O22 module/state/wiring/tests/docs plus the O22-only inter-stage observer entry, its re-exports, and its geometry tests; restore ordinary `offset2_ex` to its original two-line body and retain all O21 behavior. Existing O21 direct preparation/disposal and public-incomplete regressions must remain green without deleting reviewed work.
6. Dispatch an independent review-only thread over requirements completeness, logic correctness, edge cases, code quality, test coverage, and actual execution, plus a separate default-model OpenCode review over the same final diff and evidence. The parent sole writer fixes every blocking finding, reruns affected and full gates, reruns changed-doc review gates, and returns the identical revised diff to both implementation reviewers until literal `VERDICT: APPROVE`.
7. Create small Conventional Commits, push `main`, verify clean `HEAD == origin/main`, and require the exact commit's complete Tier-1 native matrix and optimized browser-WASM/Playwright job to pass. Any CI repair repeats all affected gates and reviews.

## Frozen local execution evidence

Two clean KSR captures reassert all O19-O21 parent literals and freeze O22 checksum `134936948052282121922360252649864225707`, totals `[1, 460, 0, 460, 632, 632, 128, 34557]`, ordered events `[259, 259, 259, 259]`, and exact-radii digest `-119839535044106185061007902266478724784`. The exact direct filter passes 11 tests, the exact integration filter passes 22 tests, the explicit O10-O22 filter passes 346 tests, and workspace Nextest passes 5,750 tests with 2 skipped. Strict all-target/all-feature Clippy passes.

The implementation worker had no command executor, so no chronological pre-implementation RED is claimed. Honest post-implementation mutation REDs remove only the source `1.05_f32` factor: `/tmp/task22o22-red-direct.txt` records 4 behavior failures among the same 11 direct tests and `/tmp/task22o22-red-integration.txt` records 2 behavior failures among the same 22 integration tests. Supplemental compiling production mutations fail all 5 alignment tests, the public lifecycle test, and the genuine later-slot transaction test at `/tmp/task22o22-red-integration-{alignment,lifecycle,transaction}.txt`. Current tuple-signature source artifacts are preserved at `/tmp/task22o22-green-production-{regularize,stage,project-slice}.rs`; each mutation was restored byte-exactly before affected GREEN and full-workspace validation.

## Stop condition

Stop O22 only when the initial regularized shell is transactionally derived from typed aligned O21 state with exact source cast/`1.05_f32`/radii/NonZero-union/Square-offset ordering, O21 allocation identity and deep cleanup are proven, KSR evidence is parent-bound and repeatable, public slicing reaches O22 once while remaining incomplete before `object_volume`, all local and exact-commit Tier-1 gates pass, all required reviewers approve, docs are synchronized, and commits are pushed.
