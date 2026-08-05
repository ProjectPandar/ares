# Task 22O.23 — Single-region vertical-shell tiny-island filtering Plan

Spec: `docs/superpowers/specs/2026-08-05-ksr-fdmtest-v4-task22o23-vertical-shell-tiny-island-filtering.md`

## Status

Executed from Ares baseline `9caa7dd000e55165765c381d942c1283c14be216` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. The final O23 spec received literal approval from an independent reviewer at `/tmp/task22o23-spec-independent-final-review.md` and a separate default-model OpenCode reviewer at `/tmp/task22o23-spec-opencode-final-review.txt`; the plan received the same two approvals before production work. Implementation is at the final review, commit, push, and exact-SHA Tier-1 release gates.

## Validation contract

Port only pinned `PrintObject::discover_vertical_shells` behavior at `PrintObject.cpp:2369-2400`: previous/next object-volume intersection, flat Miter closing of current internal Paths, exact mixed `f32`/`f64` tiny-area thresholds, the source short-circuit predicate and literal path-count protection heuristic, stable survivor ordering, and the empty gate. Stop before line 2402. Preserve exact O22 ownership beside a fresh aligned crate-private filtering sidecar. Do not add `intersection_ex`, `fill_surfaces` mutation, `InternalVoid`, new options, dependencies, public API, fallback, or an Ares-owned pipeline.

## Gate 0 — Baseline and reviewed design

1. Confirm Ares `HEAD == origin/main == 9caa7dd000e55165765c381d942c1283c14be216`, Orca checkout `8500fcdccaa10b5099ac20d252af3a7c560046f1`, and no tracked worktree changes except the reviewed O23 spec/plan.
2. Record the exact-commit O22 Tier-1 result for GitHub Actions run `30963012084`; any native or browser-WASM failure must be repaired and revalidated before O23 ships.
3. Preserve the final spec approvals named above.
4. Send this complete plan through one fresh independent review-only agent and one separate default-model OpenCode review. Any plan edit returns the complete revised plan to both. Require literal `VERDICT: APPROVE` from both before production edits.

## Task 1 — Add the O23 state/API shells and direct RED seams

Production files and budgets:

- new `crates/ares-core/src/project_slice/prepare_infill/vertical_shell_filtering.rs`, at most 180 LOC;
- new children `vertical_shell_filtering/types.rs`, `filter.rs`, `stage.rs`, and `cleanup.rs`, each at most 300 LOC;
- direct test root `vertical_shell_filtering/tests.rs` and real shards `constants.rs`, `topology.rs`, `predicate.rs`, and `transaction.rs`, each at most 300 LOC;
- declare the production module from `project_slice/prepare_infill.rs` only after the first compiling API shell exists.

Steps:

1. Define `VerticalShellTinyFilter { filtered_shell: Vec<ExPolygon> }`, an aligned per-object `Vec<Option<_>>`, and `PreparedPostVerticalShellFiltering` retaining the exact O22 predecessor fields plus fresh filters. Follow the existing O19-O22 flattened compatibility-state pattern; do not box or redesign the predecessor graph.
2. Define an ordered test-observable event vocabulary: neighbor intersection, closing grow, closing shrink, candidate scan, visibility difference, candidate expansion, protection difference, and empty gate. Test-only failure injection is accepted only at the six Clipper call sites; candidate scan and empty gate are trace-only.
3. Add minimal compiling function signatures for threshold derivation and record filtering. Initial behavior may return a deliberately incomplete placeholder only long enough to create behavior RED; it must not enter public slicing.
4. Add the first vertical RED for exact constants. Freeze:
   - shared O22 minimum bits;
   - integer `scaled(1.5)` and `scaled(8.0)` values and `f32` bits;
   - both `f32` product bits;
   - pre-cast `(1e-4_f64 / scale.factor())` `f64` bits;
   - final epsilon `f32` bits for Normal and LargeBed.
   The Normal pre-cast quotient must fail an implementation that truncates through `i64`, while the final `f32` witness records the actual call argument.
