# Task 22F Implementation Plan: Clipper 6 Closed Boolean, PolyTree, and Pre-Closing Union

## Status, fixed points, and success condition

This plan is a draft. No production or test implementation may begin until
the exact specification, this plan, and ARD-0024 receive fresh independent
fixed-source/spec, independent Ares/plan, and direct default-model approvals.

Fixed Ares baseline:

- branch: `codex/ksr-fdmtest-v4-parity`;
- commit: `645f5cb9e193750b8ffdbdf6e06e8829c7c210f4`;
- tree: `6faf7b46de9f6675427f44c175ecbb2a6be4c7c9`;
- Task 22E exact-SHA Tier-1 run: `29629113173`, green on formatting,
  Ubuntu, Windows, macOS, and WASM/browser.

Fixed Orca source:

- commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
- tree: `b62d6017ba1ac7cb986f70fd6844353c7a776549`;
- exact closed Boolean, PolyTree, wrapper, ExPolygon, project, and ordering
  dependencies are enumerated in the specification.

Task success is one safe pure-Rust closed Clipper 6 kernel, exact ordered Paths
and PolyTree output, exact two-pass `union_ex`, owned ExPolygons, and a
production-reachable per-volume/per-layer `PreClosing` project stage. It sorts
by released `VolumeOrdinal`, maps retained slicing mode to the fixed fill rule,
matches a one-time fixed-source KSR pre-closing oracle, and still returns
`ProjectSlicingIncomplete` publicly.

Task success does not include offset/closing, largest-contour selection,
simplification, cross-volume combination, surfaces, toolpaths, or G-code.

## Immutable behavior ledger

Implementation and review are constrained by these facts:

- the bundled fixed source, not a floating version label, owns behavior;
- closed Boolean is one strongly connected minima/scanbeam/AEL/intersection/
  output/join/ownership state machine;
- all four closed operations and all four fill rules are implemented;
- project production exposes only `union_ex`;
- `union_ex` executes to Paths first and PolyTree second;
- PolyTree root children become contours, their immediate children become
  holes, and islands inside holes recurse as later ExPolygons;
- output point, ring, sibling, and ExPolygon order are observable;
- comparator-equivalent minima/intersections use the source-cited pure-Rust
  rewrite of audited MSVC STL 14.44 `std::sort` control flow, with the fixed
  Y-only comparator and no invented secondary key;
- the audited sort closure is
  `algorithm:233-237,7147-7152,8242-8404` and
  `__msvc_heap_algorithms.hpp:21-136`; the reusable Orca workflows establish
  a Windows/MSBuild path but do not themselves pin toolset 14.44;
- `loRange`, `hiRange`, exact full-range slope equality, fixed floating
  containment/area, and fixed floating rounding are normative;
- no alternate engine, C++ binding, unsafe pointer graph, or post-hoc
  containment/canonicalization is allowed;
- Orca runtime `ModelVolume::id()` is not the numeric 3MF leaf ID;
- released `VolumeOrdinal` is the portable per-object creation-order
  equivalent and retains filter gaps;
- project fill mapping is Regular/Positive -> NonZero, EvenOdd -> EvenOdd,
  PositiveLargestContour -> Positive;
- KSR is Regular/NonZero but requires later 0.049 mm closing and 0.0025 mm
  effective simplification, so this output is pre-closing only;
- every layer slot, including an empty result, remains present;
- production and tests never read the reference G-code for this package;
- executable tests never inspect or invoke Orca source.

## Working protocol

1. Work only from the fixed baseline and preserve unrelated user changes.
2. Read fixed Orca evidence with `git show`; never check out or modify the
   ignored Orca tree.
3. Use `apply_patch` for every tracked edit.
4. Follow strict RED -> inspect failure -> minimal GREEN -> refactor for each
   package. Tests are registered before their production seam.
5. Use Cargo Nextest, not `cargo test`, as the default runner.
6. Keep every Rust file below 400 physical lines throughout, not only at the
   end. Split with real modules before line 400.
