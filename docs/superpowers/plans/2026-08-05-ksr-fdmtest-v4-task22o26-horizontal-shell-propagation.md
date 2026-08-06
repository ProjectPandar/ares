# Task 22O.26 — Horizontal-shell propagation Plan

Spec: `docs/superpowers/specs/2026-08-05-ksr-fdmtest-v4-task22o26-horizontal-shell-propagation.md`

## Status

Implemented from Ares O25 predecessor
`251b53bf101d8a3f72b96cf540ea4a80ef7cb917` against pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The complete spec and this plan
received literal pre-implementation approval from an independent reviewer and
a separate default-model OpenCode reviewer, and O25 exact-SHA Tier-1 run
`31028569875` is fully green. Tasks 1-12 are implemented with the frozen local
evidence below. Final independent and default-model OpenCode reviews approve;
commit/push and exact-SHA Tier-1 remain open.

## Validation contract

Rewrite only the executable post-O25 remainder of
`PrintObject::discover_horizontal_shells` at
`OrcaSlicer/src/libslic3r/PrintObject.cpp:3974-4150`. Preserve exact typed
options, region/layer/type order, count-or-thickness windows, interleaved serial
mutation visibility, `f32`/scaled-integer casts, Clipper path topology/order,
collection reconstruction, and metadata grouping. Use a temporary working
fill-surface graph for whole-project rollback and commit dirty records only.
Retain the exact O25 graph and iterative cleanup, add no durable sidecar or
public API, and keep public slicing incomplete. Debug SVG, cancellation, later
prepare-infill behavior, fill/toolpaths/G-code, fallback, fixture branches,
reference-G-code access, and Orca runtime/FFI remain excluded.

Use these reproducible focused commands for every named RED/GREEN checkpoint;
the test modules are fixed by this plan before implementation:

- opening: `cargo nextest run -p ares-core --lib -E 'test(/geometry::tests::clipper::offset::opening/)'`;
- surface template: `cargo nextest run -p ares-core --lib -E 'test(/project_slice::tests::region_slices::template/)'`;
- all O26 direct/integration tests:
  `cargo nextest run -p ares-core --lib -E 'test(/horizontal_shell_propagation/)'`;
- explicit recent regression:
  `cargo nextest run -p ares-core --lib -E 'test(/(vertical_shell_(trimming|regularization|filtering|assignment)|horizontal_shell_(promotion|propagation))/)'`.

Capture the exact command and exit code in each referenced `/tmp` evidence
file; do not replace an assertion RED with a compile failure.

## Gate 0 — Predecessor, source, and reviewed design

1. Verify `HEAD == origin/main ==
   251b53bf101d8a3f72b96cf540ea4a80ef7cb917`, pinned Orca
   `8500fcdccaa10b5099ac20d252af3a7c560046f1`, and no tracked baseline changes
   except O26 spec/plan drafts.
2. Require O25 exact-SHA Tier-1 run `31028569875` to complete successfully on
   format, macOS, Ubuntu, Windows, and optimized browser-WASM/export audit with
   both Playwright runs. Repair and re-review O25 first if it fails.
3. Send the complete spec to one fresh independent reviewer and one separate
   default-model OpenCode review. Apply every blocker as the parent sole writer,
   then repeat both until literal approval.
4. Send this complete plan through the same two independent review paths after
   spec approval. Any substantive spec/plan edit repeats the affected reviews.
5. Freeze the approved documents and exact predecessor before production REDs.

## Task 1 — Asymmetric flat-path opening, test-first

Files and budgets:

- add `crates/ares-core/src/geometry/clipper/offset/opening.rs`, at most 220 LOC;
- minimally modify `geometry/clipper/offset.rs`, `geometry/clipper.rs`, and
  `crates/ares-core/src/geometry.rs` for the narrow crate-private reexport;
- add `geometry/tests/clipper/offset/opening.rs`, at most 300 LOC, and declare it
  from the ordinary offset test module.

Steps:

1. Add a minimal compiling adapter shell with deliberately wrong runtime
   behavior, then assertion RED tests for empty, disjoint, repeated,
   contour/hole, and
   near-coordinate-limit paths. Freeze flat path output topology and order.
2. Add an interstage test observer that proves shrink by `delta1` precedes
   expand by `delta2`; freeze distinct asymmetric deltas, Miter join, limit
   `5.0`, and the existing shortest-edge configuration at both stages.