5. Run `cargo nextest run -p ares-core -E 'test(/task22o23_.*constant/)'` and preserve its compiling behavioral RED at `/tmp/task22o23-red-direct-constants.txt`. Do not implement the constants GREEN until Task 2 exposes the shared O22 minimum; the Task 1 API shell must not duplicate the one-cast expression.

## Task 2 — Exact shared helper visibility and no O21/O22 drift

1. Add a narrowly restricted O22 accessor under `crate::project_slice::prepare_infill` that delegates to the existing `min_perimeter_infill_spacing`; do not duplicate or alter the one-cast `f32` implementation.
2. Widen `vertical_shell_trimming::trim::polygons_internal` only to `crate::project_slice::prepare_infill` so O23 reuses exact surface selection and collection/contour/hole ordering.
3. Run the existing O21 and O22 direct/integration filters, frozen checksums, totals, events, and radii digest before and after these visibility-only changes. No O21/O22 output may change.
4. Implement the Task 1 constant helper only after the restricted accessor exists. It must call that accessor rather than duplicate the spacing-to-minimum expression. Rerun the byte-identical Task 1 Nextest command GREEN and preserve `/tmp/task22o23-green-direct-constants.txt`.

## Task 3 — RED/GREEN source volume construction

1. Add direct topology REDs using literal geometry for first, middle, and last layers; lower-as-subject/upper-as-clip; disjoint, partial, full, multi-component, and holed neighbors; contour-before-hole flattening; and lookup independent of adjacent slot occupancy.
2. Add flat closing REDs for epsilon-close and wider gaps, holes, mixed winding, Normal/LargeBed scales, exact Miter-3 output path/point order, and exact path count. Explicitly distinguish flat Paths from ExPolygon/PolyTree grouping and Miter from Square/Round.
3. Implement source volume construction in `filter.rs`:
   - O21 empty trim returns an empty sidecar with zero events;
   - flatten optional previous/next retained `lslices` contour then holes;
   - always call `intersection_polygons_paths(previous, next)` for a nonempty trim;
   - reuse the O21 current `polygons_internal` helper;
   - compute epsilon in `f64`, cast directly to `f32`, and call `offset_paths(+epsilon, Miter, 3.0)` then `offset_paths(-epsilon, Miter, 3.0)`.
4. Map every Clipper error to `SliceError::InvalidInput("vertical-shell tiny-island filtering geometry is outside the supported Clipper range")`.
5. Preserve RED at `/tmp/task22o23-red-direct-volumes.txt`, then rerun the byte-identical command GREEN.

## Task 4 — RED/GREEN exact candidate predicate

1. Add candidate-area REDs immediately below, exactly equal to, and immediately above both rounded thresholds. Include ordinary, odd, and supported spacing above `f32` exact-integer range, plus a contour with a hole. Freeze signed `f64` area and strict `<` behavior.
2. Add exact short-circuit REDs:
   - below 1.5: candidate scan, no visibility difference, expansion, protection difference;
   - between 1.5 and 8 and hidden: visibility, expansion, protection;
   - between thresholds and visible: visibility only;
   - at/above 8: no candidate geometry;
   - nonempty trim with empty O22 morphology: volumes plus empty gate but no candidate scan.
3. Add literal visibility topology for fully wrapped, partially wrapped, disjoint, and holed candidates.
4. Add path-count protection REDs where subtraction reduces count, preserves count, or splits/increases count, including multiple components and holes. Assertions must fail substitutions based on emptiness, area, equality, or containment.
5. Add interleaved removed/retained candidates. Freeze stable survivor ExPolygon/contour/hole/point order and require fresh nonaliasing survivor storage while every O22 allocation remains unchanged.
6. Implement the predicate in source evaluation order:
   - reuse O22 minimum;
   - derive scaled area constants through truncating `i64`, cast to `f32`, multiply in `f32`, promote only the products for `f64` comparison;
   - lazily flatten candidate Paths at each source call site;
   - run object difference only on the second visibility branch;
   - run Miter-3 expansion and protection difference only when the first clause is true;
   - remove only when `difference.len() >= internal_volume.len()`;
   - deep-clone survivors without sorting, union, canonicalization, or deduplication;
   - record the final empty gate.
