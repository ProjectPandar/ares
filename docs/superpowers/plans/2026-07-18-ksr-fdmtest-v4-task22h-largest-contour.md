# Task 22H Implementation Plan: Post-Closing Largest-Contour Selection

## Status, fixed points, and success condition

This plan is a draft. No production or test implementation is authorized until
the exact specification and plan bytes receive all pre-implementation review
approvals.

The fixed Ares baseline is commit
`b53a0a7432b5c71d4a1f3b15139fbb873674f09e`, tree
`5931e386545fe919fb420323017a6a3a497acf45`; exact-SHA Tier-1 run
`29653761751`, attempt 4, is green on all five jobs. The fixed OrcaSlicer
source is commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`, with the blobs listed in the Task
22H specification.

Success means:

- exact source-order signed `f64` polygon area is available to the private
  geometry domain;
- exact post-closing `keep_largest_contour_only` behavior is applied only to
  retained `PositiveLargestContour` layers;
- every ranking and ownership edge case is frozen in focused tests;
- the committed KSR project matches the complete approved `ARES22H` checkpoint;
- a 3MF-only spiral/bottom-shell mutation matches the non-vacuous approved
  checkpoint and proves 337 layers actually select;
- the independent threshold-21 3MF mutation matches its approved checkpoint,
  proves only 336 later layers select, and preserves behavior-bearing Regular
  slot 20 unchanged;
- native, WASM, browser, structural, provenance, and review gates pass while
  the public API still reports `ProjectSlicingIncomplete`;
- exact reviewed bytes are committed, pushed normally, and green in exact-SHA
  Tier-1 before Task 22I begins.

Task 22H does not claim complete normalized G-code parity.

## Immutable behavior ledger

The implementation must preserve these non-substitutable facts:

1. Selection runs after Task 22G closing and before Task 22I simplification.
2. Only retained `PositiveLargestContour` layers select; all other modes are
   exact identities.
3. Empty and single-element vectors are exact identities, including a single
   CW or degenerate contour.
4. Multiple candidates start with maximum `0.0` and no selected item.
5. Ranking uses signed `candidate.contour.area()`, not absolute area and not
   the ExPolygon's net area after holes.
6. The comparison is strict `>`; the first equal positive maximum wins.
7. The whole selected ExPolygon is moved, including all holes and their order.
8. Contour/hole start points, directions, point order, and coordinates are not
   normalized.
9. A multiple-candidate layer with no positive contour is an internal
   invariant failure, not an external error or fallback case.
10. Polygon area casts each i64 coordinate to `f64` before the two products,
    subtracts in source order, accumulates serially, and multiplies by 0.5.
11. The existing raw-loop PositiveLargestContour helper is not reusable: it
    uses absolute area, `swap_remove`, and orientation repair at an earlier
    stage.
12. Object, volume, layer, mode, empty slot, source index, ordinal, and plan
    order remain observable.
13. The selector consumes only the mode already derived from 3MF Options; it
    does not re-read or invent an Option.
14. KSR baseline is a geometry no-op because all 460 layers are Regular; it is
    not accepted as the only behavior proof.
15. The three-Option mutation changes only `spiral_mode`,
    `bottom_shell_layers`, and `bottom_shell_thickness` and must produce a real
    selector RED before code.
16. The threshold-21 mutation changes only `spiral_mode` and
    `bottom_shell_layers`; it must cross multi-ExPolygon slot 20 and produce a
    distinct real selector RED before code.
17. Resolution and every simplify/StrictlySimple behavior remain Task 22I.

## Working protocol

Work proceeds in serial TDD packages. For every package:

1. freeze exact allowed paths and source boundary;
2. add only package-owned tests in separate modules;
3. run focused `cargo nextest` or compile/browser command and record the real
   expected RED;
4. implement the smallest source-cited behavior that makes that RED green;
5. run focused regressions, rustfmt, relevant Clippy, LOC, and macro checks;
6. freeze a per-file hash manifest;
7. obtain independent specification and quality approval before the next
   package begins.

The complete baseline and both mutated KSR oracles are registered in Package 0.
The baseline may pass after H checkpoint plumbing because fixed source proves
it is a no-op. Both unchanged mutated oracles must stay red until Package B wires
selection. Package C promotes those already-registered assertions to final
native/browser green; it does not add expected values after production exists.

The ignored evidence ledger is `.superpowers/sdd/task22h-evidence.md`. It
records commands, exit codes, nextest run IDs, hashes, reviews, repair rounds,
release identity, and Tier-1. Ignored evidence is never a build or test
dependency.

Use `apply_patch` for source and document edits. Do not modify committed
fixtures. Do not amend, squash, force-push, or rewrite released Task 22A-G
history.

## Pre-implementation exact-byte gate

Before Package 0:

1. obtain two independent read-only approvals of the fixed Task 22H probe,
   committed-project input/output, EOF, counts, and representatives;
2. obtain two independent read-only approvals of the same probe applied to both
   exact 3MF Option-mutation Task 22G streams, including mutation provenance,
   thresholds, modes, selected layers, output digests, and EOF;
3. freeze specification and plan SHA-256 values;
4. dispatch an independent fixed-source/spec reviewer;
5. dispatch an independent current-Ares/plan reviewer;
6. run a direct default-model review with task/edit tools denied;
7. require literal approval from every reviewer on the same exact bytes.

Any spec or plan edit invalidates all document approvals. Any unresolved P0-P3
finding blocks implementation.

## Exact planned tracked manifest

No tracked path outside this list may change without a plan amendment and
fresh approvals.

### Specification, architecture, and roadmap

- `docs/superpowers/specs/2026-07-18-ksr-fdmtest-v4-task22h-largest-contour.md`
- `docs/superpowers/plans/2026-07-18-ksr-fdmtest-v4-task22h-largest-contour.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