3. Add first-stage and second-stage coordinate-failure REDs with deterministic
   first-error order. Preserve RED evidence under `/tmp/task22o26-red-opening`.
4. Implement the narrow adapter only as existing `offset_paths` shrink by
   `-delta1` then existing `offset_paths` expand by `+delta2`, both returning
   flat paths. Reuse the shared safety-offset value for the later `+10.0_f32`;
   do not use `opening_ex`, symmetric deltas, PolyTree output, or copied offset
   logic.
5. Rerun the byte-identical focused command GREEN and preserve evidence under
   `/tmp/task22o26-green-opening`.

## Task 2 — Exact surface-template seam, test-first

Files and budgets:

- minimally modify `crates/ares-core/src/project_slice/region_slices.rs`;
- extend ordinary tests under `project_slice/region_slices/tests/`, splitting
  before any Rust file reaches 400 LOC.

Steps:

1. Add a minimal compiling template shell, then assertion REDs that replace one
   surface ExPolygon while freezing kind, thickness,
   thickness_layers, bridge_angle, and extra_perimeters.
2. Implement only a crate-private template-with-new-geometry constructor or
   clone operation. Do not add a collection abstraction or public API.
3. Kill a default-metadata reconstruction mutation, then rerun focused GREEN.

## Task 3 — O26 successor, working graph, hooks, and cleanup shell

Production files and budgets:

- add `prepare_infill/horizontal_shell_propagation.rs`, at most 180 LOC;
- add `horizontal_shell_propagation/{types,stage,cleanup}.rs`, each at most
  300 LOC;
- add direct test root and ordinary child shards, each at most 300 LOC.

Steps:

1. Declare the crate-private module from
   `project_slice/prepare_infill.rs` and add a minimal compiling successor,
   prepare, and dispose shell with deliberately incomplete runtime behavior.
   Do not wire the production project lifecycle yet.
2. Add assertion REDs for the exact successor field envelope, EnsureAll-only
   no-working-clone behavior, active-object aligned working fills, and exact
   delegated cleanup.
3. Define `PreparedPostHorizontalShellPropagation` with the exact O25 fields:
   boxed predecessor, objects, caches, projections, trims, regularizations, and
   filters. Add no durable candidate/geometry/branch sidecar.
4. Define transient aligned working object/record state containing cloned
   `fill_surfaces` and a dirty flag. Original slices remain borrowed immutable
   source data. For an EnsureAll-only object, allocate no working fill clones.
   For an object with any non-EnsureAll source, clone all present fill records
   because any one may become a neighbor target.
5. Add separate test-only prepare, ordered traversal/geometry, rebuild,
   dirty-commit, and disposal hooks. Hooks observe production operations and do
   not alter decisions or retained state.
6. Implement successor disposal by reconstructing exact O25 and delegating to
   O25 cleanup.
7. Rerun the fixed O26 focused command GREEN and preserve shell evidence under
   `/tmp/task22o26-green-state-shell`.

## Task 4 — Complete inherited alignment before work

Integration files and budgets:

- add `project_slice/tests/prepare_infill/horizontal_shell_propagation.rs`;
- add ordinary `fixture.rs`, `transaction.rs`, and
  `transaction/mismatches.rs`, splitting each at 300 LOC;
- declare the root from `project_slice/tests/prepare_infill.rs`.

Steps:

1. Reuse the real O25 prepared fixture; do not embed fixture bytes or inspect
   reference G-code.
2. Add one compiling RED for each inherited invariant: retained scale versus
   typed printable area; object/O18-O25 sidecar lengths; record/plan/input/
   prelude/lslice lengths; optional slots; source object/transform identity;
   planned array index/layer ID; current layer/region; region ID; and the single
   compatible-region constraint.
3. Require every mismatch to fail before the first O26 event, fill clone,
   width conversion, geometry call, rebuild, or commit.
4. Implement a complete immutable preflight. Only after it succeeds may the
   temporary working graph be created.
5. Preserve RED/GREEN evidence under
   `/tmp/task22o26-{red,green}-alignment`.

## Task 5 — Gates, gathering, and window helpers RED/GREEN

Production files and budgets:

- add `horizontal_shell_propagation/gather.rs`, at most 240 LOC;
- add `horizontal_shell_propagation/window.rs`, at most 220 LOC;
- add direct test shards `control_flow.rs`, `gather.rs`, and `windows.rs`, each
  at most 300 LOC.

