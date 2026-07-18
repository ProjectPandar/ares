# Task 22D Implementation Plan: Open-Polyline Repair and Loop Return

## Status, fixed points, and success condition

This plan is a draft. No production/test implementation may begin until the
exact spec and plan bytes receive fresh independent upstream/spec,
independent Ares/plan, and direct default-model approvals.

Fixed Ares baseline:

- branch: `codex/ksr-fdmtest-v4-parity`;
- commit: `8c07319a5ac1f9660324ef53172ffe95d2b53230`;
- tree: `516b35098d2ae537ee7e148689d1e5bbba5be2f1`;
- Task 22C exact-SHA Tier-1 run: `29616822593`, green on format, Linux,
  WASM/browser, macOS, and Windows.

Fixed OrcaSlicer source:

- commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
- tree: `b62d6017ba1ac7cb986f70fd6844353c7a776549`;
- semantic boundary: `TriangleMeshSlicer.cpp:1163-1381,1414-1480` plus the
  exact adjacent types/helpers cited by the specification.

Task success is a private, production-reachable
`ChainedLayer -> LoopedLayer` rewrite that executes exact(false), exact(true),
gap(false), gap(true), returns only polygons, preserves all fixed source quirks,
and leaves the real zero-open fixture byte-identical. The package is not final
G-code parity and must end with an explicit next source boundary.

## Immutable behavior ledger

Implementation and reviews are constrained by these facts:

- exact passes sort unconsumed entries by cached length without recomputing;
- gap passes recompute length before each sort;
- all equal-length seed ties use original open index;
- exact lookup uses `Vertex(n) -> +n`, `Edge(n) -> -n`, including the zero
  collision;
- exact candidate ties use original index then start before end;
- exact(false) appends points and length but never advances seed end identity;
- exact(true) advances seed end and keeps that record usable if the expanded
  seed later fails;
- exact candidate endpoints are omitted without coordinate validation;
- exact closure uses tagged identity, pops the terminal point unconditionally,
  drops fewer than three points, and normalizes negative area only in the
  reversal-enabled pass;
- gap lookup chooses strict-nearest distance below the integer radius, then
  original index, then start before end;
- nominal 2 mm scales to exactly 2,000,000 normal units and, due to the fixed
  floating-division/truncation behavior, 199,999 large-bed units;
- gap closure follows the source's closure/candidate/30% branch order exactly;
- a nonzero gap keeps both endpoints; only coordinate equality removes a
  joining or closing duplicate;
- gap cached length and tagged end are not updated while a seed absorbs
  candidates;
- reversed gap winding normalization requires more than one joined source
  polyline;
- failed seeds are restored, absorbed candidates stay consumed/empty, and all
  residual opens are dropped after the fourth pass;
- no Option, public error, legacy fallback, fixture-specific branch, source
  pinning test, or reference-G-code read is added.

## Workspace discipline and evidence

1. Confirm clean tracked status, baseline SHA/tree, branch tracking ref, and
   committed fixture hashes before every package.
2. Preserve ignored `.superpowers/sdd/task22d-evidence.md` as the only RED,
   GREEN, manifest, review, CI, and oracle ledger.
3. Verify upstream citations with read-only `git -C OrcaSlicer show`; never
   checkout, build-modify, or commit the ignored source tree.
4. Use `apply_patch` for source/test/docs edits. Do not overwrite unrelated
   user work.
5. Record exact commands, exit codes, counts, hashes, and unexpected skips.
6. A package may touch only its approved manifest. A newly required path stops
   work for a plan-manifest amendment and fresh approvals.
7. Do not commit package-by-package. One conventional Task 22D commit is made
   only after code, tests, reviews, docs, and final matrix pass.

## Pre-implementation exact-byte gate

After the plan is frozen:

1. record bytes, physical lines, and SHA-256 for the spec and plan;
2. dispatch a read-only upstream/spec reviewer to re-derive every exact/gap
   rule and the stopping boundary from the fixed source;
3. dispatch a read-only Ares/plan reviewer to verify privacy, ownership,
   complexity, WASM portability, manifests, RED order, and release closure;