### Core feature and geometry

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/polygon.rs`
- `crates/ares-core/src/geometry/expolygon.rs`
- `crates/ares-core/src/geometry/tests/polygon.rs`
- `crates/ares-core/src/geometry/tests/expolygon.rs`

### Project stage and tests

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/closing.rs`
- `crates/ares-core/src/project_slice/largest_contours.rs`
- `crates/ares-core/src/project_slice/task22g_oracle.rs`
- `crates/ares-core/src/project_slice/task22h_oracle.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/largest_contours.rs`
- `crates/ares-core/src/project_slice/tests/largest_contour_fixture.rs`

### WASM browser conformance

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/index.html`
- `crates/ares-wasm/tests/browser/package.json`
- `crates/ares-wasm/tests/browser/package-lock.json`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `.github/workflows/tier1.yml`

The planned maximum is 26 tracked paths. Deleting a planned unused new path is
allowed before candidate freeze; adding a path is not.

## Module ownership and line budgets

All Rust production and test files must remain below 400 physical LOC. Start a
real module split before the limit.

| Module | Ownership | Budget |
| --- | --- | ---: |
| `geometry/polygon.rs` | points plus exact serial signed area | 80 |
| `geometry/expolygon.rs` | contour/hole ownership plus pure selector | 100 |
| `geometry/tests/polygon.rs` | area and point-preservation vectors | 160 |
| `geometry/tests/expolygon.rs` | pure selector vectors | 240 |
| `project_slice/largest_contours.rs` | in-place mode-gated traversal | 100 |
| `project_slice/closing.rs` | released stage plus narrow mutable accessors | 240 |
| `project_slice/task22h_oracle.rs` | marker wrapper only | 40 |
| `tests/largest_contours.rs` | synthetic project-stage ownership | 300 |
| `tests/largest_contour_fixture.rs` | full baseline/mutation oracles | 360 |
| `project_slice.rs` | released pipeline plus post-largest seam | 330 |
| `ares-wasm/src/lib.rs` | existing adapter plus gated byte hook | 170 |
| browser HTML | complete G/H checkpoint parser and hooks | 260 |
| browser Playwright spec | real 3MF mutation and assertions | 320 |

Do not add tests to the existing 395-LOC closing test, 384-LOC support module,
or 332-LOC closing fixture. New tests use the two planned modules. Source
splitting uses real Rust `mod` files only; `include!` and `include_bytes!` are
forbidden for splitting production or test source.

## Exact implementation shape

### Geometry

`Polygon::area()` is crate-private and uses the fixed source operation order.
It does not call Clipper's predicates area helper because that helper uses a
different algebraic reduction order.

`keep_largest_contour_only(&mut Vec<ExPolygon>)` is crate-private. For more
than one item, it scans with `enumerate`, records only strict positive maxima,
removes the selected value without reordering its internal bytes, clears the
vector, and pushes the selected value. Any temporary sibling reordering is
unobservable after the clear. The multiple/all-nonpositive case uses one
private `expect` matching the source invariant.

### Project stage

`project_slice/largest_contours.rs` accepts a mutable slice of Task 22G
`PostClosingPrintObject` records. `closing.rs` exposes only the mutable
object-volume, volume-layer, and layer-ExPolygon accessors required by that
traversal plus the retained mode. No new public type or external API is added.

`prepare_post_largest_contours()` calls `prepare_post_closing()`, applies the
stage once, and returns the same complete project/config/ownership bundle.
`slice_project` consumes this later bundle and remains incomplete. Native Task
22G tests continue to call the earlier post-closing checkpoint.

### Checkpoint and browser feature

The existing Task 22G encoder exposes one private `encode_with_magic` helper.
Its native Task 22G wrapper and tests remain unchanged. A tiny Task 22H module
calls the same encoder with `ARES22H\0` only after post-largest preparation.

Replace the non-default Cargo feature and WASM export
`task22g-browser-oracle` with `task22h-browser-oracle`. Do not keep a feature
alias or `task22gBrowserOracle` WASM compatibility wrapper. Under `cfg(test)`,
the core Task 22G hook remains available for regression and pre/post
comparison. Under the H conformance feature, expose separately named byte-only
`task22h_browser_input_oracle` and `task22h_browser_oracle` functions so the
browser can inspect the exact pre-stage G stream and post-stage H stream.

The feature gates visibility only. It cannot alter selection, Options,
coordinates, expected values, or error behavior.

The browser test package adds exact `fflate=0.8.3` as a pinned Node-side dev dependency.
Playwright unzips the committed 3MF, makes exact unique text replacements in
`Metadata/project_settings.config`, rezips it, and passes the complete archive
bytes to both WASM hooks. No mutation or Option is passed out of band, and the
dependency is absent from production crates and generated WASM bindings.

## Error and invariant contract

Task 22H adds no new external error. Parsed 3MF and prior geometry stages
remain the existing boundaries. The only new failure is the trusted internal
multiple/all-nonpositive invariant, expressed as a private panic/expect.

No error or panic text names KSR, a fixture path/hash, reference G-code,
expected digest, Orca source path, or fallback.

## Oracle registration

The approved committed-project constants are:

- length 1,644,681;
- SHA-256
  `e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163`;
- 1 object, 1 volume, 460 Regular layers, 2,890 contours, 395 holes, 99,212
  points;
- Task 22G body byte identity and only magic offset 6 changed;
- representative layer hashes listed in the specification.

The approved non-vacuous 3MF mutation constants are:

- Task 22G input 907,601 bytes / SHA-256
  `0ca404fa4a5a6fb0a97899fe6ff8fd45815a9439378708bbe594614587e38034`;
- modes `2/0/0/458`, 2,622 contours, 14 holes, 53,603 points, exact EOF;
- 337 PLC layers with multiple candidates;
- Task 22H output 427,465 bytes / SHA-256
  `a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353`;
- 470 contours, 13 holes, 25,747 points, 2,152 removed contours, exact EOF;
- ASCII comma-list SHA-256 of the 337 selected slots
  `24dad9513353d3cf165101199c4514830b5cbcbfe08ce2a100c469bc0eade813`,
  first slot 20 and last slot 459;
- five byte-identical fixed-probe runs.

The independently frozen threshold mutation changes
`spiral_mode: 0 -> 1` and `bottom_shell_layers: 3 -> 21`, retains committed
`bottom_shell_thickness=0`, and has:

- Task 22G input 1,154,017 bytes / SHA-256
  `f19e168ee3ad5d6a6c882f20bda26d8f0aedeca793fe38be7258b19abd7f4f8c`;
- modes `21/0/0/439`, 2,717 contours, 128 holes, 68,852 points, exact EOF;
- 336 selected slots, first 21, last 459, ASCII list SHA-256
  `39a5798f846adf8d41e76c8d6888c6afa6fc9f0d81e3b463989ecc2bb2cd5bc3`;
- Task 22H output 674,201 bytes / SHA-256
  `4b64a4e70bfceabf414572f6dbe13903245612908cbaf2d12985b6c1ed440214`;
- 569 contours, 127 holes, 41,012 points, exact EOF, and five identical runs;
- Regular slot 20 unchanged at 16,689 bytes / SHA-256
  `e408ee218b9fa4a2dd09da1254bc4a6e74c1d5190ca54ba5156558a5f9292730`.

Native tests independently construct both Option mutations with the existing
`KsrArchive` helper. Playwright independently constructs the three-Option
mutation from the committed archive with `fflate`. No new Task 22H production
or Task 22H test reads ignored oracle files or the reference G-code.

## TDD package sequence

### Package 0: register complete checkpoints and preserve a real selector RED

Allowed paths are exactly:

- `crates/ares-core/Cargo.toml`;
- `crates/ares-core/src/lib.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/closing.rs`;
- `crates/ares-core/src/project_slice/task22g_oracle.rs`;
- `crates/ares-core/src/project_slice/task22h_oracle.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/largest_contour_fixture.rs`;
- `crates/ares-wasm/Cargo.toml`;
- `crates/ares-wasm/src/lib.rs`;
- `crates/ares-wasm/tests/browser/index.html`;
- `crates/ares-wasm/tests/browser/package.json`;
- `crates/ares-wasm/tests/browser/package-lock.json`;
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`;
- `.github/workflows/tier1.yml`.