7. Record commands, exit codes, counts, hashes, and skips in ignored
   `.superpowers/sdd/task22f-evidence.md`.
8. A package may touch only the exact approved manifest. A required path
   addition/removal amends this plan and invalidates approvals.
9. Parallel work is read-only unless paths have been explicitly partitioned.
   Shared dependent implementation packages are serial.
10. Do not commit package by package. Make one conventional Task 22F commit
    only after implementation, review loops, docs, and final verification.

## Pre-implementation exact-byte gate

After the documents are frozen:

1. record path, bytes, LF line count, and SHA-256 for the spec, plan, and ARD;
2. dispatch one read-only fixed-source/spec reviewer to rederive the complete
   closed dependency closure, input/range/rounding rules, every operation/fill
   branch, output/tree order, two-pass wrapper, ExPolygon recursion, and
   included/deferred boundary;
3. dispatch a separate read-only Ares/plan reviewer to verify typed arena
   identity, module partition, exact manifest, VolumeOrdinal semantics,
   project ownership, error mapping, TDD order, license artifacts, WASM
   safety, and release closure;
4. run the configured OpenCode default-model reviewer directly without `-m`,
   with runtime `task=deny` and `edit=deny`;
5. require literal `VERDICT: APPROVE` from all three;
6. recheck document hashes and tracked status after review.

Any document edit invalidates every verdict and restarts the gate.

## Exact planned tracked manifest

### Documentation and notices

Created before implementation:

- `docs/architecture/ard-0024-safe-indexed-clipper6-kernel.md`;
- `docs/superpowers/specs/2026-07-17-ksr-fdmtest-v4-task22f-clipper6-pre-closing-union.md`;
- `docs/superpowers/plans/2026-07-17-ksr-fdmtest-v4-task22f-clipper6-pre-closing-union.md`.

Created during implementation closure:

- `LICENSES/BSL-1.0.txt`;
- `LICENSES/Apache-2.0-WITH-LLVM-exception.txt`;
- `THIRD_PARTY_NOTICES.md`.

Modified after implementation approval:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

### Production geometry

Modified:

- `crates/ares-core/src/geometry.rs`;
- `crates/ares-core/src/geometry/polygon.rs`.

Created:

- `crates/ares-core/src/geometry/expolygon.rs`;
- `crates/ares-core/src/geometry/clipper.rs`;
- `crates/ares-core/src/geometry/clipper/types.rs`;
- `crates/ares-core/src/geometry/clipper/predicates.rs`;
- `crates/ares-core/src/geometry/clipper/input.rs`;
- `crates/ares-core/src/geometry/clipper/input/path.rs`;
- `crates/ares-core/src/geometry/clipper/input/bounds.rs`;
- `crates/ares-core/src/geometry/clipper/minima.rs`;
- `crates/ares-core/src/geometry/clipper/ordering.rs`;
- `crates/ares-core/src/geometry/clipper/engine.rs`;
- `crates/ares-core/src/geometry/clipper/winding.rs`;
- `crates/ares-core/src/geometry/clipper/active_edges.rs`;
- `crates/ares-core/src/geometry/clipper/horizontals.rs`;
- `crates/ares-core/src/geometry/clipper/intersections.rs`;
- `crates/ares-core/src/geometry/clipper/output.rs`;
- `crates/ares-core/src/geometry/clipper/output/rings.rs`;
- `crates/ares-core/src/geometry/clipper/output/append.rs`;
- `crates/ares-core/src/geometry/clipper/output/joins.rs`;
- `crates/ares-core/src/geometry/clipper/output/join_points.rs`;
- `crates/ares-core/src/geometry/clipper/output/ownership.rs`;
- `crates/ares-core/src/geometry/clipper/output/fixup.rs`;
- `crates/ares-core/src/geometry/clipper/polytree.rs`.

### Production project stage

Modified:

- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/slicing_mode_intersections.rs`.

Created:

- `crates/ares-core/src/project_slice/pre_closing_unions.rs`.

### Tests

Modified:

- `crates/ares-core/src/geometry/tests.rs`;
- `crates/ares-core/src/project_slice/tests.rs`.

Created:

- `crates/ares-core/src/geometry/tests/expolygon.rs`;
- `crates/ares-core/src/geometry/tests/clipper.rs`;
- `crates/ares-core/src/geometry/tests/clipper/helpers.rs`;
- `crates/ares-core/src/geometry/tests/clipper/input.rs`;
- `crates/ares-core/src/geometry/tests/clipper/booleans.rs`;
- `crates/ares-core/src/geometry/tests/clipper/fill_rules.rs`;
- `crates/ares-core/src/geometry/tests/clipper/touching.rs`;
- `crates/ares-core/src/geometry/tests/clipper/polytree.rs`;
- `crates/ares-core/src/geometry/tests/clipper/options.rs`;
- `crates/ares-core/src/geometry/tests/clipper/large_coordinates.rs`;
- `crates/ares-core/src/project_slice/tests/pre_closing_unions.rs`;
- `crates/ares-core/src/project_slice/tests/pre_closing_fixture.rs`.

Ignored evidence, prompts, temporary fixed-source probes, probe outputs, and
build artifacts are not tracked manifest entries.

The planned tracked set contains 49 paths. Before implementation review,
enumerate the actual diff and require exact equality with this set. If a
listed split proves unnecessary, remove it by amending and re-reviewing this
plan rather than leaving an empty module.

## Module ownership and line budgets

Target approximately 250-350 physical lines per Rust file; 399 is a hard
ceiling after rustfmt.

- `clipper.rs`: provenance, public crate-private enums/options/error/facades.
- `types.rs`: typed IDs, edge/output/tree records, sentinelless enums, safe
  two-entry mutation helper.
- `predicates.rs`: fixed floating area/point-in-polygon, exact slope equality,
  TopX, intersection, rounding, and range constants.
- `input.rs`: path collection and shared normalization orchestration.
- `input/path.rs`: one-path cleanup and edge initialization.
- `input/bounds.rs`: local-minimum discovery, bound traversal, horizontal
  reversal, and LML construction.
- `minima.rs`: minima/scanbeam storage, reset, insertion orchestration.
- `ordering.rs`: source-cited MSVC STL 14.44 insertion, median partition,
  introsort-budget, and heap-fallback control flow used by both Y-only sorts.
- `engine.rs`: execution state, lifecycle, operation dispatch, ordered Paths
  and PolyTree entry points.
- `winding.rs`: winding count, fill rules, and contribution.
- `active_edges.rs`: AEL/SEL insert/delete/swap/promote operations.
- `horizontals.rs`: horizontal and maxima processing.
- `intersections.rs`: SEL sort, intersection list, adjacency fix, and edge
  intersection state transitions.
- `output.rs`: output module registration and shared output helpers.
- `output/rings.rs`: OutRec/OutPoint allocation, free-list, ring insertion,
  duplication, and area.
- `output/append.rs`: local min/max and append/redirection behavior.
- `output/joins.rs`: common-edge traversal and join orchestration.
- `output/join_points.rs`: horizontal/nonhorizontal point-ring joining.
- `output/ownership.rs`: containment, FirstLeft, split/merge parent repair.
- `output/fixup.rs`: duplicate/collinear cleanup and ordered Paths build.
- `polytree.rs`: tree construction and exact ExPolygon recursion.
- `expolygon.rs`: owned contour/hole domain only.
- `pre_closing_unions.rs`: ordinal sort, fill mapping, per-layer union, owned
  stage wrappers, and error mapping.

Do not create a second general abstraction around this one engine. Rust
methods may be implemented across functional modules without `include!`.

## Error contract

`geometry::clipper` has one small internal error: coordinate outside the fixed
allowed range.

Out-of-range coordinates are expected only at the external project boundary
and map to `SliceError::InvalidInput` with an option-independent geometry
message. Internal arena inconsistency is a bug and remains an invariant; do
not turn every private link traversal into defensive fallback logic.

No geometry error falls back to raw polygons, another fill rule, a simplified
union, or the legacy pipeline.

After input succeeds, internal execution preserves the fixed Boolean success
contract. If the state machine reports `false`, the wrapper disposes output
state and returns empty Paths/PolyTree because the fixed ClipperUtils wrapper
ignores the Boolean. It does not manufacture a project error or raw-loop
fallback. Private arena inconsistency remains a bug, not this source-level
failure case.

## Oracle protocol

Before Package D production wiring:

1. create an ignored one-time probe against exactly the fixed Orca Clipper
   source;
2. feed it reviewed synthetic vectors and the exact ordered Task 22E KSR raw
   polygon encoding;
3. emit operation/fill/options, ordered point lists, complete tree topology,
   and the exact versioned little-endian KSR byte stream defined by the
   specification;
4. include shuffled 35-minima and 36-intersection Windows vectors that cross
   the MSVC 32-element threshold, record complete Paths, and prove provider,
   coordinate, and EdgeId order do not substitute for the fixed sort;
5. independently verify the probe source/toolchain identity and input digest;
6. first compare independently implemented C++ and Rust encoders on nested and
   empty hand-written structures, then freeze complete expected data as Rust
   literals or reviewed SHA-256 stage hashes;
7. delete or retain the probe only in ignored evidence; never commit it;
8. prove committed tests do not read Orca, the probe, or reference G-code.

The Ares implementation cannot generate its own expected values. An oracle
disagreement stops the package for source tracing; expected output is not
updated merely to match Ares.

## TDD package sequence

Packages are serial because they share one mutable state machine. Read-only
source audits and reviews may run in parallel.

### Package A: domain, typed identity, predicates, and input

1. Register `expolygon`, `input`, and `large_coordinates` tests first.
2. Run the exact Task 22F filter and record compilation RED for absent types
   and facade.
3. Add Polygon ownership access, ExPolygon, public crate-private vocabulary,
   typed IDs/records, exact predicates/rounding, and input normalization/LML.
4. Run Package A focused tests until GREEN.
5. Run geometry tests, fmt, strict core Clippy, native core check, and core
   WASM test check.

Required RED/GREEN assertions include empty/degenerate/mixed paths, duplicate
endpoint, adjacent duplicate, collinear toggle, low/full range, positive and
negative range limits, fixed rounding, and stable edge identity. Range tests
must prove fixed ordering by contrasting an out-of-range coordinate in a path
reduced below three initial candidates (ignored) with the same coordinate in a
three-candidate path (error). It must also freeze retained tombstone edge IDs,
rejected-path slot rollback, and monotonic full-range mode after a later-flat
path has crossed `loRange`.

### Package B: complete closed sweep through ordered Paths

The sweep and output graph are one GREEN boundary. Intermediate commits may
compile, but no partial sweep package is called behaviorally complete.

1. Reconcile the already registered RED with the independently approved final
   oracle before production implementation: freeze all corrected 37-case
   ordered outputs, replace old creation-sequence names/expectations, encode
   consumed second-Execute plus `Clear()`/re-add behavior, and add direct
   35-minima and 36-intersection permutation vectors. Add separately reviewed
   ordering-helper vectors for the `>=42` Tukey-ninther branch and an
   adversarial input that exhausts the introsort budget; test-only branch
   evidence must prove both ninther and heap fallback are reached. The
   42-element threshold follows the fixed helper's inclusive-last
   `_Count = N - 1` comparison.
2. Run the exact Task 22F filter and require RED only at absent Package B
   execution/ordering seams. A small equal-key test is not sufficient RED.
3. Remove the provisional Package A `source_sequence` fields, counter,
   snapshots, tuple `sort_by_key`, and stable-order expectations. Implement
   the audited fixed MSVC sort helper, reset/execute, minima
   insertion, winding/contribution, AEL/SEL, horizontals, intersection
   sorting/fixup, maxima, and edge promotion. The Y-only comparators receive no
   secondary key. Freeze the exact 35/36 permutations before proceeding to
   ordinary Boolean GREEN.
4. Continue in the same package through ring allocation/free-list,
   append/redirection, common-edge joins, containment, FirstLeft repair,
   output cleanup, orientation, and ordered Paths. Do not make a partial
   output helper GREEN with a geometry special case.
5. Require every operation/fill vector, equal-key vector,
   touch/coincident/horizontal vector, ReverseSolution toggle, and
   PreserveCollinear toggle to match complete ordered oracle coordinates.
6. Run Package A-B, geometry, fmt, strict Clippy, native, and core WASM gates.

### Package C: PolyTree and two-pass ExPolygon union

1. Register nested-tree and two-pass tests before tree construction/facade.
2. Record RED against the absent PolyTree/`union_ex` result.
3. Implement tree ownership and exact recursive ExPolygon conversion.
4. Implement `union_ex` as Paths pass followed by a fresh PolyTree pass.
5. Require exact contour/hole/island/sibling/start-point order and deterministic
   repeated execution.
6. Run all Task 22F geometry tests plus the standard intermediate gates.

### Package D: project ordinal and pre-closing union

1. Complete and verify the one-time fixed-source KSR oracle before production
   wiring.
2. Register shuffled-ordinal, fill-mapping, empty-layer, error-mapping, owned
   lifecycle, and real KSR acceptance tests.
3. Record compilation RED against the absent `PreClosing` stage.
4. Add only the ordinal sort, exhaustive fill projection, owned wrappers,
   per-layer `union_ex`, and project traversal.
5. Do not consume closing, resolution, largest, or cross-volume behavior.
6. Require synthetic and KSR acceptance GREEN, then rerun Task 22A-E,
   mesh/project, fmt, strict Clippy, native, and both WASM checks.

### Package E: closure

1. Run the focused/full matrices and structural audits below.
2. Freeze sorted per-file hashes and a composite candidate digest.
3. Run the mandatory independent six-dimensional review/fix/re-review loop.
4. Obtain fresh whole-spec, whole-quality, and direct default-model approvals.
5. Only after implementation approval, add license notices and update
   architecture/roadmap docs.
6. Review docs, rerun the docs-inclusive matrix, commit, push, and monitor the
   exact-SHA Tier-1 run.
7. Start the ClipperOffset/closing package; do not mark the persistent goal
   complete.

## Focused and full verification matrix

Use Cargo Nextest:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22f_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22e_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22d_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22c_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22b_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22a_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(geometry)'
cargo +1.91.0 nextest run -p ares-core -E 'test(mesh_slicer)'
cargo +1.91.0 nextest run -p ares-core -E 'test(project_slice)'
cargo +1.91.0 nextest run -p ares-core
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check --workspace --all-targets
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown --tests
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown --tests
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
```

