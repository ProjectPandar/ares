# Task 22E Implementation Plan: Slicing-Mode Projection and Raw Policy

## Status, fixed points, and success condition

This plan is a draft. No production or test implementation may begin until
the exact specification and plan bytes receive fresh independent upstream/
spec, independent Ares/plan, and direct default-model approvals.

Fixed Ares baseline:

- branch: `codex/ksr-fdmtest-v4-parity`;
- commit: `a06deedecdc1e7b21b16c38e1d9bd28893eaf0fc`;
- tree: `8ec47b00baf857241095fe632abef841a0aa50fc`;
- Task 22D exact-SHA Tier-1 run: `29624952632`, green on Windows,
  formatting, macOS, Ubuntu, and WASM/browser.

Fixed OrcaSlicer source:

- commit: `8500fcdccaa10b5099ac20d252af3a7c560046f1`;
- tree: `b62d6017ba1ac7cb986f70fd6844353c7a776549`;
- direct raw boundary: `TriangleMeshSlicer.cpp:1483-1532,1864-1902`;
- real project adaptation boundary:
  `PrintObjectSlice.cpp:138-225` and
  `TriangleMeshSlicer.cpp:2003-2049`;
- upstream volume order: `Model.hpp:1227-1230`, explicitly deferred;
- exact adjacent Option, layer-Z, epsilon, polygon, winding, reversal, and
  ExPolygon helpers are cited by the specification.

Task success is a private, production-reachable stage that preserves true
source-volume association, maps only resolved 3MF Options to per-layer modes,
implements all four direct raw polygon policies, and reproduces
`slice_mesh_ex` by retaining all raw contours for project
`PositiveLargestContour` layers while recording that original mode for the
future ExPolygon stage. The KSR fixture remains byte-identical at this
geometry boundary, and the public result remains
`ProjectSlicingIncomplete`.

## Immutable behavior ledger

Implementation and reviews are constrained by these facts:

- external `regular`, `even_odd`, and `close_holes` map exhaustively to
  internal Regular, EvenOdd, and Positive;
- internal PositiveLargestContour is derived only by spiral model-part
  policy;
- Regular and EvenOdd are exact raw identity modes;
- Positive reverses only negative-area polygons and preserves polygon order;
- direct raw PositiveLargestContour uses strict greatest absolute area,
  first-on-tie selection, and CCW normalization;
- real project `slice_mesh_ex` adapts PositiveLargestContour to Positive
  before raw policy and delays largest selection until after ExPolygon union;
- project layers retain their original internal mode even when their raw
  policy is adapted;
- spiral affects only ModelPart volumes; NegativeVolume and
  ParameterModifier keep the object base mode;
- every model part uses its own resolved region matched by the real source
  volume index, never by ordinal;
- `source_volume_index` and one-based nonempty occurrence ordinal remain
  distinct identities through Raw, Chained, and Looped ownership;
- released Ares source/BFS volume order remains unchanged; Orca's ascending
  model-volume-ID order is explicitly deferred and no volume-order parity is
  claimed;
- bottom threshold starts at nonnegative `bottom_shell_layers`, extends by
  strict comparison against `bottom_shell_thickness - 1e-4`, is not clamped,
  and uses `f64::from(slice_z as f32)`;
- per-layer selection is strict `layer_index < threshold`;
- explicit layer-config ranges remain rejected by the released raw preflight;
- negative/nonfinite consumed bottom Options return keyed InvalidInput rather
  than clamp, wrap, or fallback;
- no legacy pipeline, fixture branch, reference-G-code read, source-pinning
  executable test, placeholder geometry, or premature ExPolygon behavior is
  added.

## Workspace discipline and evidence

1. Confirm clean tracked status, exact baseline commit/tree, branch tracking
   ref, direct remote SHA, and fixture hashes before implementation.
2. Use ignored `.superpowers/sdd/task22e-evidence.md` as the sole Task 22E
   RED/GREEN, review, manifest, local-matrix, release, and oracle ledger.
