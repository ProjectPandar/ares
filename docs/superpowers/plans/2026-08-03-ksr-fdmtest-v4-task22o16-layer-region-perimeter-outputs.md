# Task 22O.16 — Layer-region perimeter-output materialization Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o16-layer-region-perimeter-outputs.md`

## Status

Implemented and locally validated after approved independent/OpenCode specification and plan reviews. The KSR checksum is `-169716507603417685621692788651154411580` with totals `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 1112]`; 14 focused O16 tests, 192 O1/O10-O16 regressions, and 5,554 workspace tests with 2 skipped pass, together with strict Clippy, workspace/native and both WASM checks, formatting, diff, LOC, forbidden-pattern, source-pinning, dependency, and staging audits. The final independent six-dimensional implementation review and OpenCode review both returned `VERDICT: APPROVE`.

## Validation contract

Port the KSR-reached one-compatible-region output seam at fixed Orca commit `8500fcdcc`: materialize the five `LayerRegion` perimeter-result fields from the reviewed O15 state in exact source append order, copy `fill_expolygons` through const-reference semantics, consume only artificial O-stage intermediates, retain downstream traversal/config context, remain stack-safe/Tier-1 portable, and stop at `PrintObject::prepare_infill` (`PrintObject.cpp:560`) with `ProjectSlicingIncomplete`.

## Gate 0 — Freeze the reviewed contract

1. Keep the specification synchronized with reviewer corrections.
2. Require literal `VERDICT: APPROVE` from both the independent and OpenCode specification reviewers.
3. Review this plan with both reviewers; any revision returns to both reviewers.
4. Do not add Rust source or tests until both plan approvals cover the same spec/plan.

## Task 1 — Materialize one source-shaped layer-region record

Upstream boundary: `LayerRegion.cpp:82-142`, `Layer.hpp:50-61,72-74`, `PerimeterGenerator.hpp:85-101`, `PerimeterGenerator.cpp:1569,1623,1670,1688`, `Surface.hpp:159-166`. Rust destination: new crate-private `project_slice::perimeters::layer_region` types and record materializer.

Files, all below 400 LOC:

- `crates/ares-core/src/project_slice/perimeters/layer_region.rs`
- `crates/ares-core/src/project_slice/perimeters/layer_region/types.rs`
- `crates/ares-core/src/project_slice/perimeters/layer_region/tests.rs`
- `crates/ares-core/src/project_slice/perimeters/layer_region/tests/materialize.rs`
- module declaration/re-export in `crates/ares-core/src/project_slice/perimeters.rs`

RED:

1. Declare real test modules before the first run.
2. Construct a literal `PreparedInfillBoundaryRecord` with at least three surfaces, multiple nonempty/empty `appended.collections` and `gap_fill.entities` wrappers, two fill surfaces, and two no-overlap expolygons.
3. Assert source surface→collection/entity append order, individual collection boundaries, gap `Path`/`Loop` order, fill surface/no-overlap order, and exact metadata.
4. Capture nested allocation pointers, not wrapper-vector pointers. Require flat source append order, nested collection-`entities`/loop/path/point identity, moved record-level fill/no-overlap vector and geometry identity, and allocation-distinct value-equal `fill_expolygons` copies. Do not assert identity or nonidentity for excluded wrapper storage or inline element addresses.
5. Add a populated empty-surface record case whose record-level fill fields remain ordered.
6. Run the focused filter and record RED because no LayerRegion result type/materializer exists.

GREEN:

1. Define `PreparedLayerRegionPerimeterRecord` with ordered `perimeters`, `thin_fills`, `fill_surfaces`, `fill_expolygons`, and `fill_no_overlap_expolygons` fields.
2. Before moving `fill_surfaces`, clone each expolygon through `RegionSurface::as_parts()` in order; do not clone surface metadata into `fill_expolygons`.
3. Reserve exact summed collection/entity capacities in new record-level vectors. Append each per-surface wrapper vector in O15 surface order. Move nested entities and geometry; do not sort, union, clip, parse options, or expose wrapper-storage identity as behavior.
4. Move record-level fill surface and no-overlap vectors directly. Consume `remaining`, medial/gap-domain data, overlap sidecars, inactive provenance, source-index sidecars, and artificial wrapper storage.
5. Rerun direct materialization tests GREEN.

## Task 2 — Add aligned object/project successor and ownership proof

Upstream boundary: `Layer.cpp:185-226` one-compatible-region/nonempty branch plus `LayerRegion.cpp:82-142`. Rust destination: `PreparedPostLayerRegionPerimeters` after O15.

Files:

- `crates/ares-core/src/project_slice/perimeters/layer_region.rs`
- `crates/ares-core/src/project_slice/perimeters/layer_region/types.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/layer_region.rs`
- child test modules `shape.rs` and `ownership.rs`
- parent test/module declarations in `project_slice/tests/perimeters.rs` and `project_slice/tests/perimeters/`

