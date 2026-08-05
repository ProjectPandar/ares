# Task 22O.25 — Horizontal-shell extra-solid promotion Plan

Spec: `docs/superpowers/specs/2026-08-05-ksr-fdmtest-v4-task22o25-horizontal-shell-extra-solid-promotion.md`

## Status

Executed from Ares baseline `551a8c10420783cd56f185fe8e22ea3d3eab015c`
against pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The revised spec has literal
`VERDICT: APPROVE` from the independent reviewer at
`/tmp/task22o25-spec-independent-rereview.md` and the separate default-model
OpenCode reviewer at `/tmp/task22o25-spec-opencode-rereview.txt`; this complete
plan was approved at `/tmp/task22o25-plan-independent-rereview-2.md` and
`/tmp/task22o25-plan-opencode-rereview-2.txt` before production edits. The
implementation, local evidence, and both final review paths are complete and
approved; commit/push and the exact pushed-SHA Tier-1 matrix remain the ship
gate.

## Validation contract

Port only `PrintObject::discover_horizontal_shells` source-order iteration and
`extra_solid_infills` promotion at `PrintObject.cpp:3955-3972`, stopping before
the EnsureAll gate at line 3974. Read each aligned record's typed resolved raw
schedule, preserve the raw-empty short circuit, use the existing typed schedule
parser/matcher for nonempty values, and retag every and only Internal
`fill_surfaces` in place. Stage the whole project before mutation, retain the
exact O24 graph and iterative cleanup, and keep public slicing incomplete. No
geometry, sparse-density gate, branch-state sidecar, new dependency, public
API, fallback, fixture branch, or later horizontal-shell behavior is included.

## Gate 0 — Baseline and reviewed design

1. Confirm Ares `HEAD == origin/main ==
   551a8c10420783cd56f185fe8e22ea3d3eab015c`, Orca checkout
   `8500fcdccaa10b5099ac20d252af3a7c560046f1`, and no tracked baseline changes
   except the reviewed O25 spec/plan.
2. Require O24 exact-SHA Tier-1 run `31004838262` to finish with format,
   macOS, Ubuntu, Windows, optimized browser-WASM/export audit, and both
   Playwright runs successful before O25 ships. Repair O24 first if it fails.
3. Preserve both final spec approvals named above.
4. Send this complete plan to one fresh independent review-only agent and one
   separate default-model OpenCode review. Any plan edit returns the complete
   revised plan to both. Require literal `VERDICT: APPROVE` from both before
   production edits.

## Task 1 — Source-sized typed schedule seam, test-first

Files and budgets:

- modify `crates/ares-core/src/options/infill/extra_solid.rs`;
- minimally reexport the schedule from `options/infill.rs` and `options.rs`;
- extend ordinary tests in
  `crates/ares-core/src/options/tests/extra_solid_infills.rs`, splitting into a
  real child module if that file would approach 400 LOC.

Steps:

1. Add compiling RED tests for a crate-private raw-string parser covering empty,
   whitespace/quotes, repeating `N`, `N#K`, comma lists, explicit ranges,
   `i32::MAX`, `i32::MAX + 1`, and an explicit range whose end is near the
   source-sized bound. Require identical values/errors through the existing JSON
   entry and the exact error `invalid extra_solid_infills pattern`.
2. Add a matcher RED proving near-boundary explicit ranges never panic and use
   checked end arithmetic on both native `usize` widths. The test data must be
   architecture-independent. Task 5 adds an executed optimized browser-WASM
   witness for the same raw and JSON boundary cases; wasm32 compilation alone
   is not accepted as arithmetic/error evidence.
3. Preserve assertion-based compiling RED at
   `/tmp/task22o25-red-extra-solid-parser.txt`.
4. Refactor `ExtraSolidInfills::parse` so a new crate-private
   `parse_raw(&str)` owns the existing normalization/grammar and JSON parsing
   delegates to it. Parse positive decimal components as signed `i32`, reject
   zero/negative/out-of-range values, then convert to `usize`. Preserve strict
   Ares token parsing and the existing stable error; do not imitate
   `std::stoi` prefix acceptance in this milestone.
