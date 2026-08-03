# Task 22O.17 — Surface-type detection and clipped fill transfer Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o17-surface-type-detection.md`

## Status

Implemented and locally validated after approved independent/OpenCode specification and plan reviews. The O17 checksum is `-126362407653399901571400348049652748978`, with totals `[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294, 113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011]`; 43 focused O17 tests, 178 O1-O17 regressions, and 5,597 workspace tests with 2 skipped pass together with strict Clippy, workspace/native and both WASM checks, formatting, diff, LOC, forbidden-pattern, source-pinning, dependency, and staging audits. The final independent six-dimensional implementation rereview and OpenCode rereview both returned `VERDICT: APPROVE`.

## Validation contract

Port the first complete KSR-reached `PrintObject::prepare_infill` mutation at fixed Orca commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` from Ares baseline `a5ca28c2b1d3e09da40827e3f302f53da9e89827`: classify untyped region slices through `detect_surfaces_type`, rebuild typed fill surfaces through `slices_to_fill_surfaces_clipped`, preserve executable overload quirks, ordering, metadata, casts, ownership, stack safety, and Tier-1 portability, then stop after the equivalent of `m_typed_slices = true` at `PrintObject.cpp:1923` before `LayerRegion::prepare_fill_surfaces`.

## Gate 0 — Freeze the reviewed contract

1. Keep the specification synchronized with all independent/OpenCode corrections.
2. Require literal `VERDICT: APPROVE` from both specification reviewers.
3. Review this plan with both reviewers; any revision returns to both.
4. Before RED, assert exact Ares HEAD `a5ca28c2b1d3e09da40827e3f302f53da9e89827` and Orca HEAD `8500fcdccaa10b5099ac20d252af3a7c560046f1`; stop on drift.
5. Add no Rust source or tests until both plan approvals cover the same spec/plan.

## Task 1 — Extend source surface kinds and port one-record classification

Upstream boundary: `PrintObject.cpp:1572-1708`, `Surface.hpp:8-283`, `SurfaceCollection.hpp:11-81`, `Flow.hpp:55-63`, and directly reached Clipper helpers. Rust destination: private region-surface vocabulary and focused O17 geometry modules.

Exact files and budgets:

- `crates/ares-core/src/project_slice/region_slices.rs` — enum/constructors only, target at most 390 LOC
- `crates/ares-core/src/project_slice.rs` — add the `prepare_infill` module declaration only in Task 1
- `crates/ares-core/src/project_slice/prepare_infill.rs` — new orchestrator/module root, at most 120 LOC
- `crates/ares-core/src/project_slice/prepare_infill/surface_type_detection.rs` — new successor root, at most 260 LOC
- new production children `surface_type_detection/types.rs`, `geometry.rs`, `cracks.rs`, `preflight.rs`, and `stage.rs`, each at most 300 LOC
- new direct-test root `surface_type_detection/tests.rs` and Task 1 shards `tests/classification.rs` and `tests/cracks.rs`, each at most 350 LOC
- `crates/ares-core/src/geometry/clipper.rs` and `crates/ares-core/src/geometry.rs` — re-export the already implemented `offset_paths_tree`/`PolyTree` boundary only; no offset algorithm changes

RED:

1. Extend tests to require `Top = 0`, `Bottom = 1`, `BottomBridge = 2`, `Internal = 4`, exhaustive predicates, and `BottomBridge`-only bridge classification.
2. Add direct source-shaped record tests for first, last, interior, unsupported and fully supported bottom, narrow opening collapse, multiple surfaces and holes, empty neighbors, one-layer top/bottom overlap, and both branches after singleton crack erosion: retain a qualifying crack in bottom versus remove it from bottom.
3. Pin arithmetic and overload traps before implementation: widths that distinguish exact offset `i64→f32` cast/divide from alternatives; widths that distinguish crack-threshold integer negation then `i64→f64` multiply and `f32` narrowing; equality `bottom.area() == crack.area() * 2.0` versus strictly greater; a sub-10-unit containment-gap case distinguishing the dropped-safety overload; a holed singleton-versus-collection offset case; and geometry distinguishing clip-only 10-unit safety plus miter limit `3.0`.
4. Pin fresh metadata defaults, first-layer bottom metadata retention, final-top retention without overlap, and final-top reconstruction/defaulting after overlap.
5. Add a combined two-surface/hole/repeated-kind classification test that pins each surface's contour then holes, complete top paths before complete bottom paths, `Internal`→`Top`→`Bottom` slice append order, stable same-kind order, and exact resulting expolygon order.
6. Add valid-minimal and near/out-of-range coordinate cases; require the exact stable O17 geometry error text for every rejected coordinate.
7. Run the focused filter and record RED because required kinds and classifier do not exist.

GREEN:

1. Add only the four required private surface kinds and update every exhaustive match.
2. Build current untyped expolygons by copying source region surfaces in order. Compute `offset` as `(external_width as f32) / 10.0_f32` and crack threshold as `((-external_width) as f64 * 1.5) as f32`.
3. Implement terminal clone/retag and nonterminal 10-unit clip-only safety difference followed by miter/`3.0` opening. Call the existing first-stage `offset_expolygons_paths` and second-stage `offset_paths_tree` explicitly so shrink and expand are individually fallible/testable; expose only the missing crate-private production helper rather than duplicating offset logic. Preserve source call order and Clipper/PolyTree order; never sort.
4. Implement cracks in a separate module with exact effective overloads: ordinary intersection; singleton erosion; dropped-safety ordinary containment difference; strict area; ordinary bottom-minus-crack then collection erosion; singleton expansion; ordinary bottom subtraction; ordinary top-minus-bottom.
5. Implement internal difference from previous expolygons minus stable top-then-bottom polygon paths, then append `Internal`, `Top`, `Bottom`.
6. Map every Clipper failure to `InvalidInput("surface-type detection geometry is outside the supported Clipper range")`.
7. Rerun every named direct geometry RED test GREEN.

## Task 2 — Rebuild typed fills and add transactional project successor

Upstream boundary: `PrintObject.cpp:1520-1923`, `LayerRegion.cpp:63-80`, `Print.hpp:429`, and `PrintConfig.hpp` support/extra-bridge enums. Rust destination: `PreparedPostSurfaceTypeDetection` after O16.

Exact files and budgets:

- the Task 1 production/test roots and children listed above, within their fixed budgets; add direct shards `surface_type_detection/tests/clipped_fill.rs`, `tests/preflight.rs`, and `tests/transaction.rs`, each at most 350 LOC
- `crates/ares-core/src/project_slice/capabilities.rs` and `crates/ares-core/src/project_slice/tests/capabilities.rs`, each below 400 LOC
- new integration roots `crates/ares-core/src/project_slice/tests/prepare_infill.rs` and `tests/prepare_infill/surface_type_detection.rs`, each at most 100 LOC
- new integration shards `tests/prepare_infill/surface_type_detection/fixture.rs`, `options.rs`, `ownership.rs`, `transaction.rs`, and `metamorphic.rs`, each at most 350 LOC
- module declaration only in `crates/ares-core/src/project_slice/tests.rs`; do not grow the existing 386-line `tests/support.rs`

RED:

1. Add preflight tests that remove only the obsolete early `enable_support`/`enforce_support_layers` capability failures while retaining all unrelated capability gates.
2. Require key-major O17 precedence: any `interface_shells` error before any active extra-bridge error, both before geometry across all objects. Require disabled/internal-only extra-bridge values to pass and external/apply-all to fail stably.
3. Unit-test the literal support predicate across no support, enforced support, automatic/manual types, zero/nonzero Z distance, NormalAuto `bridge_no_support`, and all three TreeAuto conjuncts.
4. Add real 3MF support-only mutations that reach O17: one selects `Bottom`, one selects `BottomBridge`; assert coordinates, metadata, predecessor identity, and unrelated O16 fields remain unchanged.
5. Add clipped-fill tests requiring clear/rebuild semantics, actual intersection clipping to `fill_expolygons`, numeric kind order, stable intra-kind order, contour/hole order, default metadata, empty typed groups, empty fill boundaries, and no aliasing with moved `fill_expolygons`.
6. Capture O16 perimeter, thin-fill, fill-boundary, no-overlap, old fill-surface, and boxed-predecessor pointers. Require named untouched allocations to move; old fill surfaces to be consumed; rebuilt fills to be fresh.
7. Add a failure-hook matrix for: top safety difference; top shrink; top expand; bottom safety difference; bottom shrink; bottom expand; crack intersection; singleton erosion; containment difference; residual difference; collection erosion; singleton expansion; bottom subtraction; top difference; internal difference; and each reachable nonempty kind fill intersection. For every matrix entry require a reached-event assertion, exact error text, unchanged captured O16 allocations, no successor/partial output, and 64-KiB iterative disposal with both 10,000-node predecessor families.
8. Before project-successor GREEN, add and run failing real-archive metamorphic tests in `metamorphic.rs`: semantically identical ZIP entry reordering/compression plus non-slicing metadata/name change must be identical, while an exact component X-scale mutation must change the checksum and satisfy `scaled_first_layer_span = 2 * baseline_span + 300000`, preserving the fixed typed `0.15 mm` elephant-foot compensation on both sides.

GREEN:

1. Remove only `enable_support` and nonzero `enforce_support_layers` rejections from `capabilities.rs`; update tests to expect the honest downstream incomplete result rather than support completion.
2. Add whole-project typed preflight over resolved objects with the exact key order. Compute bottom kind with the source predicate; do not reject or normalize support operands.
3. Reach original region/whole-layer slices and aligned `external_width` only through the boxed traversal→hierarchy→onion→top-split→Classic-prelude/input path. Assert O16/input slot and identity alignment.
4. Stage every classified slice and clipped fill while borrowing O16. Rebuild fills by stable typed-slice bucketing, numeric kind visits, ordinary intersections against unchanged `fill_expolygons`, and fresh surfaces.
5. Only after every object succeeds, destructure O16 and move unchanged perimeters, thin fills, fill boundaries, no-overlap fields, and exact boxed predecessor into the successor. Replace old fill surfaces with staged fills and add staged typed slices.
6. On preflight or geometry failure, consume O16 iteratively. Expose test-only invocation, event, and failure hooks only under `#[cfg(test)]`.
7. Rerun direct/preflight/option/ownership/transaction/metamorphic tests plus O16-O10 regressions GREEN; preserve the recorded pre-GREEN metamorphic failure evidence.