4. run the configured default-model review directly, with runtime task/edit
   denial and no model override;
5. require literal `VERDICT: APPROVE` from all three;
6. verify `git status` and document hashes are unchanged after review.

Any document edit invalidates all approvals.

## Exact planned tracked manifest

Documentation created before implementation:

- `docs/superpowers/specs/2026-07-17-ksr-fdmtest-v4-task22d-open-polyline-repair.md`;
- `docs/superpowers/plans/2026-07-17-ksr-fdmtest-v4-task22d-open-polyline-repair.md`.

Production files modified:

- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/chaining.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/state.rs`.

Production files created:

- `crates/ares-core/src/mesh_slicer/chaining/exact.rs`;
- `crates/ares-core/src/mesh_slicer/chaining/gaps.rs`;
- `crates/ares-core/src/mesh_slicer/chaining/gaps/spatial.rs`;
- `crates/ares-core/src/project_slice/looped_intersections.rs`.

Core test registration/files modified:

- `crates/ares-core/src/mesh_slicer/tests/chaining.rs`;
- `crates/ares-core/src/mesh_slicer/tests/chaining/open.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/integration.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs`.

Test files created:

- `crates/ares-core/src/mesh_slicer/chaining/exact/tests.rs`;
- `crates/ares-core/src/mesh_slicer/chaining/gaps/tests.rs`;
- `crates/ares-core/src/mesh_slicer/chaining/gaps/spatial/tests.rs`;
- `crates/ares-core/src/mesh_slicer/tests/chaining/loops.rs`;
- `crates/ares-core/src/project_slice/tests/looped_intersections.rs`;
- `crates/ares-core/src/project_slice/tests/looped_fixture.rs`;
- `crates/ares-core/src/project_slice/tests/looped_fixture/encoding.rs`.

Post-approval documentation modified:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

Every listed path is unconditional. Adding, removing, replacing, or omitting
any tracked path requires editing this plan, freezing new exact bytes, and
repeating all three pre-implementation approvals before implementation
continues.

## Module design before RED

### Parent ownership

`mesh_slicer/chaining.rs` remains the owner of `OpenPolyline` and
`ChainedLayer`. Add private consuming parts access and:

```text
LoopedLayer { polygons }
make_loops(ChainedLayer, Coord) -> LoopedLayer
```

The parent calls the two exact and two gap passes in fixed order. Child
algorithms directly access ancestor-private fields; no field becomes public.
`LoopedLayer` exposes only crate-private polygon inspection required by the
project wrapper and tests.

### Exact index

`chaining/exact.rs` owns:

- an `i64` `ReferenceKey` conversion with the zero collision;
- `EndpointSide::{Start, End}` with start-first deterministic ordering;
- endpoint records carrying original polyline index and side;
- an ordered key-to-record structure;
- deterministic unconsumed seed sorting;
- `chain_open_polylines_exact`.

Records are grouped/sorted once. Reversal-enabled end movement updates the
seed's end record without rebuilding or scanning every polyline. Candidate
records may remain structurally present after consumption only when the
consumed flag makes them unreachable; no stale unconsumed endpoint may remain
under an old key.

Expected bound: `O(n log n + a)` time for index/sort plus attached/skipped
records and `O(n)` auxiliary memory. No per-identity queue or all-pairs scan.

### Gap index

`chaining/gaps/spatial.rs` owns a request-local deterministic integer grid:

- cell size equal to the positive integer search radius;
- `i128` cell coordinates using Euclidean division for negative inputs;
- fixed 3-by-3 neighboring-cell traversal that covers every point inside the
  radius;
- endpoint entries carrying insertion point, original polyline index, and
  side;
- exact record removal and reinsertion;
- query minimum by exact `(distance_squared, original_index, side)`;
- strict exact-integer `distance_squared < radius_squared` acceptance.

Coordinates subtract in `i128`. Component bounds reject obvious misses; values
that may be inside the radius use exact bounded `u128` squares, so strict
equality and nearest order stay correct at extreme absolute coordinates. Only
the bounded closing distance converts to `f64` for the 30% square root. The
grid stores `O(n)` entries and does not allocate during a query. A dense cell
may require examining all entries in that cell, which is the documented
source-equivalent worst case.

`chaining/gaps.rs` owns recomputed-length sorting, the fixed closure heuristic,
forward/reverse point attachment, lookup lifecycle, gap area gate, and
`chain_open_polylines_close_gaps`.

### Area helper

Add only a private point-vector signed-area helper at the repair boundary. It
implements the fixed `MultiPoint.hpp:182-187` expression by taking each
coordinate sum/difference in `i128` before converting the safe intermediates to
`f64`. Open length likewise takes `i128` coordinate differences before the
norm. Neither becomes a `Polygon` method or general geometry API.

### Project state and wrapper

Retain the already selected `CoordinateScale` as a private
`ProjectSliceState` field. Update the two exhaustive test destructures.
`slice_project` obtains `max_gap_scaled = scale.checked_scale(2.0).expect(...)`,
chains raw objects, consumes them into looped objects, traverses every private
field, and returns `ProjectSlicingIncomplete`.

`project_slice/looped_intersections.rs` maps objects, volumes, and layer slots
without sorting or cloning. It retains the full print plan and volume metadata.

## Package A: Result ownership and request scale

### A.1 RED

Before production edits, add tests that require:

- `LoopedLayer` to retain existing polygons and drop an empty open set;
- residual opens to be absent from the returned type;
- project state to retain `CoordinateScale::Normal` and `LargeBed` for
  synthetic printable areas;
- `checked_scale(2.0)` to freeze 2,000,000 and 199,999 exactly.

Run the exact Task 22D filter and record compilation failures caused only by
missing `LoopedLayer`, `make_loops`, looped wrapper, or scale field.

### A.2 GREEN

Implement only storage, consuming access, scale retention, and an initially
empty orchestration shell sufficient for ownership tests. Do not implement
exact or gap behavior yet. Run the Package A tests, formatting, strict core
Clippy, and core WASM check.

## Package B: Exact identity joining

### B.1 Freeze synthetic inputs and establish RED

Add separate `exact/tests.rs` cases with hand-built `OpenPolyline` state. The
tests must freeze:

1. cached-length descending order and original-index ties;
2. same-direction forward attachment;
3. the exact(false) stale-end behavior after one and multiple candidates;
4. exact(true) start and end attachment, live-end movement, and later
   reachability of a failed expanded seed;
5. original-index then start-before-end candidate ties;
6. signed key mapping and `Vertex(0)`/`Edge(0)` lookup collision;
7. tagged closure rejecting cross-variant zero identities;
8. unconditional joining-point omission even when coordinates differ;
9. unconditional exact-closure terminal pop;
10. fewer-than-three drop;
11. negative-area reversal only in exact(true), with zero/positive area left
    unchanged;
12. failed-seed restoration and attached-candidate destruction;
13. extreme identity/coordinate arithmetic.

Update the released Task 22C extreme open-length regression in
`mesh_slicer/tests/chaining/open.rs` so its name and expected-value expression
describe exact `i128` difference before `f64` conversion. Preserve its
overflow-safety assertion and Task 22C prefix; do not relax or delete it.

At least one integration test constructs equivalent state through Task 22C
raw lines rather than relying only on internal constructors. Record genuine
behavioral failures before exact production code is added.

### B.2 Minimal implementation and review

Implement `exact.rs`, wire both exact passes, and make no gap changes. Run:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22d_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22c_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(mesh_slicer)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown --tests
```

