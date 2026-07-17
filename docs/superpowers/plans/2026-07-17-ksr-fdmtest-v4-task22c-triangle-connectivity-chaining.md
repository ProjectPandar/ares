# Task 22C: Triangle-Connectivity Slice Chaining Implementation Plan

> **Execution contract:** Follow this checklist in order. No production or
> test implementation may begin until an independent Codex reviewer and the
> repository's direct default-model review gate approve the exact frozen spec
> and plan bytes. Keep implementation packages uncommitted until whole-slice
> approval. After implementation, use one independent read-only reviewer for
> the required six-dimensional review/fix/re-review loop.

**Specification:**
`docs/superpowers/specs/2026-07-17-ksr-fdmtest-v4-task22c-triangle-connectivity-chaining.md`

**Specification bytes / lines / SHA-256:**
`20599` / `425` /
`0173b45cdb11af91169229a99a1ce67277ac2912ff8e15446a627fc2be5f9d6c`

**Pinned OrcaSlicer SHA / tree:**
`8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549`

**Ares baseline SHA / tree / branch:**
`4180d082858696d7eacd094358787a655bfc59f4` /
`164c0f55ba38ef55c98a9cb5e6acf109bef1fca8` /
`codex/ksr-fdmtest-v4-parity`

## Goal and immutable behavior ledger

Port only OrcaSlicer
`TriangleMeshSlicer.cpp::chain_lines_by_triangle_connectivity` and the minimal
integer polygon/open-chain data it constructs. Consume released Task 22B raw
layers and produce private ordered closed polygons plus unrepaired open
polylines. Integrate that state into the production project path, then stop at
the existing `ProjectSlicingIncomplete` boundary.

The implementation is constrained by these exact facts:

- input order is Task 22B mesh face order;
- identity is tagged `Vertex(u32)` or `Edge(u32)`, never bare numeric ID;
- only directed `last.b -> candidate.a` identity equality connects;
- seed and output component order follow original raw indices;
- equal-start candidates use original-index FIFO as a deterministic tie-break;
- consumption is request-local and every line contributes exactly one edge;
- closed point lists omit the repeated first point and are otherwise untouched;
- open point lists include the final B point, retain tagged endpoints, compute
  scaled Euclidean length, and begin with `consumed=false`;
- no repair, reversal, winding, area, hole, Clipper, region, or G-code behavior
  enters this task;
- no new Option or public error is introduced;
- production project slicing never calls the old STL
  `planning/segments/contours/pipeline` path.

For the real fixture, exact source-derived acceptance is 460 layers, 3,288
closed polygons, zero opens, 116,472 polygon points, face-order digest
`6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`,
and normalized semantic digest
`7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`
over 2,190,993 encoded bytes. These corrected coordinate oracles exclude the
3MF `source_offset_x/y` provenance fields, matching released Task 22B raw
coordinates; two read-only independent replays reproduced them after the
first Package D run exposed the original oracle's invalid provenance offset.

## Frozen baseline and workspace discipline

1. Confirm `HEAD`, branch, tracking branch, status, and the two fixture hashes.
2. Preserve the pushed cleanup commit. Do not amend, squash, or mix it into
   Task 22C.
3. Verify fixed Orca citations with `git -C OrcaSlicer show`; never checkout or
   modify the ignored upstream tree.
4. Create ignored `.superpowers/sdd/task22c-evidence.md` for RED/GREEN outputs,
   exact path manifests, hashes, review verdicts, and CI closure.
5. Before each package, record `git status --short` and reject unrelated
   workspace changes. Do not discard user changes.
6. Use `apply_patch` for edits. Production source splitting uses `mod`, never
   `include!` or data-embedding macros.
7. Do not read the reference G-code from Task 22C tests. It is final
   post-processed output, not a raw-loop oracle.
8. Do not commit package-by-package. One conventional Task 22C feature commit
   is created only after implementation, documentation, and all reviews pass.

## Pre-implementation review gate

Freeze and record exact spec/plan byte counts and SHA-256 values. Dispatch a
fresh read-only reviewer to check:

- every fixed upstream line claim;
- stopping boundary and explicit deferrals;
- tagged identity, ordering, FIFO, closure, open length, and complexity;
- Rust ownership and production integration;
- source-derived fixture oracles and encoder definition;
- TDD order, exact manifest, commands, WASM safety, and release closure.

Require literal `VERDICT: APPROVE`. Run the configured default-model review
directly with runtime task/edit denial and require the same verdict. Any edit
invalidates both reviews and requires fresh hashes and reviews.