Steps:

1. Add minimal compiling helper shells with deliberately wrong return values,
   then assertion REDs for source-kind order, options, flattening, and windows.
2. Freeze exact zero shell-count and empty-path gates. Trusted direct records
   with negative top and bottom counts plus positive thickness must still visit
   a neighbor, killing `<= 0` or unsigned saturation. Gather matching slices
   before working fill surfaces, each contour before holes, without union,
   cleanup, area filtering, sorting, or deduplication.
3. Add top/down and bottom/up neighbor REDs with variable heights,
   BottomBridge bottom options, nonconsecutive stored IDs, strict `1e-4_f64`,
   and count OR thickness short-circuit.
4. Define aligned `None` behavior: a None source has no source-kind work, while
   a None neighbor remains at its array distance and is represented by an empty
   fill collection for the ordered safety intersection and subsequent
   stop-kind/continue-neighbor decision. Add direct helper REDs for these cases.
5. Implement only the pure gate/gather/window decisions used later by the full
   serial loop; do not add a test-only callback or pre-gathered project state.
6. Rerun the fixed O26 focused command GREEN and preserve evidence under
   `/tmp/task22o26-{red,green}-gather-windows`.

## Task 6 — Ordered geometry and width filters RED/GREEN

Production files and budgets:

- add `horizontal_shell_propagation/geometry.rs`, at most 300 LOC;
- add direct test shards `intersection.rs`, `narrow.rs`, `repair.rs`, and
  `numeric.rs`, each at most 300 LOC.

Steps:

1. Add minimal compiling geometry shells with deliberately wrong outputs, then
   neighbor Internal/InternalSolid safety-intersection assertion REDs with flat
   contour/hole order and no pre-union.
2. Separate empty-intersection behavior for density zero, None, CriticalOnly,
   and positive-density Moderate. Assert stop-kind versus continue-neighbor
   events exactly.
3. Freeze factor branches `1.0_f32`, `0.5_f32`, `0.2_f32`, and `0.0_f32`.
4. Freeze flow ownership and casts: neighbor external flow for the first filter,
   current/source solid flow for the second; retained `f32` width divided by
   `f64` scale, truncated to `i64`, cast to `f32`, then multiplied. Cover Normal
   and LargeBed scales with a witness sensitive to cast reordering.
5. Freeze both Miter-5 asymmetric openings with `margin + 10.0_f32`, plain
   difference ordering, and first-filter assignment to both carried `solid` and
   `new_internal_solid`.
6. Freeze the second filter factor (`1.0_f32` for None, otherwise `3.0_f32`),
   flat NonZero positive expansion with the upstream default Miter join, miter
   limit `3.0`, and existing shortest-edge configuration; no-safety
   intersection; reachable Internal/InternalSolid/InternalVoid inclusion;
   external bridge exclusion; and the absence of a carried-solid update. Add
   an acute-corner/configuration witness distinguishing repair Miter-3 from the
   adjacent opening's Miter-5.
7. Add direct out-of-range REDs for neighbor external-flow scaling and
   current/source solid-flow scaling. Freeze each conversion's precedence before
   its opening and all later Clipper sites.
8. Map every scale/Clipper failure to exactly `horizontal-shell propagation
   geometry is outside the supported Clipper range` while preserving first
   serial failure.
9. Preserve focused RED/GREEN evidence under
   `/tmp/task22o26-{red,green}-geometry`.

## Task 7 — Source-faithful neighbor reconstruction RED/GREEN

Production files and budgets:

- add `horizontal_shell_propagation/rebuild.rs`, at most 300 LOC;
- add direct test shards `rebuild.rs`, `grouping.rs`, and `metadata.rs`, each at
  most 300 LOC.

Steps:

1. Add a minimal compiling rebuild shell with deliberately wrong runtime
   output, then assertion REDs for appending existing InternalSolid paths,
   NonZero `union_ex`,
   fresh default-metadata InternalSolid, safety-differenced original Internal,
   fresh default-metadata Internal, and accumulated internal clip paths.
2. Freeze dropping original InternalVoid and every non-Top/Bottom/BottomBridge
   value whenever rebuilding executes.
3. Add stable grouping REDs: process retained externals in original order; join
   the first compatible existing group; compare kind, thickness,
   thickness_layers, and bridge_angle; exclude extra_perimeters.