Dispatch a read-only source/quality package review. Fix only listed Task 22D
defects, rerun, and obtain approval before Package C.

## Package C: Deterministic spatial lookup and gap repair

### C.1 Spatial RED then GREEN

Add separate spatial tests before implementation:

- strict `< radius` and equality rejection;
- 2,000,000/199,999 scale-specific boundaries;
- same cell, every neighbor direction, negative cells, cell boundaries, and
  extreme `i64` coordinates;
- extreme absolute coordinates whose exact component difference equals the
  normal or large-bed radius and must be rejected;
- removal, reinsertion, and consumed-record exclusion;
- equal-distance original-index/start-before-end ties;
- deterministic repeated results;
- dense-cell stress with fixed memory-entry count and no query insertion.

Implement only the private grid and make those tests green. Run format, strict
core Clippy, core WASM, and an optimized dense-case timing sanity check whose
result is evidence, not a flaky wall-clock assertion.

### C.2 Gap algorithm RED

Before wiring production gap passes, add hand-built cases for:

- recomputed rather than stale length sorting at the start of both passes;
- same-direction start attachment;
- reversal-enabled end attachment;
- coordinate-equal junction omission and nonzero bridge endpoint retention;
- strict candidate and own-closure radius boundaries under both scales;
- zero-distance vs nonzero-distance closure point retention;
- candidate absent closure without 30% test;
- closure strictly closer and passing 30%;
- closure strictly closer but failing 30%, therefore attaching;
- exact 30% equality failing the strict test;
- equal closure/candidate distances closing without the heuristic;
- candidate closer than closure still taking the source's close branch;
- reversal gate requiring more than one joined segment for area correction;
- failed reversed-seed end reinsertion;
- cached length/tagged end remaining stale within one pass;
- shorter-than-three drop and residual-open discard.

