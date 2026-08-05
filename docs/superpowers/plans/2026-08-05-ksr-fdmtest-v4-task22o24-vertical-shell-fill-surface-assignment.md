# Task 22O.24 — Single-region vertical-shell fill-surface assignment Plan

Spec: `docs/superpowers/specs/2026-08-05-ksr-fdmtest-v4-task22o24-vertical-shell-fill-surface-assignment.md`

## Status

Drafted from Ares baseline `6dde113688f369c83bca139cd51f45ca9441bdf1` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. The revised spec has literal `VERDICT: APPROVE` from the independent reviewer at `/tmp/task22o24-spec-independent-rereview.md` and separate default-model OpenCode reviewer at `/tmp/task22o24-spec-opencode-rereview.txt`. Production work must not begin until this complete plan receives the same two approvals.

## Validation contract

Port only pinned `PrintObject::discover_vertical_shells` behavior at `PrintObject.cpp:2402-2432`: exact mixed two-pass NonZero solid intersection; ordered Internal and InternalVoid differences against the original collection; stable external retention; exact Internal, InternalVoid, InternalSolid append order; default metadata; and the empty-filter no-op. Add only source-cited `InternalVoid = 8` with non-bridge semantics and the one missing flat-Polygons/ExPolygons intersection adapter. Stage the whole project before mutating `fill_surfaces`, retain the exact O23 graph outside the intentional mutation seam, and continue returning `ProjectSlicingIncomplete`. Do not add a void producer, later surface kinds, new options, dependencies, public API, reference-G-code behavior, fallback, or an Ares-owned pipeline.

## Gate 0 — Baseline and reviewed design

1. Confirm Ares `HEAD == origin/main == 6dde113688f369c83bca139cd51f45ca9441bdf1`, Orca checkout `8500fcdccaa10b5099ac20d252af3a7c560046f1`, and no tracked worktree changes except the reviewed O24 spec/plan.
2. Require exact-SHA O23 Tier-1 run `30982832344` to finish with format, macOS, Ubuntu, Windows, optimized browser-WASM/export audit, and both Playwright runs successful before O24 ships. If it fails, repair O23 first and repeat affected reviews, commit, push, and exact-SHA CI.
3. Preserve the final spec approvals named above.
4. Send this complete plan to one fresh independent review-only agent and one separate default-model OpenCode review. Any plan edit returns the complete revised plan to both. Require literal `VERDICT: APPROVE` from both before production edits.

## Task 1 — Source vocabulary and mixed `_ex` adapter, test-first

Files and budgets:

- modify `crates/ares-core/src/project_slice/region_slices.rs` and its existing ordinary test module;
- modify `crates/ares-core/src/geometry/clipper/boolean_ex.rs` and narrow reexports in `geometry/clipper.rs` and `geometry.rs`;
- add an ordinary geometry test shard under `crates/ares-core/src/geometry/tests/clipper/`, each file at most 300 LOC.

Steps:

1. Add a minimal compiling API shell before RED tests reference new items: declare a deliberately wrong `InternalVoid` discriminant/non-bridge result that keeps every exhaustive match compiling, and declare `intersection_polygons_ex(subject: &[Polygon], clip: &[ExPolygon])` with an intentional runtime stub returning an observably wrong empty result. This shell is temporary TDD scaffolding and must not enter public slicing or survive GREEN.
2. Add behavioral REDs that compile, execute, and fail assertions for `RegionSurfaceKind::InternalVoid as u8 == 8`, `InternalVoid.is_bridge() == false`, and the existing exhaustive bridge vocabulary. Preserve the no-default match contract.
3. Add compiling direct REDs for the mixed adapter using empty, disjoint, partial, full, multi-component, holed, mixed-winding, and nested-island geometry. Freeze exact output ExPolygon/contour/hole/point order. Include a mixed-winding hole witness that fails a guessed conversion which forces every standalone subject path into contour winding; do not claim that a wrapper-only conversion feeding byte-identical Paths is behaviorally distinguishable.
4. Run the exact focused test filter and preserve assertion-based compiling RED at `/tmp/task22o24-red-vocabulary-adapter.txt`.
5. Replace the temporary shell with only `InternalVoid = 8`, its explicit non-bridge match arm, and the narrow mixed adapter. The adapter adds flat subject paths directly as `PathRole::Subject`, adds each clip ExPolygon contour then holes as `PathRole::Clip`, and reuses the existing `execute_two_pass(..., Intersection)`. Do not duplicate the two-pass algorithm or alter ordinary boolean behavior.
6. Rerun the byte-identical focused command GREEN and preserve `/tmp/task22o24-green-vocabulary-adapter.txt`. Run existing geometry, O17-O23 surface-kind, and Clipper regressions to prove no unrelated drift.