## Task 3 — Wire lifecycle and iterative cleanup

Upstream boundary: `PrintObject.cpp:560-584,1904-1923`; exact next stop `PrintObject.cpp:587-592` / `LayerRegion.cpp:935-973`.

Exact files and budgets:

- `crates/ares-core/src/project_slice.rs`, module/wiring changes only, below 400 LOC
- `crates/ares-core/src/project_slice/prepare_infill.rs`, within its 120-LOC budget
- `crates/ares-core/src/project_slice/incomplete_sink.rs`, add only the child declaration and keep at most 399 LOC
- new `crates/ares-core/src/project_slice/incomplete_sink/surface_type_detection.rs`, all O17 observation/dispatch, at most 120 LOC
- new integration shards `tests/prepare_infill/surface_type_detection/lifecycle.rs` and `cleanup.rs`, each at most 350 LOC, declared by the fixed test root

RED:

1. Add lifecycle counters that require public slicing to invoke O17 exactly once and still return `ProjectSlicingIncomplete`.
2. Prove earlier spiral and non-`none` counterbore failures leave O17 invocation count zero; O17 interface/extra errors invoke O17 once but no geometry.
3. On 64 KiB threads, deepen both predecessor tree families to 10,000 nodes and exercise success/incomplete cleanup, O17 preflight failure, and every individual failure-hook matrix entry, including each reachable kind-specific fill call.
4. Assert the 398-line `incomplete_sink.rs` parent remains below 400 LOC.