1. Add native committed-project, three-Option mutation, and threshold-21
   mutation assertions against the planned but absent Task 22H hook.
   Both mutations, their exact Option replacements, G/H hashes, mode
   histograms, distinct 337/336-slot digests, unchanged Regular slot 20,
   counts, EOF, and repeatability are frozen before implementation.
2. Add Playwright assertions against the absent H input/output bindings. The
   test reads the committed 3MF, uses pinned test-only `fflate=0.8.3` to apply exactly
   the approved three replacements, and passes the complete mutated archive to
   WASM. Run focused compile/browser commands and record those REDs.
3. Add the marker-parameterized internal encoder, H marker wrapper, non-default
   feature, and two byte-only H conformance bindings, initially sourcing both
   checkpoints from the already released post-closing preparation. Retarget
   `closing.rs` read-accessor cfgs from the removed G feature to
   `test || task22h-browser-oracle`; keep the fixed-G native wrapper test-only.
4. Rerun. The committed baseline may be green because all modes are Regular.
   Both unchanged native mutation oracles and the non-vacuous browser mutation
   must fail with complete marker-only post-closing geometry instead of their
   fixed 427,465-byte and 674,201-byte H outputs. Record those behavior REDs and
   freeze all test bytes.
5. Build default and H-feature WASM into isolated target/output directories.
   Inspect both generated JS bindings: default contains neither H export;
   H-feature contains exactly `task22hBrowserInputOracle` and
   `task22hBrowserOracle`; neither contains `task22gBrowserOracle`.