3. Verify Orca citations with read-only `git -C OrcaSlicer show`; never switch
   or modify the ignored source checkout.
4. Use `apply_patch` for every source, test, and documentation edit.
5. Preserve unrelated user work. Any unexpected tracked path stops the
   package for scope reconciliation.
6. Record exact commands, exit codes, test counts, hashes, and skips.
7. A package may touch only the exact approved manifest. Any required path
   addition or removal amends this plan and invalidates all approvals.
8. Do not commit package by package. Make one conventional Task 22E commit
   only after implementation, review loops, docs, and final verification pass.

## Pre-implementation exact-byte gate

After the documents are frozen:

1. record path, bytes, physical lines, and SHA-256 for both files;
2. dispatch a read-only upstream/spec reviewer to re-derive the dual
   `slice_mesh`/`slice_mesh_ex` semantics, Option mapping, per-volume spiral
   policy, f32 threshold, area/winding, and deferrals;
3. dispatch a separate read-only Ares/plan reviewer to verify source-volume
   ownership, resolved Option provenance, manifest completeness, TDD order,
   privacy, complexity, WASM safety, and release closure;
4. run the configured default-model review directly without `-m`, with
   runtime `task=deny` and `edit=deny`;
5. require literal `VERDICT: APPROVE` from all three;
6. recheck document hashes and tracked status after reviews.

Any document edit invalidates all three verdicts.

## Exact planned tracked manifest

Documentation created before implementation:

- `docs/superpowers/specs/2026-07-17-ksr-fdmtest-v4-task22e-slicing-mode-projection.md`;
- `docs/superpowers/plans/2026-07-17-ksr-fdmtest-v4-task22e-slicing-mode-projection.md`.

Production files modified:

- `crates/ares-core/src/geometry/polygon.rs`;
- `crates/ares-core/src/mesh_slicer.rs`;
- `crates/ares-core/src/mesh_slicer/chaining.rs`;
- `crates/ares-core/src/project_slice.rs`;
- `crates/ares-core/src/project_slice/raw_intersections.rs`;
- `crates/ares-core/src/project_slice/chained_intersections.rs`;
- `crates/ares-core/src/project_slice/looped_intersections.rs`.

Production files created:

- `crates/ares-core/src/mesh_slicer/slicing_mode.rs`;
- `crates/ares-core/src/project_slice/slicing_mode_intersections.rs`.

Test registration or support files modified:

- `crates/ares-core/src/mesh_slicer/tests.rs`;
- `crates/ares-core/src/project_slice/tests.rs`;
- `crates/ares-core/src/project_slice/tests/looped_fixture.rs`;
- `crates/ares-core/src/project_slice/tests/raw_fixture/mutations.rs`.

Test files created:

- `crates/ares-core/src/mesh_slicer/tests/slicing_mode.rs`;
- `crates/ares-core/src/project_slice/tests/slicing_mode_intersections.rs`;
- `crates/ares-core/src/project_slice/tests/slicing_mode_fixture.rs`.

Post-implementation documentation modified only after code approval:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

This is an unconditional 20-path final manifest. Adding, removing, replacing,
or omitting a path requires a plan amendment and fresh exact-byte approvals.
Ignored evidence and OpenCode prompt files are not tracked manifest entries.

## Module design before RED

### Polygon and loop mutation seam

`geometry::Polygon` gains one crate-private in-place `reverse` operation that
delegates to its owned point vector. It does not expose mutable points.

`mesh_slicer::LoopedLayer` gains one parent-visible mutable polygon-vector
reader used by its sibling slicing-mode module. Existing immutable polygon
access remains unchanged. No public API or clone-based geometry path is added.

### Pure raw policy

Create `mesh_slicer::slicing_mode` with:

```text
enum SlicingMode {
    Regular,
    EvenOdd,
    Positive,
    PositiveLargestContour,
}

apply_slicing_mode(&mut LoopedLayer, SlicingMode)
```

The module owns only the direct raw polygon policy and source-equivalent signed
area. It does not inspect project Options or know about ExPolygons.

Positive iterates in place and reverses a polygon only when signed area is
negative. PositiveLargestContour performs one ordered area scan, records only
strict improvements, removes the selected polygon without cloning, clears the
rest, normalizes selected winding, and pushes the selected polygon as the sole
result. Empty input returns unchanged; a nonempty all-zero set remains an
internal invariant failure.

### Source index propagation

Add `source_volume_index: usize` to:

- `RawVolumeIntersections`;
- `ChainedVolumeIntersections`;
- `LoopedVolumeIntersections`.

Every consuming `into_parts` tuple gains this value in the same first position.
All constructors forward it unchanged. Occurrence ordinal and volume type stay
unchanged. Test-only readers expose it for ownership assertions.

### Project mode stage

Create `project_slice::slicing_mode_intersections` with owned wrappers:

```text
SlicingModeLayer {
    mode: SlicingMode,
    looped_layer: LoopedLayer,
}

SlicingModeVolumeIntersections {
    source_volume_index,
    volume_ordinal,
    volume_type,
    layers,
}

SlicingModePrintObject {
    plan,
    volumes,
}
```

The entry point consumes `Vec<LoopedPrintObject>`, borrows the matching
`ResolvedProjectObject` slice, and receives the final normalized
`spiral_mode` boolean from `resolved.views.full.process.print.spiral_mode`.
It returns `Result<Vec<SlicingModePrintObject>, SliceError>` because consumed
external bottom Options require validation.

For each object it maps `object.slicing_mode` exhaustively. For each spiral
ModelPart it matches `source_volume_index` against the sole resolved
candidate's `ResolvedModelPartCandidate.volume_index`, computes its threshold,
and derives the original per-layer mode. It maps original
PositiveLargestContour to Positive only for the raw helper call, then stores
the original mode on the layer.

Missing resolved objects, candidates, or model parts are trusted internal
pipeline invariants and use `expect`, not public fallback logic. Negative or
nonfinite consumed external bottom values use the existing keyed InvalidInput
shape.

### Production wiring

In `slice_project`, derive the normalized spiral boolean before destructuring
resolved views, consume looped objects through the new stage, traverse its
owned plan/volume/layer/mode/polygon state, and retain the exact terminal
`ProjectSlicingIncomplete` error.

No previous stage is retained in parallel and no production geometry is
cloned.

## Planned test inventory

Every new test name begins with `task22e_`.

### Pure raw policy tests

In `mesh_slicer/tests/slicing_mode.rs`:

1. use the specification's exact A/B/C vectors and require Regular and EvenOdd
   identity, Positive's two exact full-vector reversals, and direct
   PositiveLargestContour's exact C-only result;
2. use the exact two-square tie vector and require the reversed first square;
3. freeze empty, single CCW, single CW, zero-area Positive, and nonempty
   all-zero PositiveLargestContour invariant behavior.

Use distinguishable polygons with exact integer coordinates. Assertions freeze
the full point sequence after reversal, not only area sign.

### Ownership and project policy tests

In `project_slice/tests/slicing_mode_intersections.rs`:

1. an ordinal-gap project proves real source index propagation through Raw,
   Chained, and Looped wrappers;
2. all three external object modes map exhaustively;
3. spiral derives PositiveLargestContour only for ModelPart, while negative
   and parameter-modifier volumes keep the base mode;
4. separate model-part source indices consume their own resolved regions;
5. use the specification's exact four-Z threshold table and its
   `bottom_shell_thickness=0.5001`, `boundary=0.5` equality vector to freeze
   layer count, thickness extension, zero, and no-clamp behavior;
6. `slice_z` is used instead of `print_z`, with the upstream f32-rounding
   regression;