GREEN:

1. Add a focused `prepare_post_surface_type_detection(project)` successor after O16.
2. Move the public incomplete stop to the O17 successor without marking `prepare_infill` done.
3. Add `incomplete_sink/surface_type_detection.rs` and move O17 observation/dispatch into it unconditionally; reuse the iterative boxed-traversal sink and keep the 398-line parent below 400 LOC.
4. Rerun lifecycle, precedence, and constrained-stack tests GREEN.

## Task 4 — Pin KSR and anti-hardcoding metamorphic evidence

Exact files and budgets:

- new `crates/ares-core/src/project_slice/tests/prepare_infill/surface_type_detection/ksr.rs`, at most 350 LOC
- existing `fixture.rs` and `metamorphic.rs` from Task 2, each within its fixed budget; no edits to `tests/support.rs`

Characterization after the already recorded Task 1/2 RED→GREEN cycle:

1. Parse original KSR bytes independently twice. Guard the exact O16 checksum first, then hash O17 object/slot delimiters, retained source indices, all unchanged O16 fields, typed slice/fill kinds and metadata, and complete contour/hole/point order.
2. Freeze only after a clean implementation run: literal O17 checksum plus totals by object, slot, record, slice/fill kind, contour, hole, point, and unchanged O16 output counts. Remove diagnostics.
3. Rerun the already RED-driven ZIP repack/non-slicing rename and exact component-scale relation tests against the frozen structure; do not add metamorphic behavior after GREEN.
4. Keep source-shaped overload/order tests as the independent parity evidence. Do not read the reference G-code or branch by fixture name, hash, layer count, or geometry identity.
5. Keep the CLI golden incomplete/ignored. Record `LayerRegion::prepare_fill_surfaces` as the next source boundary.