6. Run Task 22G native regression, fixture integrity, parser/KAT, fmt, Clippy,
   LOC, and forbidden-source-split checks.
7. Obtain fresh Package 0 specification and quality approval.

The pass-through is a temporary test-only checkpoint seam. Package B must wire
the source selector before any candidate freeze; it is not a legacy fallback.

### Package A: exact polygon area and pure ExPolygon selector

Allowed paths are exactly:

- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/polygon.rs`;
- `crates/ares-core/src/geometry/expolygon.rs`;
- `crates/ares-core/src/geometry/tests/polygon.rs`;
- `crates/ares-core/src/geometry/tests/expolygon.rs`.

1. Register area tests for fewer-than-three points, both orientations, and a
   large-coordinate operation-order vector; record the missing-method RED.
2. Implement only exact `Polygon::area`; run the focused tests and Task 22F/G
   geometry regressions.
3. Register selector tests for empty/single identity, strict signed maximum,
   negative absolute-area decoy, distinct first tie, contour-vs-net area,
   selected holes/order, and all-nonpositive invariant; record the
   missing-helper RED.
4. Implement only the pure helper. Do not touch raw slicing-mode code or
   perform orientation repair.
5. Run focused tests, complete geometry nextest, fmt, Clippy, WASM check, LOC,
   and hardcoding audits.
6. Obtain fresh Package A specification and quality approval.

### Package B: mode-gated post-closing project stage

Allowed paths are exactly:

- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/closing.rs`;
- `crates/ares-core/src/project_slice/largest_contours.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/largest_contours.rs`;
- `crates/ares-core/src/project_slice/tests/largest_contour_fixture.rs`.