5. Make explicit-range matching use checked addition and return false on an
   impossible overflow rather than panic. Add only the minimum crate-private
   reexport needed by project slicing.
6. Rerun the byte-identical focused command GREEN and preserve
   `/tmp/task22o25-green-extra-solid-parser.txt`. Run all option/registry/config
   codec regressions affected by the shared parser.

## Task 2 — O25 state shell and direct promotion RED/GREEN

Production files and budgets:

- add `prepare_infill/horizontal_shell_promotion.rs`, at most 160 LOC;
- add `horizontal_shell_promotion/{types,promote,stage,cleanup}.rs`, each at
  most 300 LOC;
- direct test root `horizontal_shell_promotion/tests.rs` with ordinary child
  shards, each at most 300 LOC.

Steps:

1. Define `PreparedPostHorizontalShellPromotion` with the exact O24 fields:
   boxed predecessor, objects, caches, projections, trims, regularizations, and
   filters. Add no durable geometry or branch-state sidecar.
2. Define a transient staged record decision (`Noop` or `PromoteInternal`) and
   test-only ordered events that distinguish `RawScheduleVisit`,
   `NonemptySchedule`, `ParserInvocation`, `MatcherInvocation`, and each
   promoted surface. Add separate prepare-invocation, commit, and
   cleanup/disposal counters following O24's test-hook pattern; reset/read hooks
   must let direct, failure, lifecycle, and KSR tests assert every count
   independently without altering production data.
3. Add minimal compiling function shells, then direct behavioral REDs for:
   - exact raw-empty short circuit: visit only, no parser/matcher/promotion;
   - a nonempty whitespace/quote-only raw string: visit, nonempty, parser,
     matcher, no promotion, proving the raw `.is_empty()` guard is not applied
     after normalization;
   - ordinary nonempty nonmatch: visit, nonempty, parser, matcher, no commit;
   - matching mixed collection: every and only Internal becomes InternalSolid;
   - multiple Internal surfaces, exact event/order accounting;
   - vector pointer/capacity, inner geometry allocations, metadata, contour,
     hole, point, and unrelated kind preservation;
   - `slices` and all unrelated record fields unchanged;
   - one-based planned array index distinguished from stored layer ID;
   - idempotence;
   - promotion at sparse density 0%, 15%, and 100%, killing accidental legacy
     density gating.
4. Preserve compiling RED at `/tmp/task22o25-red-direct-promotion.txt`.
5. Implement crate-private
   `stage_decision(raw: &str, planned_layer_index: usize)` in `promote.rs` before
   the Task 2 GREEN checkpoint. It emits the exact events above, short-circuits
   only an exactly empty raw string, calls `ExtraSolidInfills::parse_raw` for
   every other string, invokes `matches_layer`, and returns the transient
   decision without mutating a record. Task 3 must reuse this exact seam after
   inherited alignment validation rather than reimplementing option logic.
6. Implement record commit only with `RegionSurface::retag` in existing surface
   order. Never reconstruct, clone, sort, filter, reserve, or change geometry.
   Empty/nonmatching decisions must not touch the record.
7. Rerun the identical focused command GREEN and preserve
   `/tmp/task22o25-green-direct-promotion.txt`.

## Task 3 — Complete inherited alignment and whole-project staging

Integration files and budgets:

- add `project_slice/tests/prepare_infill/horizontal_shell_promotion.rs`;
- add ordinary child modules `fixture.rs`, `transaction.rs`,
  `transaction/mismatches.rs`, `ownership.rs`, `cleanup.rs`, `lifecycle.rs`,
  `options.rs`, `metamorphic.rs`, and `ksr.rs`, splitting further before any
  file reaches 400 LOC;
- declare the root from `project_slice/tests/prepare_infill.rs`.

Steps:

1. Reuse the real O24 preparation fixture; do not embed fixture bytes or inspect
   reference G-code.