Generate fresh `wasm-bindgen --target web` output and run the committed
real-3MF Playwright test with bundled workspace dependencies. Missing external
dependencies are concrete blockers, never implicit passes.

## Structural, provenance, and hardcoding audits

1. enumerate every repository Rust file and require maximum physical LOC
   `<400`;
2. require no production source-splitting `include!`/`include_bytes!` macros;
3. require no unsafe, raw pointer, `Rc<RefCell>`, filesystem, native thread,
   Rayon, TBB, FFI, native dependency, platform branch, or mutable global in
   the new path;
4. require no alternate clipping dependency or legacy geometry call;
5. run a scoped audit over `geometry/clipper` and reject every host slice-sort
   call (`.sort`, `.sort_by`, `.sort_by_key`, and all unstable variants);
   minima and intersections may call only `ordering.rs`;
6. require no executable test opening/parsing/hashing/compiling Orca source;
7. require no Task 22F reference-G-code read;
8. inspect constants and branches for fixture names, hashes, dimensions,
   coordinates, layer counts, and oracle-specific shortcuts;
9. prove the fixture 3MF/G-code and Task 22E raw/config hashes unchanged;
10. prove all expected coordinate vectors came from recorded fixed-source
   oracle inputs, not the implementation under test;
11. require BSL text and notice content exact and scoped to the component;
12. run `git diff --check`;
13. compare the exact tracked diff with the 49-path manifest;
14. freeze sorted per-file SHA-256 values and the composite digest.