1. Register mixed-mode/multi-object/multi-volume tests against the absent stage
   and record the compile RED.
2. Cover Regular, EvenOdd, Positive identity; PLC empty/single/multiple;
   source/transform/plan/volume/ordinal/type/layer/mode/hole/order retention;
   and per-layer independence.
3. Implement the narrow in-place traversal and post-largest preparation seam.
4. Switch the public incomplete lifecycle and Task 22H checkpoint from
   post-closing preparation to post-largest preparation. Keep native Task 22G
   on post-closing.
5. Rerun both unchanged mutation oracles. They must become green only now.
   Compare each Task 22G input with its Task 22H output and prove exact
   thresholds, modes, distinct 337/336-slot digests, unchanged Regular slot 20,
   layer counts, and selection facts.
6. Run Task 22E mode, Task 22F union, Task 22G closing, public lifecycle,
   geometry, fmt, Clippy, WASM, LOC, and hardcoding regressions.
7. Obtain fresh Package B specification and quality approval.

### Package C: unchanged complete native and browser oracle promotion

Expected constants and parsers remain byte-for-byte those frozen in Package 0.

1. Run the already-registered committed and both mutated native oracles twice and
   preserve Package 0 RED evidence.
2. Run fresh wasm-bindgen 0.2.121 generation and Playwright twice over both the
   real committed 3MF and the complete non-vacuous mutated 3MF. Verify G input
   digest/modes/337-slot set and H output digest/counts/EOF in WASM. The
   threshold-21 mutation remains native because the browser's three-Option
   mutation already executes the selector non-vacuously.
3. Correct only source-proven implementation defects. Do not change expected
   values because Ares disagrees.
4. Prove committed fixture hashes are unchanged; no new Task 22H production or
   Task 22H test reads the reference G-code; isolated default bindings expose
   no H hook; H bindings expose both H-named hooks but no old G export; and the
   public API remains incomplete.
5. Run focused Task 22A-H regressions and obtain fresh Package C specification
   and quality approval.

### Package D: closure, six-axis review, docs, and release

1. Freeze the exact implementation manifest and normalized patch digest.
2. Run structural, hardcoding, provenance, fixture, and platform audits.
3. Run the complete local verification matrix.
4. Start one independent read-only six-axis reviewer on requirement
   completeness, logical correctness, edge cases, code quality, test coverage,
   and actual execution results.
5. Receive a prioritized P0-P3 fix list. Only the main thread repairs; rerun
   affected/full verification, refreeze, and return the same candidate to the
   same reviewer. Repeat until all six axes pass or a reproduced blocker is
   documented.
6. Obtain three fresh whole-candidate reviews on unchanged bytes:
   specification compliance, code quality, and direct default-model review.
7. Update architecture and roadmap only after implementation approval; obtain
   independent documentation approval.
8. Rerun the docs-inclusive full matrix and exact 26-path manifest audit.
9. Stage exactly the approved paths, create a Conventional Commit, push
   normally, verify local/tracking/direct remote identity, and monitor exact-SHA
   Tier-1 through all five jobs.
10. Begin Task 22I immediately; do not complete the persistent goal.

## Focused and full verification matrix

Use `cargo nextest run`, never `cargo test`, as the default Rust runner.

Focused filters cover all `task22h_` tests and all affected Task 22E-G tests.
Final verification includes:

```text
cargo fmt --check
git diff --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets
cargo nextest run -p ares-core
cargo nextest run --workspace
cargo check -p ares-core --target wasm32-unknown-unknown
cargo check -p ares-wasm --target wasm32-unknown-unknown
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --target-dir target/wasm-default
fresh wasm-bindgen 0.2.121 default generation to target/wasm-browser-default
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --features task22h-browser-oracle --target-dir target/wasm-task22h
fresh wasm-bindgen 0.2.121 H-feature generation to target/wasm-browser
generated-JS exact export presence/absence audit
committed and non-vacuous mutated real-3MF Playwright exact-oracle tests
```

Record nextest run IDs, pass/skip counts, command exit codes, exact tested
manifest, and per-file hashes. Any platform digest disagreement blocks release
for numeric source tracing; platform-specific expected output is forbidden.
The isolated target directories prevent a prior feature build from satisfying
the default no-export check through shared Cargo artifacts.