2. Add compiling REDs for every inherited O24 alignment relation: typed
   printable-area scale, outer sidecar/object lengths, record and slot
   alignment, plan/input/prelude/lslice counts, source/transform identity,
   planned array index/layer ID, current layer/region, region ID, and the
   one-compatible-region envelope.
3. Add a genuine later-object typed schedule failure after a complete earlier
   valid object. Require zero commits, no successor, the exact invalid-pattern
   error, and iterative disposal of the unmodified O24 graph. If the normal
   fixture builder cannot construct two objects, extend only test fixture
   builders; never add production fallback or per-fixture logic.
4. Preserve transaction RED at
   `/tmp/task22o25-red-integration-transaction.txt`.
5. Implement complete alignment validation before the first O25 event. While
   borrowing O24, walk stable object/record order and for every populated record:
   - obtain the aligned input and its `region_options(input)`;
   - pass `region_options.extra_solid_infills.0.as_str()` (the `.0` belongs to
     the existing `OrcaString`, not to `ExtraSolidInfills`) and
     `input.planned_layer_index` to Task 2's `stage_decision` seam;
   - stage the returned decision without duplicating raw-empty, parser, event,
     or matcher logic.
6. Only after all staging succeeds, destructure/move O24 and commit decisions in
   stable object/record order. On any error, dispose exact O24. Construct the
   successor with exact outer allocation identity.
7. Implement cleanup by reconstructing
   `PreparedPostVerticalShellAssignment` and delegating to O24 disposal.
8. Rerun the transaction command GREEN and preserve
   `/tmp/task22o25-green-integration-transaction.txt`.

## Task 4 — Ownership, cleanup, public lifecycle, and precedence

1. Add ownership REDs proving predecessor, object, record vector, all O19-O24
   sidecars, non-kind record fields, surface vector buffers, inner geometry,
   and metadata retain identity/content; only matched Internal discriminants
   change.
2. Add constrained-stack REDs for both independent 10,000-node predecessor
   families across direct success, later parse failure, and public-incomplete
   disposal. Use only the shared Unix/non-Windows 64 KiB and Windows 256 KiB
   baseline; do not weaken depth or iterative-cleanup assertions.
3. Add lifecycle REDs proving the intended O17→O25 order, exactly one O25
   invocation, zero O25 invocation for every earlier capability/O17/O19/O20/
   O21/O22/O23/O24 failure, and exact predecessor error precedence.
4. Require separate cleanup/disposal counts on direct success, parse failure,
   and public incomplete, and reset them alongside prepare/event/commit hooks.
5. Declare `horizontal_shell_promotion` from `prepare_infill.rs`. In
   `project_slice.rs`, invoke O25 once after O24, replace production O24 terminal
   consumption with O25 consumption, dispose O25, and continue returning
   `ProjectSlicingIncomplete`. Retain the O24 consumer under `#[cfg(test)]` for
   focused predecessor regressions.
6. Rerun the identical ownership/cleanup/lifecycle filters GREEN and preserve
   `/tmp/task22o25-green-lifecycle-cleanup.txt`.

## Task 5 — Typed provenance, metamorphism, and active archive behavior

1. Add a normal typed archive mutation that introduces a valid nonempty
   `extra_solid_infills` value matching at least one planned layer with Internal
   surfaces. Assert nonzero matching records/promoted surfaces from parsed state,
   and freeze before/after kind accounting while geometry/metadata digests and
   allocations remain invariant.
2. Add a nonmatching valid pattern and malformed later-object pattern through
   the same typed config path. Require no fixture name/hash/layer/geometry
   branch and exact transaction behavior.
3. Prove model-part/object precedence selects each aligned record's resolved
   schedule rather than one global/first-record value.
4. Repack the 3MF with reversed ZIP order, Stored compression, timestamp
   changes, and a non-slicing rename. O25 evidence must remain invariant.
5. Preserve transform/printable-area mutations as changes only through normal
   O24 predecessor geometry; O25 itself performs no geometry and must not branch
   on those values after alignment succeeds.