Record genuine failures against the still-missing algorithm.

### C.3 Minimal gap implementation and review

Implement `gaps.rs`, wire gap(false) then gap(true), and make all Package C
tests green. Dispatch a read-only upstream/edge-case review focused on branch
ordering, lookup lifecycle, strict thresholds, overflow, and orientation.
Apply its exact fix list, rerun the focused/core/WASM gates, and obtain approval.

## Package D: Project ownership and production reachability

### D.1 RED

Add a multi-object/multi-volume test with empty layer slots. Freeze complete
plan values, volume ordinal/type, layer count/order, and polygon order after
loop repair. Add a production API test proving the looped path is reached and
still returns `ProjectSlicingIncomplete` for admitted project input.

Record failure before `looped_intersections.rs` exists or before
`slice_project` consumes it.

### D.2 GREEN

Implement the project wrapper, retain request scale in state, update exhaustive
destructures, and replace production traversal of open/chained state with
closed looped state. Do not change public errors or any legacy path.

Run Task 22A/B/C/D filters, all project-slice tests, strict Clippy/format, both
WASM checks, and a release WASM build. Obtain a read-only ownership/reachability
review before Package E.

## Package E: Frozen real-fixture no-op acceptance

### E.1 Freeze independent constants

Before observing looped Ares output, place the already approved Task 22C
constants into the separate looped fixture test or re-export them without
changing their bytes:

- 460 layers;
- 3,288 closed polygons;
- zero input opens;
- 116,472 points;
- 2,190,993 encoded bytes;
- face SHA
  `6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`;
- semantic SHA
  `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`;
- released config-block size/hash.

The test first asserts zero Task 22C opens, then calls Task 22D. Its encoder is
test-only and does not call production normalization. It never reads the
reference G-code or upstream source.

### E.2 Run acceptance

Assert exact counts, representative layer polygon lengths, face/semantic byte
length and hashes, repeat-run equality, retained plan/volume ownership,
unchanged config block, and public incomplete error. Any mismatch stops work
for source/encoder diagnosis; expected hashes are never replaced from observed
Ares output.

## Package F: Full regression and structural closure

Run and record:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22d_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22c_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22b_[^:]*$/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22a_[^:]*$/)'
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

Run the existing real-3MF browser/WASM gate with bundled workspace
dependencies. A missing external dependency is recorded exactly; it is not
reported as a pass.

Structural audits:

1. enumerate all `.rs` files and require maximum physical LOC `<400`;
2. require no production `include!`, `include_bytes!`, or `include_str!`;
3. require no native thread, Rayon, unsafe, filesystem, platform branch, or
   mutable global state in the new path;