## Structural, provenance, and hardcoding audits

The final candidate must prove:

- every Rust production/test file is below 400 physical LOC;
- no Task 22H `include!` or `include_bytes!` source split;
- no unsafe, FFI, filesystem/process/thread, native dependency, platform
  branch, alternate geometry engine, or host-specific oracle;
- no fixture filename/hash/G-code read in production and no expected geometry
  or count table in production;
- no new Task 22H production or Task 22H test opens the reference G-code;
  audit this over the exact changed manifest/diff because older unrelated CLI
  and config-export tests intentionally consume that fixture;
- no literal production spiral threshold, layer index, mode rewrite, 337, 470,
  427465, or expected digest branch;
- exact signed contour area and strict first-tie behavior;
- selector runs only after closing and only once before Task 22I;
- all mode and threshold behavior is still derived from resolved 3MF Options;
- obsolete executable Orca source-pinning tests remain absent;
- committed fixtures and released Task 22G production blobs change only at
  explicitly planned narrow seams;
- `git diff --check`, exact path manifest, per-file hashes, and normalized
  composite digest pass.

## Mandatory independent review loop

After implementation and the complete local matrix, start one independent
read-only reviewer thread. Give it the exact manifest/digest, specification,
plan, source boundaries, test inventory, and execution evidence. It must issue
separate verdicts for:

1. requirement completeness;
2. logical correctness;
3. edge cases;
4. code quality;
5. test coverage;
6. actual execution results.

The reviewer returns P0-P3 findings and a concrete fix list. It may not edit
files. The main thread applies only source-cited fixes, reruns affected and
full verification, freezes a new digest, and sends the candidate back to the
same reviewer. Continue until all six axes pass or a concrete external blocker
is reproduced.

After the six-axis loop passes, obtain fresh whole-specification, whole-quality,
and direct default-model approvals on unchanged bytes. Any code or test edit
invalidates all three.

## Documentation and release

Only after implementation approval:

1. update `option-parity-v4.md` with exact post-closing ownership, numeric
   semantics, baseline/mutation facts, and Task 22I deferrals;
2. update `roadmap.md` to mark Task 22H implemented while full G-code parity
   remains incomplete and preserve Task 22I's exact source boundary;
3. obtain independent documentation approval;
4. rerun the complete docs-inclusive matrix and manifest audits;
5. commit with Conventional Commits, expected subject
   `feat(slicing): port largest-contour selection`;
6. push normally, verify refs, and monitor the exact-SHA Tier-1 run until all
   five jobs pass.

## Stop conditions

- Any fixed-source, signed-area operation-order, stage-order, or oracle
  ambiguity stops implementation for source audit.
- Any mutated-oracle disagreement is traced before production or expected
  values change.
- Any path outside the 26-path manifest stops work for plan amendment and fresh
  review.
- Any Rust file reaching 400 LOC is split with a real module before continuing.
- A Tier-1 platform mismatch is not normalized with a platform oracle.
- Never add a Task 22H production or Task 22H test read of the reference
  G-code; existing unrelated tests are outside this slice.
- Never claim full KSR parity or complete the persistent goal after Task 22H.

## Gate checklist

- [ ] Committed-project oracle receives two independent approvals
- [ ] Both 3MF Option-mutation oracles each receive two independent approvals
- [ ] Exact spec/plan hashes frozen
- [ ] Fixed-source/spec approval
- [ ] Ares/plan approval
- [ ] Direct default-model spec/plan approval
- [ ] Package 0 native/browser registration and real mutation RED approved
- [ ] Package A geometry RED/GREEN approved
- [ ] Package B project-stage RED/GREEN approved
- [ ] Package C unchanged native/browser complete oracles approved
- [ ] Full native/WASM/browser matrix green
- [ ] Structural/provenance/hardcoding audits green
- [ ] Same-thread six-dimensional fix/re-review loop passes
- [ ] Whole specification, quality, and default-model approvals
- [ ] Architecture/roadmap docs reviewed
- [ ] Final docs-inclusive matrix and exact manifest green
- [ ] Conventional commit pushed normally
- [ ] Exact-SHA Tier-1 green on all five jobs
- [ ] Task 22I started; persistent goal remains active

**Status: DRAFT — implementation is forbidden until the exact specification
and plan receive all pre-implementation approvals.**