## Mandatory independent review loop

After Package E, dispatch one independent read-only reviewer with the exact
baseline, manifest, digest, spec/plan/ARD, fixed source identity, oracle
evidence, and fresh verification logs. Require separate verdicts for:

1. requirement completeness;
2. logical correctness;
3. boundary and edge cases;
4. code quality;
5. test coverage;
6. actual execution results.

The reviewer returns one prioritized fix list with paths, evidence, and rerun
requirements. Only the main thread edits. After fixes, rerun affected and full
gates, freeze a new digest, and send the revised candidate to the same reviewer
thread. Repeat until all six dimensions pass with an empty fix list or a
concrete external blocker is reproduced.

Then obtain three fresh whole-candidate approvals:

- specification implementation compliance;
- code quality and maintainability;
- direct default-model implementation review with task/edit denial.

Any code or test edit invalidates those approvals.

## Documentation and release

Only after implementation approval:

1. change ARD-0024 from Proposed to Accepted without changing its decision;
2. add the canonical BSL-1.0 and Apache-2.0 WITH LLVM-exception texts plus a
   concise Clipper/MSVC-sort third-party notice;
3. update `option-parity-v4.md` with actual engine/project ownership, exact KSR
   pre-closing facts, and explicit deferrals;
4. update `roadmap.md` to mark Task 22F pre-closing union implemented while
   full G-code parity remains incomplete;