7. Preserve RED at `/tmp/task22o23-red-direct-predicate.txt`, then rerun the byte-identical command GREEN.

## Task 5 — Whole-project alignment and stage-before-move

1. Create the integration test root plus the transaction, ownership, mismatch, snapshot, and cleanup shards listed in Task 8. Before implementing stage or cleanup behavior, add compiling REDs for every alignment relation, a genuine later active slot and later object failure after earlier successful filtering, exact O22 ownership retention, fresh O23 ownership, failure rollback, and success disposal. The REDs must observe missing/incorrect behavior rather than compilation. Run `cargo nextest run -p ares-core -E 'test(/task22o23_.*(alignment|transaction|rollback|ownership|cleanup)/)'` and preserve `/tmp/task22o23-red-integration-transaction.txt`.
2. Implement `stage.rs` validation before any O23 event:
   - outer object counts across O18 objects, O19 caches, O20 projections, O21 trims, O22 regularizations, and traversal objects;
   - record counts across every sidecar, inputs, prelude records, plan layers, and retained `lslices`;
   - complete `Some`/`None` alignment;
   - source/transform identity, planned index, layer ID, current layer/region, region ID, compatible region IDs, one-region envelope, and retained project scale.
3. Iterate objects and records in stable source order. Use `lslices[index - 1]` and `lslices[index + 1]` by current layer index, independent of neighboring O18-O23 slot occupancy.
4. Stage every fresh filter while borrowing O22. Only after all objects/slots succeed destructure and move exact O22 fields into `PreparedPostVerticalShellFiltering`.
5. Implement cleanup only after the cleanup RED exists. On any failure, expose no successor and delegate exact O22 disposal. On success disposal, drain fresh O23 geometry first, reconstruct exact O22, and delegate to O22 cleanup.
6. Rerun the byte-identical Task 5 command GREEN. Require exact operation prefix, predecessor drop probe, no partial successor, whole-project rollback, and exact/fresh ownership witnesses; preserve `/tmp/task22o23-green-integration-transaction.txt`.

## Task 6 — Fault boundaries, ownership, and constrained-stack cleanup

1. Before extending fault handling, add compiling REDs for malformed-coordinate paths, all six injected failure prefixes, complete allocation identity/freshness, and both depth-10,000 cleanup families. Preserve the focused RED at `/tmp/task22o23-red-integration-fault-cleanup.txt`.
2. Use genuinely malformed coordinates for integrated operations that can independently receive invalid staged input. Do not manufacture a claim that the final protection difference can naturally be the first failing integrated operation after both operands passed prior Clipper validation.
3. Implement test-only failure injection at all six O23 Clipper call sites. For each, freeze stable error text, exact event prefix, no later events, no successor, and predecessor disposal.
4. Snapshot all retained O22 allocations/content, including both predecessor tree families, O18 surfaces, O19 caches, O20 projections, O21 trims, and O22 ExPolygon/path/point buffers. Require exact identity after success and fresh/nonaliasing O23 object/record/vector/ExPolygon/contour/hole/point buffers.
5. Make depth-10,000 tests pass for both classic predecessor tree families on direct O23 success, every injected failure class, and public-incomplete disposal. Use only the shared `CONSTRAINED_TEST_STACK_SIZE`: Unix/non-Windows 64 KiB, Windows 256 KiB. Do not weaken node counts or iterative-cleanup assertions. Rerun the byte-identical focused command GREEN and preserve `/tmp/task22o23-green-integration-fault-cleanup.txt`.

## Task 7 — Wire public slicing and preserve error precedence