4. Freeze no-safety group difference, group/fragments output order, and complete
   first-member metadata templating. Surfaces differing only in
   extra_perimeters must merge and output must inherit the first value.
5. Include holes, disconnected results, repeated paths, empty group output, and
   geometry-equal rebuild. Mark dirty on block execution, not on value change.
6. Implement with the Task 2 template seam and no general one-use abstraction.
7. Rerun exact focused GREEN and preserve evidence under
   `/tmp/task22o26-{red,green}-rebuild`.

## Task 8 — Whole-project commit, rollback, ownership, and failure order

Files and budgets:

- add `horizontal_shell_propagation/propagate.rs`, at most 300 LOC;
- complete `horizontal_shell_propagation/stage.rs` and `geometry.rs` without
  exceeding 300 LOC; split a real child module if needed;
- add integration `ownership.rs`, `transaction/failures.rs`, and snapshots
  shards plus direct/integration `serial.rs`, each at most 300 LOC.

Steps:

1. Add a minimal compiling full-traversal shell that does not yet commit. Add
   the mandatory assertion RED where an earlier source/type rebuilds a later
   layer and that layer's later source gather observes the rebuilt working fill
   collection. Pre-gathered, reverse-order, independent-decision, and parallel
   implementations must fail it.
2. Add None-neighbor integration REDs proving the empty safety-intersection
   call/error order occurs at the original array distance before exact
   stop-kind/continue-neighbor control.
3. Add injected failure REDs at both out-of-range scaled-width conversions,
   safety intersection, both stages of both
   openings, each plain difference, expansion, repair intersection, union,
   Internal safety difference, and each external-group difference. Include a
   late failure after multiple successful working rebuilds. Freeze external
   flow scaling before first-filter Clipper calls and current solid flow scaling
   before second-filter calls.
4. Require every failure to expose no successor, commit zero original records,
   return the one stable O26 error, and dispose exact O25 once.
5. Implement the literal region/object/layer/type/neighbor loop using Tasks 5-7
   helpers against the complete temporary working graph while borrowing
   unmodified O25. Translate `goto EXTERNAL` as stopping only the current type
   scan; later reads must see earlier working mutations.
6. On complete success, destructure/move exact O25 and replace only dirty
   records' `fill_surfaces`. Clean records preserve vector/capacity and all
   geometry allocations; dirty records are fresh even if values compare equal.
7. Freeze predecessor, objects, records, unrelated buffers, perimeter/thin-fill/
   fill-boundary state, and all O19-O24 sidecar allocation/content/order.
8. Rerun the fixed O26 command GREEN, including the serial witness, and preserve
   rollback/ownership RED/GREEN evidence under
   `/tmp/task22o26-{red,green}-{transaction,ownership}`.

## Task 9 — Public wiring, lifecycle, and constrained-stack cleanup

Files and budgets:

- modify `project_slice.rs` only at the O25→O26 terminal seam;
- add integration `lifecycle.rs` and `cleanup.rs`, splitting before 300 LOC;
- adjust O25 terminal-helper tests only where required.

Steps:

1. With the already compiled O26 module still absent from the public call path,
   add lifecycle assertion REDs proving O26 one invocation after O25, zero O26 invocation
   for every earlier capability/O17/O19/O20/O21/O22/O23/O24/O25 error, and O26
   failure precedence over terminal incomplete.
2. Nest `horizontal_shell_propagation::prepare` exactly once after O25 and
   replace production O25 terminal disposal with O26 disposal. Retain the O25
   terminal consumer under `cfg(test)` only where predecessor regressions need
   it. Public success remains `ProjectSlicingIncomplete`.
3. Reuse the shared constrained-stack fixture and both independent 10,000-node
   predecessor families. Test direct success, injected late geometry failure,
   and public-incomplete disposal at Unix/non-Windows 64 KiB and Windows 256
   KiB without weakening depth or iterative-cleanup assertions.
4. Require independent prepare/disposal/commit counters on every path.
5. Preserve lifecycle/cleanup GREEN evidence under `/tmp/task22o26-green-public`.

## Task 10 — Typed archives, parent-bound KSR, metamorphism, and WASM

Files and budgets:

- add integration `ksr.rs`, `ksr/digest.rs`, `options.rs`, `archive.rs`, and
  `metamorphic.rs`, each at most 300 LOC;
- minimally extend `crates/ares-wasm/tests/browser/project-slice-page.mjs` and
  `project-slice.spec.mjs`; split an ordinary vector shard if needed.