## Task 2 — Shared `polygonsInternal` void semantics, test-first

1. Before changing the shared helper, add O21 and O23 direct REDs with interleaved `Internal`, `InternalVoid`, and `InternalSolid` surfaces, including holes and metadata. Require collection order and contour-before-holes flattening, and require O23 closing/protection to consume the same path sequence.
2. Preserve a focused RED at `/tmp/task22o24-red-shared-internal-void.txt`.
3. Extend only `vertical_shell_trimming::trim::polygons_internal` to select `InternalVoid` between the source membership alternatives without changing collection order. Do not add a producer or modify O18 retag behavior.
4. Rerun the byte-identical focused command GREEN and preserve `/tmp/task22o24-green-shared-internal-void.txt`. Reassert all frozen O21-O23 KSR checksums/totals/events; the real fixture remains unchanged because its void count is zero.

## Task 3 — O24 state/API shell and direct record REDs

Production files and budgets:

- new `crates/ares-core/src/project_slice/prepare_infill/vertical_shell_assignment.rs`, at most 180 LOC;
- new children `vertical_shell_assignment/{types,assign,stage,cleanup}.rs`, each at most 300 LOC;
- direct test root `vertical_shell_assignment/tests.rs` plus ordinary shards `tests/{geometry,assignment,transaction}.rs`, each at most 300 LOC.

Steps:

1. Define `PreparedPostVerticalShellAssignment` with the exact same retained fields as O23: boxed predecessor, objects, caches, projections, trims, regularizations, and filters. It adds no persistent assignment sidecar.
2. Define transient staged record data containing `new_internal`, `new_internal_void`, and `new_internal_solid`, plus an explicit no-op representation for empty O23 filters. Define an ordered test event vocabulary: `SolidIntersection`, `InternalDifference`, `InternalVoidDifference`.
3. Add minimal compiling record function signatures. Do not wire public slicing or commit mutations yet.
4. Add direct behavioral REDs for:
   - empty filter: zero events and no staged replacement;
   - exact event order for nonempty filter, including empty `polygonsInternal`, Internal, or InternalVoid subjects;
   - solid intersection over flat `polygonsInternal` using the O23 filtered shell as clip;
   - Internal and InternalVoid differences selected independently from original collection order;
   - disjoint, partial, full, multi-component, holed, and nested-island cases;
   - all three test-injected failures with exact operation prefixes and the stable O24 error.
5. Preserve focused RED at `/tmp/task22o24-red-direct-record.txt`.
6. Implement source-order record staging in `assign.rs`:
   - return no-op immediately for an empty filter;
   - compute `polygonsInternal` through the shared helper;
   - collect current Internal ExPolygons in collection order;
   - collect current InternalVoid ExPolygons identically;
   - run mixed solid intersection, Internal difference, then InternalVoid difference, even for empty subjects;
   - map each Clipper failure to `SliceError::InvalidInput("vertical-shell fill-surface assignment geometry is outside the supported Clipper range")`.
7. Rerun the byte-identical focused command GREEN and preserve `/tmp/task22o24-green-direct-record.txt`.

## Task 4 — Exact assignment commit RED/GREEN

1. Add direct assignment REDs using a collection that interleaves retained externals and all representable internal kinds with distinct geometry and metadata. Require:
   - the original stable Top/Bottom/BottomBridge subsequence, not grouping by membership-list order;
   - removal of every old Internal/InternalVoid/InternalSolid value;
   - appended groups in exact Internal, InternalVoid, InternalSolid order and each group's staged Clipper order;
   - default metadata bits `(-1.0, 1, -1.0, 0)` on every rebuilt surface;
   - exact retained external metadata and inner geometry allocation identity;
   - fresh/nonaliasing rebuilt geometry relative to original fill geometry and O23 clip geometry;
   - no mutation of `slices`, perimeters, thin fills, `fill_expolygons`, or `fill_no_overlap_expolygons`.