5. record the next exact fixed-source ClipperOffset, `offset_ex`, `offset2_ex`,
   KSR 0.049 closing, largest, and simplify boundary;
6. obtain independent documentation approval;
7. rerun the docs-inclusive local matrix and exact manifest/hash audits;
8. stage exactly the approved 49 paths;
9. commit with Conventional Commits, expected subject
   `feat(geometry): port Clipper closed boolean tree`;
10. push normally without amend, force, squash, or history rewrite;
11. verify local HEAD, tracking ref, and direct remote SHA are identical;
12. monitor the exact-SHA Tier-1 run until all five jobs pass;
13. append release evidence and immediately begin the offset/closing slice.

## Stop conditions

- A fixed-source or oracle ambiguity stops implementation for source audit.
- A required path outside the manifest stops implementation for plan amendment
  and fresh approvals.
- An arena representation that cannot preserve identity/order stops for ARD
  amendment; unsafe or another engine is not an automatic escape.
- A test premise contradicted by fixed source is corrected in docs/tests, not
  hidden with production fallback.
- An oracle mismatch is traced to source semantics before either side changes.
- A Tier-1 failure is diagnosed on the exact SHA before release is claimed.
- Never amend, squash, force-push, or rewrite released Task 22A-E commits.
- Never mark the persistent user goal complete while normalized reference
  G-code parity remains absent.

## Gate checklist

- [ ] Exact spec/plan/ARD hashes frozen
- [ ] Independent fixed-source/spec APPROVE
- [ ] Independent Ares/plan APPROVE
- [ ] Direct default-model spec/plan/ARD APPROVE
- [ ] Package A domain/input RED then GREEN
- [ ] Package B complete closed sweep/output RED then GREEN
- [ ] Package C PolyTree/two-pass union RED then GREEN
- [ ] Fixed-source KSR pre-closing oracle independently frozen
- [ ] Package D project pre-closing RED then GREEN
- [ ] Package E full matrix and structural audits green
- [ ] Exact implementation manifest/digest frozen
- [ ] Six-dimensional fix/re-review loop passed
- [ ] Whole spec, quality, and default-model reviews approved
- [ ] License, architecture, and roadmap docs reviewed
- [ ] Final docs-inclusive local matrix green
- [ ] Conventional commit pushed
- [ ] Exact-SHA Tier-1 green on all five jobs
- [ ] Offset/closing source slice recorded and started

**Status: DRAFT — implementation is forbidden until fresh independent and
default-model reviewers approve these exact specification, plan, and ARD
bytes.**