7. `-1` bottom layers and `-0.1`, NaN, positive infinity, and negative infinity
   thickness return the specification's exact keyed errors when consumed;
8. the project raw adapter retains multiple polygons while making them CCW
   and recording PositiveLargestContour.

Synthetic resolved structs may exercise private policy seams. They must not
read Orca source or reference G-code at runtime.

### Real 3MF fixture tests

In `project_slice/tests/slicing_mode_fixture.rs`:

1. the committed fixture's typed Options and every Regular layer are exact;
2. Task 22D counts, representative lengths, both encodings/hashes, config
   hash, repeatability, and public lifecycle remain unchanged;
3. changing only process `slicing_mode` in the archive changes retained mapped
   mode; `regular -> even_odd` preserves exact raw polygons;
4. an object-level `slicing_mode` metadata override wins over process base;
5. changing only spiral and bottom region Options changes the exact layer-mode
   threshold while preserving raw polygon count above that threshold;
6. close-holes and spiral raw reversal are proven with the normative synthetic
   CW vectors, not inferred from the fixture's incidental winding.

Reuse the existing Task 22D test encoder through a test-only re-export. Do not
copy its encoding logic or move fixture data into production.

## TDD package sequence

Packages are serial in the shared worktree. Tests are registered before their
production seam is added. Parallelism is reserved for read-only review.

### Package A: source-volume ownership

1. Register the ownership test first.
2. Run the exact Task 22E filter and record the missing accessor/tuple RED.
3. Add the source index to Raw and forward it through Chained and Looped.
4. Adapt the placeholder traversal in `project_slice.rs` and the released test
   tuple consumer in `tests/raw_fixture/mutations.rs` in the same package.
5. Run Task 22B-D focused regressions, project-slice tests, formatting,
   strict core Clippy, and core WASM check.

### Package B: pure raw mode policy

1. Register all pure policy tests against the absent module/API.
2. Record the compilation RED.
3. Add Polygon reverse, LoopedLayer mutable ownership, module registration,
   and the minimal policy implementation.
4. Run Task 22D/E, mesh-slicer tests, formatting, strict core Clippy, and core
   WASM check.

### Package C: resolved project projection

1. Register project policy and threshold tests against the absent wrapper.
2. Record the compilation RED.
3. Add the owned project stage, exhaustive mapping, source-index region lookup,
   external bottom validation, f32 threshold, raw adaptation, and production
   wiring.
4. Run Task 22A-E, mesh/project tests, formatting, strict core Clippy, and both
   native/core WASM checks.

### Package D: real KSR acceptance and mutations

1. Register the real-archive tests before any test-only re-export they need.
2. Record the focused RED.
3. Add only the test encoder re-export and assertions; production code may
   change only if a test reveals a fixed source semantic defect.
4. Require the unmodified fixture to remain exact and every archive mutation
   to derive behavior from changed 3MF Options.

### Package E: closure

1. Run the focused and full matrices below.
2. Perform structural, hardcoding, source-pinning, fixture, and manifest
   audits.
3. Freeze sorted per-file SHA-256 values and the composite candidate digest.
4. Run the mandatory independent review/fix/re-review loop.
5. Obtain fresh whole-spec, whole-quality, and direct default-model approvals.
6. Only then update architecture/roadmap, review docs, verify, commit, push,
   and monitor exact-SHA Tier 1.

## Focused and full verification matrix

Use Cargo Nextest, never `cargo test`, as the default test entry point:

```text
cargo +1.91.0 nextest run -p ares-core -E 'test(/(^|::)task22e_[^:]*$/)'
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

Generate fresh `wasm-bindgen --target web` output and run the committed real-3MF
Playwright test with bundled workspace dependencies. Missing external
dependencies are recorded as blockers, never reported as passes.

## Structural and provenance audits

1. enumerate repository Rust files and require maximum physical LOC `<400`;
2. require no production source-splitting include macros;
3. require no filesystem, terminal/UI, native thread, Rayon, unsafe, platform
   branch, mutable global, or native Clipper use in the new path;
4. require no executable test opening, parsing, or hashing Orca source;
5. require no Task 22E reference-G-code read;
6. require no legacy planning/segments/contours/pipeline call;
7. inspect constants and branches for fixture-specific behavior;
8. prove fixture files and hashes unchanged;
9. run `git diff --check`;
10. compare the exact tracked set to the 20-path manifest;
11. freeze sorted per-file hashes and a composite digest in the ignored ledger.

## Mandatory independent review loop

After Package E, dispatch one independent read-only reviewer with the exact
manifest/digest and require separate verdicts for:

1. requirement completeness;
2. logical correctness;
3. boundary and edge cases;
4. code quality;
5. test coverage;
6. actual execution results.

The reviewer returns one prioritized fix list with paths, evidence, and
required reruns. Only the main thread edits. After fixes, rerun affected and
full gates, freeze new hashes, and send the revised candidate to the same
reviewer. Repeat until all six dimensions pass with an empty fix list or a
concrete external blocker is reproduced and recorded.

Then obtain three fresh whole-candidate approvals:

- specification implementation compliance;
- code quality and maintainability;
- direct default-model implementation review with task/edit denial.

Any code or test edit invalidates those approvals.

## Documentation and release

Only after implementation approval:

1. update `docs/architecture/option-parity-v4.md` to correct the direct
   `slice_mesh` versus project `slice_mesh_ex` distinction and record actual
   ownership, Option projection, fixture facts, and deferrals;
2. update `docs/roadmap.md` to mark Task 22E implemented while full G-code
   parity remains incomplete;
3. record the next exact source-cited ExPolygon/Clipper boundary;
4. record that upstream `Model.hpp:1227-1230` volume-ID ordering remains a
   prerequisite before cross-volume combination;
5. obtain independent documentation approval;
6. rerun the docs-inclusive local matrix and exact manifest/hash checks;
7. stage exactly the approved 20 paths;
8. commit with Conventional Commits;
9. push normally without amend, force, squash, or history rewrite;
10. verify local HEAD, tracking ref, and direct remote SHA are identical;
11. monitor the exact-SHA Tier-1 run until Windows, format, macOS, Ubuntu, and
    WASM/browser all pass;
12. append release evidence and start the next bounded source slice.

## Stop conditions

- A fixed-source ambiguity stops implementation for source audit.
- A required path outside the manifest stops implementation for plan
  amendment and fresh approvals.
- A test premise contradicted by fixed source is corrected in the test/plan,
  not hidden by production fallback.
- A Tier-1 failure is diagnosed on the exact SHA before claiming release.
- Never amend, squash, force-push, or rewrite released Task 22A-D commits.
- Never mark the persistent user goal complete while normalized reference
  G-code parity remains absent.

## Gate checklist

- [ ] Exact spec/plan hashes frozen
- [ ] Independent upstream/spec APPROVE
- [ ] Independent Ares/plan APPROVE
- [ ] Direct default-model spec/plan APPROVE
- [ ] Package A source identity RED then GREEN
- [ ] Package B pure mode RED then GREEN
- [ ] Package C project projection RED then GREEN
- [ ] Package D real fixture/mutations GREEN
- [ ] Package E full matrix and structural audits green
- [ ] Exact implementation manifest/digest frozen
- [ ] Six-dimensional fix/re-review loop passed
- [ ] Whole spec, quality, and default-model reviews approved
- [ ] Architecture/roadmap docs reviewed
- [ ] Final docs-inclusive local matrix green
- [ ] Conventional commit pushed
- [ ] Exact-SHA Tier-1 green on all five jobs
- [ ] Next source-cited slice recorded and started

**Status: DRAFT — implementation is forbidden until fresh independent and
default-model reviewers approve these exact specification and plan bytes.**