## Exact planned tracked manifest

Documentation created before implementation:

- `docs/superpowers/specs/2026-07-17-ksr-fdmtest-v4-task22c-triangle-connectivity-chaining.md`
- `docs/superpowers/plans/2026-07-17-ksr-fdmtest-v4-task22c-triangle-connectivity-chaining.md`

Production candidates:

- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/polygon.rs`
- `crates/ares-core/src/mesh_slicer.rs`
- `crates/ares-core/src/mesh_slicer/intersection.rs`
- `crates/ares-core/src/mesh_slicer/chaining.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/chained_intersections.rs`

Test candidates:

- `crates/ares-core/src/geometry/tests.rs`
- `crates/ares-core/src/geometry/tests/polygon.rs`
- `crates/ares-core/src/mesh_slicer/tests.rs`
- `crates/ares-core/src/mesh_slicer/tests/chaining.rs`
- `crates/ares-core/src/mesh_slicer/tests/chaining/identity.rs`
- `crates/ares-core/src/mesh_slicer/tests/chaining/open.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/chained_intersections.rs`
- `crates/ares-core/src/project_slice/tests/chained_fixture.rs`
- `crates/ares-core/src/project_slice/tests/chained_fixture/encoding.rs`
- `crates/ares-core/src/project_slice/tests/chained_fixture/oracles.rs`

Post-approval documentation candidates:

- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

Omit an optional child test file if its parent remains comfortably below 400
lines. Adding any other tracked path requires a scope explanation and fresh
manifest review before implementation continues. The ignored evidence ledger
and build outputs are not tracked candidates.

## Test inventory and naming

All new test names begin with `task22c_`. Freeze these behavioral tests before
production implementation:

1. `task22c_polygon_preserves_integer_points_without_normalization`
2. `task22c_edge_cycle_forms_one_ordered_polygon`
3. `task22c_vertex_cycle_forms_one_ordered_polygon`
4. `task22c_open_chain_preserves_endpoints_points_length_and_state`
5. `task22c_equal_coordinates_with_different_references_do_not_connect`
6. `task22c_vertex_and_edge_with_same_numeric_id_do_not_connect`
7. `task22c_successor_is_never_reversed`
8. `task22c_matching_identities_require_matching_coordinates_in_debug_builds`
9. `task22c_components_keep_seed_order`
10. `task22c_equal_start_candidates_use_input_fifo`
11. `task22c_empty_single_open_and_single_closed_layers_are_retained`
12. `task22c_open_length_converts_extreme_coordinates_before_subtraction`
13. `task22c_mixed_components_conserve_every_input_edge`
14. `task22c_project_wrapper_preserves_object_volume_and_layer_ownership`
15. `task22c_ksr_fixture_matches_exact_counts_lengths_and_digests`
16. `task22c_ksr_fixture_chaining_is_repeatable_and_public_api_stays_incomplete`

If two assertions naturally share one setup, they may stay in one named test,
but no listed behavior may disappear. Additional tests are allowed only for a
review-discovered contract and must be recorded in the ledger.

## Package A: Minimal integer polygon domain

### A.1 Establish the polygon RED

Create the test registration and a polygon test that fixes an input containing
non-lexicographic order, clockwise winding, and a repeated interior point. It
must require exact output equality and no appended first point.

Run the exact test and record its missing-module/type failure before production
files are created.

### A.2 Implement only storage

Create `geometry/polygon.rs` with `Polygon { points: Vec<Point> }`, constructor,
and read-only access. Register it from `geometry.rs`. Do not add area, winding,
validation, normalization, `ExPolygon`, or generic path helpers.

Run:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(task22c_polygon_preserves_integer_points_without_normalization)'
cargo +1.91.0 fmt --all -- --check
```

Require GREEN and `<400` LOC before Package B.

## Package B: Core triangle-connectivity chaining

### B.1 Establish all core REDs

Register the chaining test module and write tests 2-13 with hand-built points,
tagged endpoint references, and expected point sequences fixed in source.

The coordinate-invariant test is compiled under `cfg(debug_assertions)` and
uses `catch_unwind` to cover both the successor A/previous B assertion and the
closed seed A/final B assertion without turning either into a public error.

The test helper may require `IntersectionPoint::new` and
`IntersectionLine::new` to become `pub(super)` within `mesh_slicer`. This is the
only allowed visibility widening; do not add public constructors or mutate raw
records.

Run the exact Task 22C filter. Record the compilation/test failures and prove
no production chaining file exists yet.

### B.2 Implement the smallest algorithm