RED:

1. Add object/record shape cases with leading, middle, and trailing `None` slots and multiple objects. Require exact slot/order preservation.
2. For every populated record, assert predecessor input alignment by object identity, slot count, `region_id`, and compile-time one-entry compatibility; reject no trusted invariant by searching values.
3. Capture KSR O10 nested collection/entity/path/point, O14 gap-loop/path/point, O15 fill/no-overlap vector/geometry, and boxed O5 predecessor allocations. Require exact destination identity only for allocations named by the spec; explicitly assert copied fill expolygons do not alias source fill geometry.
4. Require the successor to retain only `Box<PreparedPostClassicTraversal>` plus LayerRegion output objects; no O13/O14/O15 compatibility shell remains.

GREEN:

1. Add `PreparedPostLayerRegionPerimeters` and object/record types.
2. Implement a non-fallible whole-project move that first asserts all trusted alignment, then destructures O15 once and materializes records in order.
3. Keep `None` slots untouched and preserve exact predecessor pointer identity.
4. Rerun direct, shape, ownership, and O15-O10 regressions GREEN.

## Task 3 — Wire lifecycle and iterative cleanup

Upstream boundary: completion of `Layer::make_perimeters` and the exact next stop at `PrintObject::prepare_infill`, `PrintObject.cpp:560`.

Files:

- `crates/ares-core/src/project_slice/perimeters.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/incomplete_sink.rs`
- new `crates/ares-core/src/project_slice/incomplete_sink/layer_region.rs`
- lifecycle/cleanup tests under `project_slice/tests/perimeters/layer_region/`

RED:

1. Add a lifecycle counter test that fails until public `slice_project` invokes O16 exactly once and still returns `ProjectSlicingIncomplete`.
2. Add a 64 KiB success/incomplete cleanup test. Deepen both traversal-seed and hierarchy-loop predecessor families to 10,000 nodes before O16 materialization and sink consumption.
3. Add a typed mutation test proving `counterbore_hole_bridging != none` fails in existing Classic preflight and O16 invocation count remains zero.

GREEN:

1. Add `prepare_post_layer_region_perimeters(project)` after `prepare_post_classic_infill_boundary`.
2. Switch `slice_project_sync` to the O16 successor and consume output records through the new sink child module.
3. Reuse the existing iterative boxed-traversal sink; keep the 392-line parent below 400 LOC.
4. Rerun lifecycle, cleanup, counterbore precedence, and O15 lifecycle regressions GREEN.

## Task 4 — Pin KSR outputs and anti-hardcoding mutations

Files:

- `crates/ares-core/src/project_slice/tests/perimeters/layer_region/ksr.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/layer_region/options.rs`

RED/GREEN:

1. Add the KSR checksum test with a placeholder before the successor passes.
2. Hash object/slot/record delimiters, retained input source/layer/region indices, all five output field lengths, collection/entity/path metadata and points, gap path/loop metadata and points, fill surface metadata/geometry, copied fill geometry, and no-overlap geometry.
3. Run from independently parsed bytes twice. Pin only the literal checksum and useful nonempty totals after a clean run; remove temporary prints.
4. Mutate typed 3MF wall-loop, gap-infill, and ordinary/top overlap options independently. Assert the corresponding perimeter, thin-fill, or fill-geometry structure changes without fixture-name/hash branching.
5. Keep the CLI golden ignored/incomplete; identify `PrintObject.cpp:560` as the next required source boundary.

## Task 5 — Documentation, verification, independent review, and ship

1. Update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, and synchronized spec/plan status with the exact source seam, one-region limitation, wrapper-consumption/nested-identity rule, checksum/totals, and next `PrintObject.cpp:560` boundary.
2. Run focused O16 plus O15-O10 regressions.
3. Run:
   - `cargo nextest run --workspace`
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`
   - `cargo fmt --all -- --check`
   - `git diff --check`
   - changed-Rust LOC, forbidden-pattern, source-pinning, dependency, and staging audits.
4. Record the existing Windows/macOS/Linux CI matrix definition as Tier-1 native evidence; do not claim runtime CI for an uncommitted tree.
5. Run an independent read-only six-dimensional implementation review and the separate read-only OpenCode review against the same diff/evidence. A finding from either returns to the main thread; after fixes rerun affected/full gates and both reviewers.
6. After both literal approvals, synchronize final status, create small Conventional Commits for slicing and docs, exclude `.pi-subagents/`/generated logs, push `main`, verify remote HEAD, and verify a clean worktree.

## Stop condition

Stop O16 only when the exact five LayerRegion output fields are materialized from source-derived O15 state, wrapper consumption and nested identity/copy semantics are proven, KSR and anti-hardcoding tests pass, public slicing reaches O16 once and remains incomplete at `PrintObject.cpp:560`, every verification gate passes, both final reviewers approve the identical diff, documentation is synchronized, and commits are pushed.