2. Add an empty-filter no-op witness that freezes the entire record, `fill_surfaces` vector pointer, every inner allocation, and metadata.
3. Preserve RED at `/tmp/task22o24-red-direct-assignment.txt`.
4. Implement commit only for fully staged success. For active records, use stable in-place retention of Top/Bottom/BottomBridge and append moved staged geometry through `RegionSurface::new` in exact group order. Do not clone retained externals, inherit old internal metadata, sort, union, canonicalize, or deduplicate.
5. Rerun the byte-identical focused command GREEN and preserve `/tmp/task22o24-green-direct-assignment.txt`.

## Task 5 — Whole-project alignment and stage-before-move

Integration files and budgets:

- new root `crates/ares-core/src/project_slice/tests/prepare_infill/vertical_shell_assignment.rs`;
- ordinary shards `fixture.rs`, `transaction.rs`, `transaction/{failures,mismatches}.rs`, `ownership.rs`, `ownership/snapshots.rs`, `cleanup.rs`, `lifecycle.rs`, `lifecycle/precedence.rs`, `options.rs`, `metamorphic.rs`, and `ksr.rs`, each at most 300 LOC;
- declare the integration root from `project_slice/tests/prepare_infill.rs`.

Steps:

1. Reuse the real O23 preparation fixture; do not embed fixture bytes or read reference G-code.
2. Before implementing project stage/cleanup, add compiling REDs for every complete O23 alignment relation, genuine later active-slot and later-object failures after earlier staged successes, no early mutation, exact O23 ownership retention, intentional active fill-surface mutation, empty-record identity, success disposal, failure rollback, and constrained-stack cleanup.
3. Run `cargo nextest run -p ares-core -E 'test(/task22o24_.*(alignment|transaction|rollback|ownership|cleanup)/)'` and preserve `/tmp/task22o24-red-integration-transaction.txt`.
4. Implement complete pre-event validation in `stage.rs`: all O18-O23 outer lengths, records, plan layers, inputs, prelude records, `Some`/`None` slots, source/transform identity, planned index/layer IDs, current layer/region, region ID, compatible region IDs, one-region envelope, and scale relations.
5. Stage all records while borrowing O23 in stable source order. Only after the entire project succeeds destructure/move O23, commit every staged record, and construct `PreparedPostVerticalShellAssignment` with exact outer allocation identity.
6. On failure, drop transient staged geometry and delegate exact iterative O23 disposal. On successor disposal, reconstruct exact O23 with mutated objects and delegate to O23 cleanup.
7. Rerun the byte-identical Task 5 command GREEN. Require no successor on error, predecessor drop probes, no early record mutation, and exact operation prefixes. Enumerate constrained-stack evidence for each of the two independent 10,000-node predecessor families across all five required paths: direct O24 success disposal, injected solid-intersection failure, injected Internal-difference failure, injected InternalVoid-difference failure, and public-incomplete O24 disposal. Use only the shared Unix/non-Windows 64 KiB and Windows 256 KiB baseline; do not weaken node counts or iterative-cleanup assertions. Preserve `/tmp/task22o24-green-integration-transaction.txt`.

## Task 6 — Public lifecycle and error precedence

1. Before wiring production, add compiling lifecycle REDs proving intended O17→O24 order, exactly one O24 invocation, and zero O24 invocations for every earlier capability/O17/O19/O20/O21/O22/O23 failure. Preserve exact predecessor errors.
2. Run the exact lifecycle filter and preserve `/tmp/task22o24-red-integration-lifecycle.txt`.
3. Declare `vertical_shell_assignment` from `prepare_infill.rs`.
4. In `project_slice.rs`, invoke O24 exactly once after successful O23, replace the production O23 terminal consumer with an O24 consumer, dispose O24, and continue returning `ProjectSlicingIncomplete`. Retain the O23 consumer under `#[cfg(test)]` for focused regressions.
5. Rerun the byte-identical lifecycle command GREEN and preserve `/tmp/task22o24-green-integration-lifecycle.txt`.

## Task 7 — Typed provenance and metamorphic tests