Create `mesh_slicer/chaining.rs` and register it from `mesh_slicer.rs`.

Implementation order:

1. Define `OpenPolyline` and `ChainedLayer` storage/accessors.
2. Define a private start index with separate flat edge and vertex
   `(identity_id, original_index)` vectors.
3. Sort both vectors by `(identity_id, original_index)`, derive flat equal-ID
   ranges, and retain one advancing cursor per range.
4. Keep a local consumed vector and raw-index seed scan.
5. Implement directed successor selection and `debug_assert_eq!` for matching
   successor coordinates.
6. Emit closed polygons without the terminal point or open polylines with it.
7. In the closed branch, separately `debug_assert_eq!` the seed A coordinate
   against the last B coordinate before constructing the polygon.
8. Compute open length by f64 conversion before subtraction.
9. Preserve short/zero/duplicate forms; add no error path.

Use flat sorted records plus range cursors and binary search. Each candidate
record may be examined or advanced past only once. Do not allocate one
`VecDeque` per identity, repeatedly rescan a full identity group, or derive
connectivity from coordinates. The resulting bound is `O(n log n)` overall
and `O(n)` auxiliary memory without per-identity heap allocation.

Run all core Task 22C tests plus existing mesh-slicer tests, Task 22B tests,
strict Clippy, and core WASM tests. Record exact counts.

## Package C: Project ownership wrapper

### C.1 Establish the wrapper RED

Use the existing `project_slice::tests::raw_support::{intersections,
planned_layers}` seam with hand-built project objects to create multiple raw
objects/volumes and empty layer slots. Do not change or expose fields from
`raw_intersections.rs`, and do not add production validation solely for tests.

Fix expected object order, complete plan, volume ordinal/type, layer-slot
order, and per-layer chained output. Run the test and record failure before the
wrapper exists.

### C.2 Consume raw state once

Create `project_slice/chained_intersections.rs`. Map object, volume, and layer
vectors without sorting or cloning. Preserve the plan and volume metadata.

In `slice_project`, move `intersected_objects` into the wrapper immediately
after `prepare_project_slice`, traverse the chained state to keep every field
production-reachable, and retain `ProjectSlicingIncomplete`.

Do not change `ProjectSliceState` or released Task 22B raw tests unless an
independent reviewer identifies a real ownership defect.

Run the wrapper test, existing project-slice tests, Task 22A/22B filters, and
WASM check.

## Package D: Independent real-fixture acceptance verification

### D.1 Freeze the encoder and fixture constants before observing Ares output

Create separate fixture encoder/oracle modules. The encoder must be
test-specific and must not call production normalization helpers.

Face-order encoding:

- preserve layer order;
- preserve polygon output order and point start/order;
- write ASCII `L<layer>\n`;
- write each polygon as
  `C;<count>;<x1>,<y1>;...;<xn>,<yn>\n` with one semicolon before each point,
  semicolons between points, no trailing semicolon, base-10 signed integers,
  and LF.

Semantic encoding:

- rotate each polygon to the earliest occurrence of its lexicographically
  smallest numeric `(x, y)` point;
- never reverse it;
- sort the resulting `Vec<Point>` values per layer by numeric lexicographic
  sequence order;
- use the same bytes as face-order encoding afterward.

Before consulting Ares output, freeze all values from the approved spec in test
constants: counts, representative layers, polygon-length vectors, byte length,
and both SHA-256 values. Because Packages B and C already implement and
integrate the behavior, the first fixture run may honestly be GREEN. Package D
is an independently frozen acceptance verification, not a claimed RED stage.
It may never update constants from observed Ares bytes.

If the first frozen run exposes a faulty external oracle, keep the constants
unchanged until at least two read-only independent reproductions identify the
same oracle defect and derive the same corrected values from released Task 22B
input facts. Record the superseded values and diagnosis before changing the
spec, plan, or test constants; never derive a correction from Ares output
alone.

### D.2 Run the frozen acceptance verification without changing constants

Chain a clone of one prepared Task 22B state, then assert:

- plan/volume/layer ownership;
- 3,288 closed, zero open, and 116,472 closed points;
- conservation and no repeated terminal point;
- all representative counts and polygon lengths;
- 2,190,993 semantic bytes and both corrected fixed digests;
- repeated chaining equality;
- unchanged 49,004-byte config block and existing SHA;
- public `slice_project` still returns `ProjectSlicingIncomplete`.

If a digest differs, stop and diagnose ordering, interpolation provenance, or
encoder mismatch. Never replace an expected digest with Ares output merely to
make the test pass.