1. Before production wiring, add compiling lifecycle and error-precedence REDs proving intended O17→O23 order, exactly one O23 invocation, and zero O23 invocations for every earlier failure. Run `cargo nextest run -p ares-core -E 'test(/task22o23_.*(lifecycle|precedence)/)'` and preserve `/tmp/task22o23-red-integration-lifecycle.txt`.
2. Declare `vertical_shell_filtering` from `prepare_infill.rs`.
3. In `project_slice.rs`, invoke O23 exactly once after successful O22, replace the production O22 terminal consumer with an O23 consumer, dispose O23, and continue returning `ProjectSlicingIncomplete`.
4. Retain test-only O22 and earlier consumers for focused regressions.
5. Rerun the byte-identical Task 7 command GREEN and preserve `/tmp/task22o23-green-integration-lifecycle.txt`. Spiral, counterbore, multi-region, interface-shell, active extra-bridge, O17 geometry, O19, O20, O21, and O22 failures must retain their exact errors and invoke O23 zero times.

## Task 8 — Real 3MF provenance and metamorphic tests

Integration files and budgets:

- new root `project_slice/tests/prepare_infill/vertical_shell_filtering.rs`;
- real shards `fixture.rs`, `ksr.rs`, `options.rs`, `metamorphic.rs`, `ownership.rs`, `ownership/mismatches.rs`, `ownership/snapshots.rs`, `transaction.rs`, `lifecycle.rs`, and `cleanup.rs`, each at most 300 LOC;
- declare the root from `project_slice/tests/prepare_infill.rs`.

Steps:

1. Reuse the real O22 fixture preparation path; do not add embedded fixture bytes or reference-G-code access.
2. Add typed solid-infill-flow mutations proving resolved 3MF inputs → `ClassicPreludeRecord::solid_infill_spacing` → exact minimum/threshold/expansion bits → survivor change, including model-part precedence.
3. Add typed printable-area mutation across the large-bed threshold proving retained scale, pre-cast epsilon quotient, final epsilon, scaled constants/coordinates, and physically corresponding result.
4. Add ZIP reverse-order/Stored/Unix/timestamp repacks, non-slicing rename, active/inactive ensure mode, adjacent aligned `None`, and component-transform scaling witnesses. Production must remain independent of archive name/hash/dimensions/layer count/geometry identity.

## Task 9 — Freeze parent-bound KSR evidence

1. Parse `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` independently twice through normal project APIs.
2. Before O23 evidence, reassert O19-O22 parent checksums/totals/events and O22 radii digest exactly as frozen in the spec.
3. Define delimited O23 digest functions over exact object/slot/`None`/`Some` order, input and survivor ExPolygons, contours, holes, points, coordinates, removed counts, exact threshold values, and ordered events.
4. Freeze only after two independent captures agree:
   - O23 parent-bound checksum;
   - totals `[objects, slots, none, some, input_expolygons, survivor_expolygons, removed_expolygons, contours, holes, points]`;
   - threshold digest;
   - eight ordered event totals.
5. Keep literals only in tests/docs. Never read the reference G-code.

## Task 10 — Honest compiling mutation evidence

After clean GREEN, preserve byte-exact production backups under `/tmp`, apply one compiling mutation at a time, run the exact focused command, preserve RED output, and restore byte-for-byte before continuing. These are post-implementation mutation REDs, not chronological pre-implementation RED.

Required mutations:

1. route epsilon through truncating `i64` before `f32`;
2. change either strict `<` to `<=`;
3. multiply either area threshold in `f64`;
4. replace previous/next intersection with union or current-layer geometry;
5. reorder contour and holes;
6. run visibility difference unconditionally;
7. replace the literal path-count comparison with emptiness/equality/area/containment;
8. bypass O23 public wiring;
9. remove pre-geometry alignment validation;
10. truncate staging before a genuine later active slot.

Every mutation must fail its intended behavioral witness rather than compilation or an unrelated test. After restoration, rerun the exact command GREEN and compare the restored source byte-for-byte.

## Task 11 — Full local gates and audits

1. Run the exact direct O23 filter and exact O23 integration filter.
2. Run explicit O10-O23 regression filters.
3. Run `cargo nextest run --workspace --no-fail-fast`.
4. Run:
   - `cargo check --workspace --all-targets`;
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
   - `cargo check -p ares-core --target wasm32-unknown-unknown`;
   - `cargo check -p ares-core --target wasm32-unknown-unknown --features task22n-browser-oracle`;
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`;
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown --features task22n-browser-oracle`;
   - optimized browser-WASM build/bindgen and both Playwright suites used by `.github/workflows/tier1.yml`;
   - `cargo fmt --all -- --check` and `git diff --check`.