4. require no executable test opening/hashing/parsing Orca source;
5. require no Task 22D reference-G-code read;
6. require no production/project Task 22D call into old
   planning/segments/contours/pipeline APIs;
7. inspect constants and branches for fixture-specific production behavior;
8. verify fixture files/hashes unchanged;
9. run `git diff --check` and compare the exact tracked path set with this
   manifest;
10. freeze sorted per-file SHA-256 values and a composite candidate digest in
    the ignored ledger.

## Mandatory six-dimensional review loop

After Package F, dispatch one independent reviewer with read-only instructions
and the exact frozen manifest/digest. The same reviewer must report separately:

1. requirement completeness;
2. logical correctness;
3. edge cases;
4. code quality;
5. test coverage;
6. actual execution results.

It returns one prioritized fix list with paths, evidence, and required reruns.
The main thread alone edits. After fixes, rerun affected plus full gates, freeze
new hashes, and send the same reviewer the revised candidate. Repeat until all
six dimensions are PASS with an empty fix list or a concrete external blocker
is reproduced and recorded.

After that approval, obtain three fresh whole-candidate gates:

- specification implementation approval;
- code-quality/maintainability approval;
- direct default-model implementation approval with task/edit denial.

Any code/test change invalidates those approvals.

## Documentation and release

Only after code/test/whole-candidate approval:

1. update `docs/architecture/option-parity-v4.md` with the fixed source
   boundary, pass order, signed-key collision, deterministic tie rules,
   request-local scale, ownership, fixture no-op facts, and deferrals;
2. update `docs/roadmap.md` to mark Task 22D implemented while final G-code
   parity remains incomplete;
3. record the exact next source-cited rewrite boundary;
4. obtain independent documentation approval;
5. rerun the complete docs-inclusive matrix and structural audits;
6. stage exactly the approved manifest;
7. commit with a Conventional Commit such as
   `feat(slicing): repair project slice loops`;
8. push `codex/ksr-fdmtest-v4-parity` normally, without force;
9. verify local HEAD, tracking ref, and direct remote SHA are identical;
10. monitor the exact pushed SHA and require Tier-1 format, Linux,
    WASM/browser, macOS, and Windows success;
11. append release evidence and immediately begin the next bounded rewrite
    slice without marking the persistent goal complete.

## Rollback and failure policy

- Fix a failed package in place; never bypass it with a fallback, relaxed
  oracle, fixture branch, ignored test, or platform-specific output.
- A source-semantics defect revises the spec/plan and invalidates approvals
  before code resumes.
- An oracle mismatch requires independent source/encoder diagnosis before any
  expectation changes.
- Any newly required, removed, replaced, or omitted tracked path pauses
  implementation for a plan amendment, new exact hashes, and all three fresh
  pre-implementation approvals.
- Platform-only failure is reproduced or isolated; no platform output branch
  is added.
- Never amend, squash, force-push, or rewrite released Task 22A/B/C commits.

## Completion checklist

- [ ] Exact spec/plan hashes frozen
- [ ] Independent upstream/spec APPROVE
- [ ] Independent Ares/plan APPROVE
- [ ] Direct default-model spec/plan APPROVE
- [ ] Package A ownership/scale RED then GREEN and review
- [ ] Package B exact RED then GREEN and review
- [ ] Package C spatial/gap RED then GREEN and review
- [ ] Package D project reachability RED then GREEN and review
- [ ] Package E frozen fixture no-op acceptance GREEN
- [ ] Package F full matrix and structural audits green
- [ ] Exact implementation manifest/digest frozen
- [ ] Six-dimensional fix/re-review loop passed
- [ ] Whole spec, quality, and default-model implementation reviews approved
- [ ] Architecture/roadmap docs reviewed
- [ ] Final docs-inclusive local matrix green
- [ ] Conventional commit pushed
- [ ] Exact-SHA Tier-1 green on all five jobs
- [ ] Next source-cited slice recorded and started

**Status: DRAFT — implementation is forbidden until fresh independent and
default-model reviewers approve these exact spec/plan bytes.**