6. Add an executed optimized browser-WASM boundary matrix using the existing
   public bindings only, without a new export or alternate pipeline:
   - modify `crates/ares-wasm/tests/browser/project-slice.spec.mjs` and
     `project-slice-page.mjs`, splitting an ordinary `.mjs` vector helper if
     needed;
   - through the existing generated `bindings.slice_stl` export, feed JSON
     `SliceOptions` values for
     `i32::MAX`, `i32::MAX + 1`, and a near-boundary explicit range, requiring
     valid values to resolve normally and invalid values to reject with exactly
     `invalid extra_solid_infills pattern` without a trap;
   - through `bindings.sliceProject`, apply the same values to the real 3MF's
     resolved `RegionOptions` raw path, requiring valid values to reach
     `ProjectSlicingIncomplete` and invalid values to return the identical
     stable error with no partial successor/trap;
   - execute this test in both optimized Playwright repetitions from Tier-1.
   Compilation-only wasm32 checks do not satisfy this witness.

## Task 6 — Freeze parent-bound real KSR evidence

1. Parse `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` independently twice
   through normal project APIs.
2. Reassert the final O24 checksum
   `-117597382518472843802490205604634875775`, record digest
   `-65994586923856785425316699963519338136`, event digest
   `-110138798119262824097709645699717637653`, pre/post kind and geometry
   totals, 460 records, 161 active O24 records, and 299 O24 no-op/unchanged
   records before freezing O25.
3. Build an O25 capture from actual aligned options/records, with explicit
   object/slot/record/surface/path/metadata framing. Freeze only after two
   independent captures agree:
   - 460 aligned raw schedule visits;
   - zero nonempty guards, separate parser invocations, matcher invocations,
     matches, promoted surfaces, commits, and changed records;
   - byte-logically unchanged O24/O25 kind, geometry, metadata, and record
     sequence digests;
   - exactly one prepare invocation and one cleanup/disposal, asserted through
     independent counters rather than inferred from other events.
4. Keep literals only in tests/docs. Never read or derive expectations from the
   reference G-code.

## Task 7 — Honest compiling mutation evidence

After clean GREEN, save byte-exact production backups under `/tmp`, apply one
compiling mutation at a time, run the intended focused witness, preserve RED,
and restore byte-for-byte before continuing. These are post-implementation
mutation REDs, not chronological pre-implementation RED.

Required mutations:

1. omit one-based conversion;
2. use stored layer ID instead of planned array index;
3. reverse the match condition;
4. promote only the first Internal;
5. promote InternalVoid/InternalSolid or external kinds;
6. reconstruct promoted surfaces with default metadata;
7. consult one global/first-record schedule;
8. introduce the legacy sparse-density gate;
9. mutate `slices` instead of `fill_surfaces`;
10. bypass the raw-empty short circuit and invoke parser/matcher for KSR;
11. bypass O25 public wiring;
12. remove retained scale/alignment validation;
13. commit an earlier object before a later schedule parse failure;
14. restore unrestricted `usize` parsing or unchecked explicit-range addition.

Every mutation must fail its intended behavioral test rather than compilation or
an unrelated assertion. After final restoration, compare all production files
byte-for-byte and rerun affected GREEN plus the complete gate.

## Task 8 — Full local gates and audits

1. Run exact direct O25 and integration filters, then explicit O21-O25
   regressions.
2. Run `cargo nextest run --workspace --no-fail-fast`.
3. Run:
   - `cargo check --workspace --all-targets`;
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
   - all four wasm32 checks from `.github/workflows/tier1.yml`;
   - optimized default and feature browser-WASM builds, wasm-bindgen/export
     audit, and both Playwright runs exactly as Tier-1, including the executed
     O25 JSON/raw numeric-boundary matrix from Task 5;
   - `cargo fmt --all -- --check` and `git diff --check`.
4. Audit every Rust file `<400 LOC`, every new O25 shard `<=300 LOC`, no
   dependency diff, and no staged `.pi-subagents/` or `target/parity/` artifact.
5. Audit added Rust for no `unsafe`, `include!`, `include_bytes!`, broad lint
   allowance, binary oracle payload, source-text/hash/line pinning test,
   reference-G-code access, fixture identity branch, Orca command/FFI, or
   fallback.