5. Audit every Rust file `<400 LOC`, every new O23 shard `<=300 LOC`, no dependency diff, and no staged `.pi-subagents/` or `target/parity/` artifact.
6. Audit added Rust for no `unsafe`, `include!`, `include_bytes!`, broad lint allowance, binary oracle payload, reference-G-code access, fixture identity/hash/layer/geometry branch, Orca command/FFI, or fallback.
7. Manually audit pinned commits and the documented source boundary. Tests must not read, parse, hash, grep, or line-pin Orca/Ares source text.
8. Mechanically audit rollback: restore O22 terminal consumption; remove only O23 module/state/wiring/tests/docs and restricted helper visibility changes; retain all O22 behavior and ordinary geometry APIs.

## Task 12 — Six-dimensional review loop, docs, commit, push, and exact-SHA CI

1. Update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec, and this plan with the exact O23 seam, arithmetic/order/ownership decisions, frozen checksum/totals/threshold/events, test totals, and next boundary at `PrintObject.cpp:2402`.
2. Rerun both spec reviewers and both plan reviewers after evidence edits. Any doc change repeats the appropriate gate.
3. Dispatch one fresh independent review-only thread over requirements completeness, logic correctness, edge cases, code quality, test coverage, and actual execution. Dispatch a separate default-model OpenCode review over the same final diff and evidence. Require the exact implementation verdict format from the workflow.
4. The parent remains sole writer. Fix every blocking finding, search for sibling occurrences, rerun affected and full gates, and return the identical revised diff/evidence to both reviewers. Repeat until both return literal `VERDICT: APPROVE`.
5. Use Conventional Commits, keeping implementation and final evidence/docs commits small where practical. Do not stage `.pi-subagents/` or `target/parity/`.
6. Push `main`, verify clean `HEAD == origin/main`, then require the exact pushed commit's complete Tier-1 native matrix and optimized browser-WASM/Playwright job to pass. Any CI repair repeats affected tests, audits, reviews, commit, push, and exact-SHA verification.

## Execution evidence

The implementation freezes KSR checksum
`-41564956609250807593946297629749369320`, totals
`[1, 460, 0, 460, 632, 554, 78, 554, 128, 33815]`, threshold digest
`-167664109034474951983490568976349754300`, and ordered events
`[259, 259, 259, 632, 66, 80, 80, 259]`. Its LargeBed witness retains
truncating `scaled(8.0) = 799999`.

Final local runs pass 18 direct and 29 integration O23 tests, 393 O10-O23
regressions, and 5,797 workspace tests with 2 skipped. Workspace native check,
strict Clippy, four WASM checks, optimized default/feature browser builds and
export audit, and two 9-test Playwright suites pass. Ten required compiling
behavioral mutations are killed by their intended witnesses, after which final
production files compare byte-for-byte with their backups and affected/full
GREEN gates pass. Formatting, diff, dependency, forbidden-pattern, staging,
rollback, and LOC audits pass; all Rust files are below 400 LOC and the largest
O23 shard is 270 LOC. The next exact boundary is `PrintObject.cpp:2402`.

## Rollback

Rollback restores O22 as the public terminal consumer, removes the O23 module/state/wiring/tests/docs, and restores the two helper visibility changes. It does not alter O22 morphology, the O21 internal flattening implementation, existing geometry APIs, dependencies, persisted formats, or any public API.

## Stop condition

Stop O23 only when lines `PrintObject.cpp:2369-2400` are transactionally derived from typed aligned O22 state with exact neighbor/closing/threshold/short-circuit/path-count/order semantics; O22 allocation identity and deep iterative cleanup are proven; KSR evidence is parent-bound and repeatable; public slicing reaches O23 exactly once while remaining incomplete before line 2402; all local and exact-commit Tier-1 gates pass; both final reviewers approve; docs are synchronized; and commits are pushed.