## Package E: Scope and regression closure

Run and record:

```text
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

Run the existing real-3MF browser/WASM gate if its bundled dependencies are
available. A skipped external gate must be recorded with the exact missing
dependency; do not claim it passed.

### Required structural audits

1. Enumerate every `.rs` file and require maximum physical LOC `<400`.
2. Require no production `include!`, `include_bytes!`, or `include_str!`.
3. Require no native threads, Rayon, unsafe, filesystem, platform branch, or
   nondeterministic global state in the new path.
4. Require no executable test that opens/hashes/parses Orca source files.
5. Require no production or Task 22C test reference to old STL
   `planning/segments/contours/pipeline` APIs.
6. Require no reference-G-code read in Task 22C tests.
7. Require exact fixture hashes and no generated fixture changes.
8. Require `git diff --check` and an exact tracked path set within the approved
   manifest.
9. Inspect every new constant for fixture-specific branching or hardcoded
   production geometry. Test oracle constants are allowed only in tests.
10. Freeze a sorted final implementation manifest with per-file SHA-256 and a
    composite path/content digest in the ignored ledger.

## Whole-implementation review loop

After Package E is green, freeze the candidate manifest and fresh command
results. Dispatch one independent reviewer that cannot edit the workspace.
Require separate verdicts for:

1. requirement completeness;
2. logical correctness;
3. edge cases;
4. code quality;
5. test coverage;
6. actual execution results.

The reviewer must return one prioritized fix list with exact paths and reasons.
The main thread alone applies fixes, reruns affected and full gates, updates the
manifest, and asks the same reviewer to revalidate the revision. Continue until
all six verdicts pass or the same external blocker has been established with
concrete evidence.

After the six-dimensional review passes, require:

- whole-specification implementation approval;
- whole-code-quality approval;
- direct default-model implementation approval with task/edit denial;
- no workspace modifications by any reviewer.

Any code/test change after approval invalidates applicable verdicts.

## Documentation and release

Only after code/test approval:

1. update `docs/architecture/option-parity-v4.md` with the fixed source
   boundary, destination ownership, exact fixture facts, deterministic FIFO
   choice, and explicit next-stage deferrals;
2. update `docs/roadmap.md` to record Task 22C as implemented but the overall
   G-code parity goal as incomplete;
3. obtain independent documentation approval;
4. rerun the complete local matrix and audits against the final docs-inclusive
   manifest;
5. stage exactly the approved manifest;
6. commit with a Conventional Commit such as
   `feat(slicing): chain project intersections`;
7. push `codex/ksr-fdmtest-v4-parity` normally;
8. verify local HEAD, tracking ref, and direct remote SHA match;
9. monitor the exact pushed SHA's Tier-1 workflow and require format, Linux,
   macOS, Windows, and WASM jobs green;
10. append release evidence to the ignored ledger.

Do not mark the persistent user goal complete. Task 22C release only unlocks a
new Task 22D spec/plan for the cited exact-join and gap-repair boundary.

## Rollback and failure policy

- A package failure is fixed in place; do not bypass it with legacy fallback,
  fixture branching, relaxed digest constants, or skipped tests.
- An oracle mismatch requires a concrete source/encoder diagnosis before any
  expectation changes.
- If the new production path needs behavior outside the approved upstream
  boundary, stop and revise the spec/plan through fresh reviews.
- If the fixed Orca citation is wrong, correct the docs and invalidate all
  prior approvals before coding.
- If Tier-1 fails only on one platform, reproduce or isolate the portability
  cause; do not add a platform-specific output branch.
- Never force-push or rewrite the released Task 22A/22B/cleanup commits.

## Completion checklist

- [ ] Exact spec and plan hashes independently approved
- [ ] Default-model spec/plan review approved
- [ ] Package A RED then GREEN
- [ ] Package B RED then GREEN
- [ ] Package C RED then GREEN
- [ ] Package D frozen-oracle acceptance verification GREEN
- [ ] Package E full matrix and structural audits green
- [ ] Exact implementation manifest frozen
- [ ] Six-dimensional review/fix/re-review loop passed
- [ ] Whole spec, quality, and default-model implementation reviews approved
- [ ] Architecture and roadmap updated and reviewed
- [ ] Final docs-inclusive local matrix green
- [ ] Conventional commit pushed
- [ ] Exact-SHA Tier-1 format/Linux/macOS/Windows/WASM green
- [ ] Task 22D continuation boundary recorded

**Status: DRAFT — implementation is forbidden until fresh independent and
default-model reviewers approve these exact spec/plan bytes.**