## Task 5 — Documentation, verification, independent review, and ship

1. Update architecture, roadmap, spec, and plan with exact O17 boundary, capability-gate relocation, supported option envelope, dropped-safety overload quirk, ordering/metadata/ownership rules, KSR checksum/totals, and next `LayerRegion::prepare_fill_surfaces` boundary.
2. Run focused O17 and O16-O10 regressions.
3. Run:
   - `cargo nextest run --workspace`
   - `cargo check --workspace --all-targets`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`
   - `cargo fmt --all -- --check`
   - `git diff --check`
   - an all-Rust-source/test LOC audit proving every `.rs` file is below 400 LOC, not only changed files;
   - diff-scoped zero-addition audits for `unsafe`, `include!`, `include_bytes!`, binary payloads, source-text/hash/line pins, reference-G-code reads, Orca runtime/FFI, fixture name/hash/layer-count/geometry-identity branches, and broad lint allowances;
   - `git diff -- Cargo.toml Cargo.lock` must be empty; dependency and staging audits must show no manifest/lock changes and no `.pi-subagents/`/generated logs staged.
4. Document that O17 adds no public API, persisted format, migration, or compatibility layer; rollback restores the O16 terminal and capability gates while removing only O17 state/wiring/tests/docs.
5. Record the unchanged Windows/macOS/Linux CI matrix as native Tier-1 evidence; do not claim current-tree CI execution.
6. Run an independent read-only six-dimensional implementation review and separate OpenCode review against the same diff/evidence. Any finding returns to the main thread; rerun affected/full gates and both reviewers after fixes.
7. After both literal approvals, synchronize final status, create small Conventional Commits for slicing/docs, exclude `.pi-subagents/` and generated logs, push `main`, and verify remote HEAD plus clean worktree.

## Stop condition

Stop O17 only when source surface classification and clipped fill transfer are complete through `m_typed_slices = true`, all executable quirks/casts/orders/metadata and ownership are proven, support options reach only the honest incomplete boundary, KSR plus real-archive metamorphic tests pass, public slicing reaches O17 once and remains incomplete before `prepare_fill_surfaces`, all validation gates pass, both final reviewers approve the identical diff, documentation is synchronized, and commits are pushed.