Steps:

1. Parse the real KSR 3MF twice independently through normal project APIs and
   reassert exact O25 parent evidence before capturing O26.
2. Freeze resolved KSR values from typed aligned records: EnsureAll,
   top-shell layers 5, bottom-shell layers 3, top thickness 1 mm, bottom
   thickness 0 mm, and sparse density 15%. Do not trust raw archive strings as
   the execution assertion.
3. Freeze two equal O26 captures with explicit object/slot/record/event framing:
   460 record visits, 460 EnsureAll skips, zero source-kind visits, geometry,
   rebuilds, dirty commits, and changed records; exact O25/O26 allocations,
   kinds, geometry, metadata, and sequence digests; one prepare/disposal.
4. Add a normal typed archive mutation that activates O25 promotion while
   EnsureAll proves O26 observes and preserves it.
5. Add a separate real typed archive mutation to Moderate with normal shell
   options. Require nonzero source/neighbor/rebuild/dirty commits through
   resolved options. Add direct synthetic geometry only for branches absent from
   that archive.
6. Add independent resolved model-part/archive witnesses for the EnsureAll gate,
   top count/thickness window, bottom count/thickness window, sparse-density
   stop/continue branch, neighbor outer-wall-derived external flow, and current
   internal-solid-infill-derived solid flow. Add a direct retained-state witness
   with deliberately distinct source/neighbor flow fields to prove production
   reads each aligned `PerimeterInputRecord` rather than global options or a
   recomputed/first-record flow.
7. Prove invariance under ZIP entry reversal, Stored compression, timestamp
   changes, and non-slicing rename. Printable-area scale and transforms may
   change predecessor/O26 geometry only through typed state, never fixture
   branching.
8. Execute both EnsureAll and active archives through existing optimized
   `sliceProject`; retain `slice_stl` regressions, no new export, no trap, and
   exact `ProjectSlicingIncomplete` on successful O26. Run both Playwright
   repetitions.

## Task 11 — Honest compiling mutation evidence

After clean GREEN, save byte-exact production backups under `/tmp`, apply one
compiling mutation at a time, run its intended focused witness, and restore
byte-for-byte before continuing. Mutation RED is not chronological TDD RED.

Required mutations include:

1. invoke O26 before O25 or bypass public O26;
2. remove/reverse EnsureAll gate;
3. reorder source kinds;
4. gather fill before slices, omit holes, or pre-union paths;
5. use stored layer ID or reverse top/bottom direction;
6. replace exact count `== 0` with `<= 0`/unsigned saturation;
7. replace count OR thickness with AND or alter strict EPSILON;
8. globally pre-gather, isolate source decisions, or skip a None neighbor;
9. swap stop-kind and continue-neighbor behavior;
10. read one global/first-record option set or change any density/mode factor;
11. use a recomputed/wrong aligned flow owner or bypass scaled-integer truncation;
12. use symmetric opening, opening Miter-3, or omit `+10.0_f32`;
13. change the repair expansion from Miter-3/its shortest-edge configuration;
14. update carried solid in the second filter;
15. include bridge externals in repair or exclude InternalVoid;
16. skip existing-solid union or Internal safety difference;
17. retain InternalVoid during reconstruction;
18. sort/group by kind, include extra_perimeters, or use the wrong template;
19. mutate originals before a late failure;
20. replace clean records or clear dirty on a geometry-equal rebuild;
21. bypass iterative cleanup or inherited alignment/scale validation.

Every mutation must fail its intended behavioral test, not compilation or an
unrelated assertion. Compare final production hashes to the saved manifest and
rerun affected GREEN plus the full gate.

## Task 12 — Full local gates and audits

1. Run focused geometry/O26 direct and integration filters, then explicit
   O21-O26 regressions.
2. Run `cargo nextest run --workspace --no-fail-fast`.
3. Run:
   - `cargo check --workspace --all-targets`;
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
   - all four wasm32 checks from `.github/workflows/tier1.yml`;
   - optimized default/feature WASM builds, wasm-bindgen/export audit, and both
     Playwright repetitions;
   - `cargo fmt --all -- --check` and `git diff --check`.
4. Audit every Rust file `<400 LOC` and every new O26 shard `<=300 LOC`; no
   dependency diff; no staged `.pi-subagents/` or `target/parity/` artifact.