1. Prove active/inactive `ensure_vertical_shell_thickness` behavior comes only from typed 3MF predecessor state. Inactive modes retain complete records and invoke no O24 geometry.
2. Preserve model-part option precedence, printable-area Normal/LargeBed scale selection, and component-transform scaling. O24 itself introduces no numeric option, but its geometry must respond only through typed predecessor coordinates and O23 filters.
3. Add ZIP reverse-order/Stored/Unix/timestamp repacks and a non-slicing rename witness. Results must be invariant.
4. Add direct synthetic InternalVoid behavior while explicitly proving the real KSR producer count is zero. Production must never branch on that count.

## Task 8 — Freeze parent-bound KSR evidence

1. Parse `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` independently twice through normal project APIs.
2. Before O24 evidence, reassert the exact O23 checksum `-41564956609250807593946297629749369320`, totals `[1,460,0,460,632,554,78,554,128,33815]`, threshold digest `-167664109034474951983490568976349754300`, ordered events `[259,259,259,632,66,80,80,259]`, and all retained O19-O22 parent evidence.
3. Define delimited O24 digest functions over object/slot order, active/no-op records, pre/post kinds, metadata bits, ExPolygons, contours, holes, points, coordinates, and exact three-event order.
4. Freeze only after two independent captures agree:
   - O24 parent-bound geometry+metadata checksum;
   - totals covering objects, slots, active/no-op, pre/post surfaces by kind, ExPolygons, contours, holes, and points;
   - ordered event totals `[SolidIntersection, InternalDifference, InternalVoidDifference]`;
   - real KSR produced and final InternalVoid counts, both honestly zero unless actual runtime evidence differs.
5. Keep literals only in tests/docs. Never read reference G-code.

## Task 9 — Honest compiling mutation evidence

After clean GREEN, preserve byte-exact production backups under `/tmp`, apply one compiling mutation at a time, run the exact intended focused command, preserve RED output, and restore byte-for-byte before continuing. These are post-implementation mutation REDs, not chronological pre-implementation RED.

Required mutations:

1. change `InternalVoid` discriminant or classify it as a bridge;
2. wrap each flat subject path as a standalone ExPolygon while forcibly normalizing it to contour winding, changing mixed-winding hole semantics;
3. replace the solid intersection operation with difference; retain role-only operand reversal as an explicitly equivalent commutative control, not a required RED;
4. skip or reorder the InternalVoid difference;
5. replace NonZero/two-pass output with EvenOdd, flat Paths, safety offset, or pre-union;
6. mutate/retain the collection before all three booleans read it;
7. replace stable external retention with grouping or unstable order;
8. reorder any appended category;
9. inherit old internal metadata instead of source defaults;
10. rebuild a record whose O23 filter is empty;
11. bypass O24 public wiring;
12. remove pre-geometry alignment validation;
13. truncate staging before a genuine later active slot.

Every mutation must fail its intended behavioral witness rather than compilation or an unrelated test. After restoration, compare production files byte-for-byte, rerun affected focused GREEN, and then rerun the full workspace gate.

## Task 10 — Full local gates and audits

1. Run exact direct O24 and exact O24 integration filters.
2. Run explicit O10-O24 regression filters.
3. Run `cargo nextest run --workspace --no-fail-fast`.
4. Run:
   - `cargo check --workspace --all-targets`;
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
   - `cargo check -p ares-core --target wasm32-unknown-unknown`;
   - `cargo check -p ares-core --target wasm32-unknown-unknown --features task22n-browser-oracle`;
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`;
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown --features task22n-browser-oracle`;
   - optimized default/feature browser-WASM build, wasm-bindgen/export audit, and both Playwright runs exactly as `.github/workflows/tier1.yml`;
   - `cargo fmt --all -- --check` and `git diff --check`.
5. Audit every Rust file `<400 LOC`, every new O24 shard `<=300 LOC`, no dependency diff, and no staged `.pi-subagents/` or `target/parity/` artifact.
6. Audit added Rust for no `unsafe`, `include!`, `include_bytes!`, broad lint allowance, binary oracle payload, reference-G-code access, fixture identity/hash/layer/geometry branch, Orca command/FFI, or fallback.
7. Manually audit pinned commits and the documented source boundary. Tests must not read, parse, hash, grep, or line-pin Orca/Ares source text.
8. Mechanically audit rollback: restore O23 terminal consumption; remove only O24 module/state/wiring/tests/docs, mixed adapter/export/tests, InternalVoid representation/exhaustive updates, and shared helper void selection; retain all O23 behavior and ordinary existing geometry APIs.