6. Mechanically audit rollback: restore O24 terminal consumption and remove
   only O25 modules/state/wiring/tests/docs plus the narrow raw-string parser
   seam/reexports; retain O24 and the legacy density-gated helper unchanged.

## Task 9 — Six-dimensional review, docs, commit, push, exact-SHA CI

1. Update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec,
   and this plan with the exact O25 seam, frozen KSR counters/digests, test and
   mutation totals, and next boundary at `PrintObject.cpp:3974`.
2. Rerun both spec reviewers and both plan reviewers after evidence edits. Any
   substantive doc edit repeats the relevant complete gate.
3. Dispatch one fresh independent review-only thread over requirements
   completeness, logic correctness, boundary cases, code quality, test coverage,
   and actual execution. Dispatch a separate default-model OpenCode review over
   the same final diff/evidence. Require literal `VERDICT: APPROVE` from both.
4. The parent remains sole writer for reviewer fixes. Fix every blocker, search
   for siblings, rerun affected and full gates, and return the identical revised
   diff/evidence to both reviewers until both approve.
5. Use Conventional Commits, keeping implementation and final evidence/docs
   commits separate where practical. Do not stage `.pi-subagents/` or
   `target/parity/`.
6. Push `main`, verify clean `HEAD == origin/main`, and require the final pushed
   O25 SHA's complete Tier-1 Windows, macOS, Ubuntu, optimized browser-WASM
   export audit, and both Playwright runs to succeed. A pending/failing exact-SHA
   run blocks shipping; local gates and the predecessor O24 run are not
   substitutes. Any CI repair repeats affected gates, reviews, commit, push, and
   exact-SHA verification.

## Executed evidence

Tasks 1-8 produced the source-sized shared parser, exact O24 successor,
whole-project staging, iterative cleanup, public once-only wiring, typed archive
and browser-WASM witnesses, and parent-bound KSR capture without a new API or
dependency. The frozen KSR evidence is checksum
`58727684244877231975278290246623082466`, record digest
`160750122870413723145549886803558415603`, event digest
`95826544899519698779358289371798515623`, unchanged surface digest
`-107673730348313625723619859456104452971`, 460 unchanged records, events
`[460, 0, 0, 0, 0]`, zero commits, and one prepare/disposal. The active typed
archive witness promotes 1,281 surfaces in 460 records.

All 14 required compiling mutations failed their intended behavioral witnesses
and production restoration matched the saved SHA manifest byte-for-byte.
Forty-two focused O25/shared-option tests, 191 explicit O21-O25 regressions,
and 5,856 workspace tests with 2 skipped pass. Native checks, strict Clippy,
four WASM checks, optimized default and feature export audit, two 10-test
Playwright runs, and static repository audits complete the local gate.
Reviewer-added coverage freezes holed contour/hole/point
identity, every O19-O24 sidecar allocation/content sequence, unrelated record
buffers, all earlier capability/error precedence, and transform/printable-area
metamorphism. Both implementation reviewers approve this final evidence; Task
9 closes only after the exact pushed SHA is green.

## Rollback

Rollback restores O24 as the production terminal consumer; removes O25
module/state/wiring/tests/docs and the narrow crate-private raw-string schedule
parser seam/reexports; and leaves O24 geometry, vocabulary, sidecars, the legacy
`layer_role::sparse_or_extra_solid` path, dependencies, persisted formats, and
public API unchanged.

## Stop condition

Stop O25 only when pinned `PrintObject.cpp:3955-3972` is derived from typed
aligned O24 state with exact raw-empty short circuit, source-sized deterministic
parser behavior, one-based planned-array matching, stable allocation-free
Internal promotion, whole-project transactionality, exact ownership, iterative
cleanup, parent-bound KSR no-op evidence, active typed archive evidence, public
once-only wiring, complete local/Tier-1 gates, synchronized docs, both final
review paths approved, commits pushed, and the final exact-SHA CI fully green.