5. Audit added code for no `unsafe`, `include!`, `include_bytes!`, broad lint
   allowance, binary oracle, source-text/hash/line pinning test,
   reference-G-code access, fixture identity branch, Orca command/FFI, or
   fallback.
6. Mechanically audit rollback: restore O25 terminal consumption and remove
   only O26 module/state/wiring/tests/docs, path-opening adapter, and narrow
   template seam; retain O25 unchanged.

## Task 13 — Documentation, six-dimensional review, commit, push, exact-SHA CI

1. Update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec,
   and this plan with exact O26 captures/digests, test and mutation totals,
   limitation/rollback, review paths, and the next cited upstream boundary.
2. Rerun both complete spec reviews and both complete plan reviews after
   substantive evidence edits.
3. Dispatch one fresh independent review-only thread over requirements
   completeness, logic correctness, boundary cases, code quality, test coverage,
   and actual execution. Dispatch a separate default-model OpenCode review over
   the identical final diff/evidence. Require literal `VERDICT: APPROVE` from
   both.
4. The parent remains sole writer. Fix every blocker, search for siblings,
   rerun affected and complete gates, and return the identical revised state to
   both reviewers until approval.
5. Use Conventional Commits, separating implementation and final docs/evidence
   where practical. Never stage `.pi-subagents/` or `target/parity/`.
6. Push `main`, verify clean `HEAD == origin/main`, and require the final pushed
   O26 SHA's complete Tier-1 Windows, macOS, Ubuntu, format, optimized
   browser-WASM/export audit, and both Playwright runs to pass. Pending/failing
   exact-SHA CI blocks shipping and any CI repair repeats gates and reviews.

## Frozen local evidence

- real KSR EnsureAll: surface digest
  `-107673730348313625723619859456104452971`, event digest
  `55157732452648897477979936233453742487`, visits/skips `460/460`, zero
  source/neighbor/geometry/rebuild/commit activity;
- typed Moderate archive: surface digest
  `55371787254720044626064449746884984931`, event digest
  `71433667081695804905700384637078674080`, raw event totals
  `[460, 460, 0, 1380, 1010, 547, 143]` in FillClone/RecordVisit/
  EnsureAllSkip/SourceKindVisit/NeighborVisit/Rebuild/DirtyCommit order,
  geometry-event count `5469`; all 547 rebuilds follow nonempty intersections
  and commit 143 distinct dirty records;
- final O26 filter `45/45`, supporting opening `6/6`, template `1/1`, full
  workspace `5908 passed, 2 skipped`;
- `33/33` compiling behavioral mutations killed with no survivor or compile
  failure and restored production source; the set includes public/EnsureAll,
  ID/direction/EPSILON, pre-gather/global-option, asymmetric opening, solid
  union/Internal difference, original-before-failure, validation and iterative
  cleanup mutations;
- controlled serial gather, all-site rollback fingerprints, full active
  sidecar/clean-geometry ownership, an actual geometry-equal production
  rebuild, resolved option/flow/scale variants, and rename invariance pass;
- final formatting, all-target native check, and strict all-target/all-feature
  Clippy pass; optimized native/WASM, export, and two executed 11-test browser
  runs pass on the existing `sliceProject` boundary;
- no dependency, persisted-format, public-export, fallback, reference-G-code,
  fixture-identity, source-pinning, `unsafe`, or include-macro addition.

Final independent six-dimensional and default-model OpenCode implementation
reviews approve the current diff. The exact pushed-SHA Tier-1 run remains the
release gate until observed.

## Rollback

Restore O25 as the production terminal consumer; remove O26
module/state/wiring/tests/docs, the path-opening adapter and its narrow
`geometry/clipper/offset.rs`, `geometry/clipper.rs`, and `geometry.rs` reexports,
and the narrow RegionSurface template seam; retain O25 options, geometry, sidecars,
dependencies, persisted formats, public API, and incomplete disposition.

## Stop condition

Stop O26 only when pinned `PrintObject.cpp:3974-4150` runs in exact serial order
from typed aligned O25 state, later gathers observe earlier working mutations,
all geometry/casts/topology/rebuild/metadata semantics are frozen, whole-project
rollback and dirty-only ownership are proven, KSR EnsureAll no-op and active
archive behavior are parent-bound, public once-only wiring and iterative cleanup
are verified, local/Tier-1 gates are green, synchronized docs and both final
review paths approve, commits are pushed, and the exact pushed SHA CI succeeds.