## Task 11 — Six-dimensional review loop, docs, commit, push, and exact-SHA CI

1. Update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec, and this plan with the exact O24 seam, InternalVoid decision, operation/order/ownership rules, frozen checksum/totals/events, test totals, and next source boundary after completed `PrintObject::discover_vertical_shells`.
2. Rerun both spec reviewers and both plan reviewers after evidence edits. Any doc change repeats the appropriate complete gate.
3. Dispatch one fresh independent review-only thread over requirements completeness, logic correctness, edge cases, code quality, test coverage, and actual execution. Dispatch a separate default-model OpenCode review over the same final diff and evidence. Require the exact implementation verdict format from the workflow.
4. The parent remains sole writer. Fix every blocking finding, search for siblings, rerun affected and full gates, and return the identical revised diff/evidence to both reviewers. Repeat until both return literal `VERDICT: APPROVE`.
5. Use Conventional Commits, keeping implementation and final evidence/docs commits separate where practical. Do not stage `.pi-subagents/` or `target/parity/`.
6. Push `main`, verify clean `HEAD == origin/main`, then require the exact pushed commit's complete Tier-1 native matrix and optimized browser-WASM/Playwright job to pass. Any CI repair repeats affected tests, audits, reviews, commit, push, and exact-SHA verification.

## Rollback

Rollback restores O23 as the production terminal consumer; removes O24 module/state/wiring/tests/docs, the mixed intersection adapter/export/tests, `InternalVoid` representation/exhaustive updates, and shared O21 helper void selection; and leaves all O23 filtering, predecessor state, existing geometry APIs, dependencies, persisted formats, and public API unchanged.

## Stop condition

Stop O24 only when `PrintObject.cpp:2402-2432` is transactionally derived from typed aligned O23 state with exact three-call source order, mixed topology, stable external retention, append order, default metadata, InternalVoid discriminant/non-bridge semantics, and empty-filter no-op; O23 ownership and deep iterative cleanup are proven; KSR evidence is repeatable and parent-bound; public slicing reaches O24 exactly once while remaining incomplete after completed vertical-shell assignment; all local and exact-commit Tier-1 gates pass; both final reviewers approve; docs are synchronized; and commits are pushed.

## Final evidence record

The final KSR tuple is checksum
`-117597382518472843802490205604634875775`, pre/post kind totals
`[113, 6, 48, 1127, 0, 0]` / `[113, 6, 48, 1281, 575, 0]`, and pre/post
geometry totals `[1294, 168, 46011]` / `[2023, 270, 73848]`. Record accounting
is 460 total, 161 active, 299 no-op, 299 unchanged, and 299 unchanged no-op.
Object/slot positions, record/surface boundaries, path counts, contour/hole
role and index, point counts, and end markers delimit the stream. The delimited
record digest is `-65994586923856785425316699963519338136`;
the exact event digest is `-110138798119262824097709645699717637653`
with `[161, 161, 161]` ordered calls. InternalVoid remains honestly `[0, 0]`
in real KSR while synthetic direct and O23 closing/protection tests freeze void
topology and ordering.

The final focused inventory has 31 passing tests: nine production-adjacent
assignment/adapter tests, three shared vocabulary/helper/filter tests, and 19
project lifecycle, transaction, ownership, cleanup, provenance, metamorphic,
and KSR tests. The full release evidence includes 149 O21-O24 regressions and
5,827 workspace tests passed with 2 skipped, all 13 planned mutations plus the retained-scale review mutation and restoration, native/WASM/browser gates, both
final review paths, and exact-SHA Tier-1. The role-only intersection reversal
is recorded as an equivalent commutative control; intersection-to-difference is
the killed behavioral operation mutation.

After O24, the planned source rewrite proceeds to
`PrintObject::prepare_infill` line 618 and
`PrintObject::discover_horizontal_shells` at `PrintObject.cpp:3955-4161`.
