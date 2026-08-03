# Task 22O.15 — Classic Infill-Boundary Construction Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o15-infill-boundary.md`

## Status

Implemented and locally validated after approved independent/OpenCode plan review. The literal KSR checkpoint is `136197013209006370081121271251125478104`; 49 focused O15 tests and geometry regressions, 5,540 workspace tests with 2 skipped, strict Clippy, workspace/native and both WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, and staging audits pass. The final independent six-dimensional implementation rereview and OpenCode rereview both returned `VERDICT: APPROVE`.

## Validation contract

Port the supported Classic slice of `PerimeterGenerator.cpp:1628-1691` exactly: derive all overlap values from aligned effective typed 3MF options, preserve every integer/floating cast and geometry call order, aggregate source internal fill surfaces and no-overlap polygons, stage transactionally, retain O14-O5 ownership, remain stack-safe/Tier-1 portable, and leave public slicing at `ProjectSlicingIncomplete`. Activated extra-perimeter generation stays rejected by the existing Classic preflight and is not replaced with a fallback.

## Gate 0 — Approve the development contract

1. Synchronize final spec and plan status after every review correction.
2. Require literal `VERDICT: APPROVE` from independent read-only spec and plan reviewers.
3. Require separate OpenCode approval of the same final spec and plan.
4. Do not add RED tests or production code until all four approvals cover the synchronized documents.

## Task 1 — Add exact `ExPolygon::simplify_p` polygon output

Upstream boundary: `ExPolygon.cpp:223-248`, the complete Douglas–Peucker definition at `MultiPoint.cpp:164-229`, and the StrictlySimple NonZero flat-path repair reached through `ClipperUtils.hpp:509-520`/`ClipperUtils.cpp:1019-1031`. `ExPolygon.cpp:250-253` is explicitly unreached/deferred. Rust destination: a sibling helper in `crates/ares-core/src/geometry/simplification.rs` with exports only inside `geometry`; reviewed Clipper/DP kernels remain unchanged.

Files:

- `crates/ares-core/src/geometry/simplification.rs`
- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/tests/simplification.rs` or a real child module if the file approaches 400 LOC

RED:

1. Add literal contour-plus-two-holes input whose output order differs if holes precede the contour.
2. Add two overlapping expolygons that distinguish the required per-expolygon StrictlySimple flat-path repair from the forbidden per-expolygon `union_ex`/PolyTree grouping, then distinguish both from appending every `simplify_p` polygon before one aggregate union.
3. Add tolerance equality and just-above cases that pin closed-end handling and exact fixed points.
4. Run the focused simplification filter and record RED because only the existing expolygon-union helper exists.

GREEN:

1. Extract or reuse the existing closed Douglas–Peucker path operation without changing its numeric comparisons.
2. Implement a crate-private borrowed sibling helper that copies contour then holes, closes each, simplifies, removes the duplicate endpoint, calls the reached `simplify_polygons` flat-path repair, and returns ordered `Vec<Polygon>` without a PolyTree grouping pass.
3. Keep `append_simplified_expolygon` and all O1 callers unchanged; O15 must call only the new sibling helper.
4. Rerun simplification and existing prelude regressions GREEN.

## Task 2 — Preflight source-exact inset and typed overlap values

Upstream boundary: `PerimeterGenerator.cpp:1628-1654`, `PerimeterGenerator.hpp:135,161`, `libslic3r.h:52,92-94,124-125`, `Config.hpp:1165-1178`, and `PrintConfig.cpp:4148-4172`. Rust destination: new `classic::infill_boundary::preflight`.

Files, split below 400 LOC:

- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary/preflight.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary/types.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary/tests.rs` and `tests/preflight.rs`
- production module declaration/re-export in `crates/ares-core/src/project_slice/perimeters/classic.rs`, and `#[cfg(test)] mod tests;` inside the new `classic/infill_boundary.rs` before the first RED run
- `crates/ares-core/src/project_slice/tests/perimeters/classic/infill_boundary.rs` plus `options.rs`, and parent real-`mod` declarations before the first aligned RED run

RED:

1. Add table-driven literal tests for final O3 loop numbers `< 0`, `0`, and `> 0`, proving integer external/perimeter half selection.
2. Add layer zero, middle layer with upper slices, and last layer without upper slices. Pin ordinary/top overlap values and the final inset for distinct typed percentages.
3. Add odd solid spacing and odd inset values so integer basis division is observable.
4. Pin Normal and LargeBed percent conversion values through literal integer results, including a fractional percentage whose source unscale/percent/scale sequence differs from a rounded shortcut. Pin raw `m_scaled_resolution = max(typed resolution, EPSILON) / factor` separately from O1's arc-adjusted surface tolerance.
5. Make negative representable percentages mandatory because typed `Percent` accepts every finite value; pin the full `basis -> unscale -> percent -> scale -> truncation` sequence rather than an algebraically cancelled shortcut.
6. Independently trigger ordinary-overlap conversion overflow/non-finite intermediate, middle-layer top-overlap conversion overflow/non-finite intermediate, post-overlap subtraction overflow, and integer `min/2 - overlap` overflow. Pin the stable overlap-range error and zero geometry invocations for every reachable failure.
7. Add maximal/minimal source-derived boundary proofs that basis addition, the 0.6 minimum-spacing conversion, unary negation, and selected `-inset - overlap` remain representable. Do not create impossible scalar states to manufacture failures.
8. Before production code, add typed rebuilt-3MF mutations for `infill_wall_overlap` and `top_bottom_infill_wall_overlap`, including fixed, fractional, negative-representable, and overflow values; add a LargeBed printable-area mutation and assert the RED boundary.
9. Add an observation counter proving whole-project numeric preflight completes before any O15 geometry.

GREEN:

1. Recover immutable aligned inputs through the exact traversal → hierarchy → onion → top-split → prelude → perimeter-input path. Assert slot, surface-count, and `source_index` alignment as internal invariants rather than searching by value.
2. Recover raw typed print `resolution` from the preserved resolved config, validate it consistently with O1, and build aligned record/surface sidecars containing raw `m_scaled_resolution` plus only the other source-derived scalar values needed by geometry. Do not reuse O1's `surface_simplify_resolution`.
3. Preserve source integer divisions and the exact `double(basis) * factor * percent / 100 / factor` operation order. Evaluate every reached signed `coord_t` intermediate with checked/wider arithmetic before narrowing, including selected no-overlap deltas and unary negation.
4. Do not add a new generic option adapter; read typed `RegionOptions.infill_wall_overlap` and `top_bottom_infill_wall_overlap` directly.
5. Rerun focused preflight tests GREEN.

## Task 3 — Construct internal fill and no-overlap geometry

Upstream boundary: `PerimeterGenerator.cpp:1655-1691`, `ExPolygon.cpp:223-248`, `ClipperUtils.hpp:19-27,344-348,391-393,509-520,548-553`, `ClipperUtils.cpp:560-588,788-824,1019-1031`, `Surface.hpp:9-55,245-269`, and `SurfaceCollection.hpp:74-85`. Rust destination: `classic::infill_boundary::geometry` plus orchestration.

Files:

- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary/geometry.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary/types.rs`
- tests in `.../infill_boundary/tests/geometry.rs`, `top_fill.rs`, and `ordering.rs`, declared in `tests.rs` before RED
- aligned direct tests in `crates/ares-core/src/project_slice/tests/perimeters/classic/infill_boundary/direct.rs`, declared before RED

RED:

1. Add direct rectangle/donut literals that pin `not_filled_exp`, `min_perimeter_infill_spacing`, ordinary infill, and both no-overlap branches. Add an arc-fitting-enabled path whose fixed points differ under raw versus one-fifth tolerance; assert O15 uses raw `m_scaled_resolution` while existing O1 tests keep the adjusted value.
2. Use odd minimum spacing to distinguish floating `/ 2.` from integer `/ 2` in the two source calls.
3. Add empty/nonempty `top_fills` cases. Pin top offset's `i64 -> f64 -> f32` integer external-spacing half, `fill_clip` intersection, conditional `i64 -> f64 -> f32` top-overlap expansion, ordinary-infill union, and unexpanded top geometry in `fill_no_overlap`.
4. Add narrow input that collapses to empty and empty remaining input that still preserves source geometry-call order.
5. Add multiple source surfaces with contour/hole geometry and pin internal fill-surface order, exact `RegionSurface::internal` defaults, and `fill_no_overlap` append order.
6. Before production code, add aligned direct RED cases for empty/nonempty top fills, all three loop-number branches, absent upper slices, empty remaining input, both no-overlap branches, and source-surface order without fixture-identity selection.
7. Add a source-order event/counter test for: every per-expolygon simplification repair; one aggregate union; ordinary offset2; mandatory top offset and intersection even when original `top_fills` is empty; conditional top-overlap offset/union selected from the original vector; logical internal-surface append; inactive extra-perimeter guard; selected no-overlap offset; conditional final union with unexpanded top geometry. An empty-top-fill injected failure must detect skipped mandatory calls.
8. Independently cover simplification repair, aggregate `union_ex(pp)`, ordinary offset, top-fill offset/intersection, top-overlap offset/union, each no-overlap branch, and final top union. Use naturally invalid geometry where source-reachable; where prior successful range validation makes a later failure mathematically unreachable, use one narrow operation-specific `#[cfg(test)]` hook solely to prove call order, stable mapping, rollback, and precedence without changing production-build behavior. Every case asserts the exact stable geometry error.

GREEN:

1. Simplify every O14 `remaining` expolygon to polygons in order, then execute one aggregate NonZero union.
2. Compute the 0.6 solid spacing with source `f64` multiplication and truncation.
3. Call existing `offset2_ex`, `offset_expolygons`, `intersection_ex`, and `union_expolygons` with the exact source expressions, including `float(double(external_spacing / 2))`, `float(double(top_infill_peri_overlap))`, `float(-inset - min_spacing / 2.)`, `float(min_spacing / 2 - overlap)`, and `float(double(-inset - overlap))`; use default Miter join and miter limit `3.0`, and do not merge or skip empty calls.
4. Create ordered `RegionSurface::internal` values from ordinary/top-unioned infill.
5. Compute and append no-overlap geometry only after the internal surfaces are staged.
6. Map every reached geometry failure at the stage boundary without changing lower-level Clipper errors.
7. Rerun focused geometry and all offset/boolean/simplification regressions GREEN.

## Task 4 — Add aligned successor, transactional ownership, and lifecycle

Upstream boundary: the per-island output append order at `PerimeterGenerator.cpp:1668-1691` and the exact six-operand source guard plus activated body boundary in `PerimeterGenerator.cpp:1087-1114`. Rust destination: `classic::infill_boundary` successor and project lifecycle.

Files:

- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/infill_boundary/types.rs`
- `crates/ares-core/src/project_slice/perimeters.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/incomplete_sink.rs`
- lifecycle/ownership/precedence/cleanup tests under `crates/ares-core/src/project_slice/tests/perimeters/classic/infill_boundary/`

RED:

1. Add aligned object/record/surface shape tests with empty middle slots and multiple surfaces. Require record-level ordered fill surfaces and no-overlap polygons.
2. Capture O14 gap entity-vector/path/point allocations, O14 `remaining` vector plus every contour/hole point allocation, O13 medial point/width, O11 polygon, O10 collection/entity/path/point, and boxed O5 allocation addresses; require identity after success.
3. Build a table-driven cross-product pairing every reachable numeric-preflight failure class before and after simplification, aggregate union, offset, intersection, and later union candidates. Assert the exact numeric error always wins and zero O15 geometry events occur.
4. Add an inactive-extra-perimeter guard RED matrix. With `extra_perimeters_on_overhangs=true`, independently falsify lower-slice presence, `detect_overhang_wall`, positive `wall_loops`, and `layer_id > raft_layers`; assert an inactive guard event between logical append and no-overlap work, subject to all other preflight rules. Assert the full non-spiral conjunction is rejected by O1 and spiral remains independently rejected.
5. Add exact rollback probes and 64 KiB cleanup for success, every reachable numeric error, and every independently covered geometry operation. Deepen both 10,000-node predecessor families—the traversal-seed tree and hierarchy-loop tree—in every case.
6. Add a lifecycle counter that fails until the public path calls O15 exactly once and still returns `ProjectSlicingIncomplete`.

GREEN:

1. Stage all records and surfaces into sidecars before moving O14 ownership.
2. On success, move O14 surfaces field-for-field, append record-level fill outputs, and retain the exact boxed O5 predecessor.
3. On failure, iteratively consume every O14 surface/object plus the boxed predecessor; add iterative O15 success sinks for the public incomplete boundary.
4. Reconstruct the source guard from aligned `spiral_mode`, lower-slice presence, `detect_overhang_wall`, `extra_perimeters_on_overhangs`, `wall_loops`, `layer_id`, and `raft_layers`. Rely on approved O1 preflight to reject the non-spiral activating conjunction and its independent spiral rule. Keep inactive true-option cases accepted only subject to all other Classic preflight rules; do not implement the activated body or a fallback.
5. Wire `prepare_post_classic_infill_boundary` after O14 in `perimeters.rs`, switch `slice_project`'s terminal consumer to O15, and preserve the exact incomplete error.
6. Rerun lifecycle, ownership, precedence, cleanup, and O14-O5 regression filters GREEN.

## Task 5 — Pin the KSR checkpoint

Files:

- `crates/ares-core/src/project_slice/tests/perimeters/classic/infill_boundary/ksr.rs`
- the already declared `.../classic/infill_boundary.rs` test root

RED/GREEN:

1. Add the checksum placeholder before the complete stage passes.
2. Define a source-shaped KSR checksum over object/record/surface delimiters, moved O14 content, overlap sidecar values, internal-surface kind/default metadata and polygon order, no-overlap polygon order, and predecessor facts.
3. Run twice from independently parsed project bytes; pin only the literal checksum after a clean implementation run and remove temporary output.
4. Rerun the option, LargeBed, and direct anti-hardcoding tests introduced in Tasks 2-3 together with the checkpoint.
5. Keep the CLI golden ignored/incomplete because `PerimeterGenerator.cpp:1695` onward and G-code emission remain deferred. Record this as a milestone limitation, not as Orca E2E evidence, and identify the next source boundary required before the Ares-versus-Orca golden can run.

## Task 6 — Documentation, full verification, independent review, and ship

1. Update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, and synchronized spec/plan status. Record included and deferred upstream behavior and the next source boundary.
2. Run focused O15, geometry simplification/offset/boolean, and O14-O5 regression filters.
3. Run:
   - `cargo nextest run --workspace`
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`
   - `cargo fmt --all -- --check`
   - `git diff --check`
   - changed-Rust LOC, forbidden-pattern, source-pinning, dependency, and staged-file audits.
4. Record the existing Tier-1 Windows, macOS, and Linux CI matrix as cross-platform evidence in addition to the local browser-WASM gates.
5. Start an independent read-only six-dimension implementation review covering requirements completeness, source/logic correctness, boundary cases, code quality, tests, and actual runtime evidence.
6. Run the separate read-only OpenCode review against the same final diff. A fix from either reviewer returns the work to the main thread; rerun affected and full gates, then rerun **both** reviews. Do not commit until both literal approvals cover the identical diff and validation evidence.
7. Create small Conventional Commits for geometry, slicing, and docs; exclude `.pi-subagents/` and generated logs; push `main`, verify remote HEAD, and verify a clean worktree with no staged files. If final evidence claims runtime Tier-1 execution rather than only the committed matrix definition, verify the Windows/macOS/Linux workflow for that exact pushed HEAD.

## Stop condition

Stop O15 only when every included `PerimeterGenerator.cpp:1628-1691` behavior is represented by source-derived typed state, literal tests and KSR evidence are green, ownership/error/stack contracts pass, both independent reviewers approve, documentation is synchronized, commits are pushed, and the lifecycle remains explicitly incomplete at the next cited source boundary.
