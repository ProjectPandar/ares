# Task 22O.14 — Classic Variable-Width Gap Extrusion Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o14-gap-extrusion.md`

## Status

Implemented, validated, independently reviewed, and approved. The post-fix workspace run passed 5,491 tests with 2 skipped; strict Clippy, native/workspace checks, both WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, and staging audits passed. Independent Codex and OpenCode re-reviews both returned `VERDICT: APPROVE`. The paired spec is synchronized.

## Validation contract

The successor must reproduce Orca `PerimeterGenerator.cpp:1604-1624` and every directly reached helper in the reviewed spec, derive its threshold and flow only from aligned typed 3MF configuration, preserve O13/O11/O10/O5 order and surviving allocations, stage transactionally, remain stack-safe and WASM-portable, and leave public slicing at `ProjectSlicingIncomplete`. Every behavior slice below begins with literal RED tests, records the focused failure, then adds only the production implementation required for GREEN.

## Task 1 — Port Clipper open-butt offsets test-first

Upstream boundary: `ClipperUtils.hpp:21-31,336-338`, `ClipperUtils.cpp:267-293,333-357,412-428`, and Clipper 6 `deps_src/clipper/clipper.cpp:3371-3810`. Rust destination: the existing crate-private `geometry::clipper::offset` module. Closed polygons, other end types, and unrelated Clipper features remain unchanged/deferred.

Production files:

- `crates/ares-core/src/geometry/clipper/offset.rs`
- `crates/ares-core/src/geometry/clipper/offset/input.rs`
- `crates/ares-core/src/geometry/clipper/offset/generate.rs`
- `crates/ares-core/src/geometry/clipper/offset/execute.rs`
- exports in `crates/ares-core/src/geometry/clipper.rs` and `crates/ares-core/src/geometry.rs`

Test files:

- module declaration in `crates/ares-core/src/geometry/tests/clipper/offset.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset/open.rs`
- existing closed-offset tests remain unchanged

RED:

1. Add literal tests for a one-point square, a straight horizontal segment with exact butt-cap polygon points, left/right square bends, reversed input, and multiple open paths.
2. Add consecutive duplicate and shortest-edge tests with distances below, exactly equal to, and above `abs(delta * 0.005)`; equality must survive.
3. Add a multi-path literal that distinguishes one cleared engine and Positive cleanup per input path from the final aggregate NonZero union.
4. Add Normal/LargeBed-sized literal deltas and an out-of-range error case.
5. Run `cargo nextest run -p ares-core geometry::tests::clipper::offset::open` and record RED.

GREEN:

1. Add the minimum source-shaped `EndType::{ClosedPolygon, OpenButt}` state to each offset path; keep all existing closed calls selecting `ClosedPolygon`.
2. Add open input preparation without closed-end stripping or lowest-point participation. Reuse strict `points_are_near` and retain source one-point behavior.
3. Split raw generation by end type. Preserve the existing closed generator byte-for-byte in behavior and add open-butt forward joins, two-point end cap, reversed normals/reverse joins, and two-point start cap using existing fixed rounding and square-join code.
4. Add raw/open wrapper execution that clears/configures once per input path, applies per-path Positive cleanup, appends in order, then performs aggregate NonZero union. Keep line join Square, miter limit `0`, and shortest-edge factor `0.005` at the public crate-private wrapper.
5. Rerun the focused open test, all offset tests, and both WASM checks GREEN.

## Task 2 — Add source-exact Classic variable-width entities

Upstream boundary: `VariableWidth.hpp`, `VariableWidth.cpp:99-234` (`thick_polyline_to_extrusion_paths_2` and `variable_width`), reached `Flow.hpp`/`Flow.cpp`, `ExtrusionEntity.hpp`, `ExtrusionEntity.cpp:68-71,347-351`, and `ExtrusionEntityCollection.cpp:99-103`. Rust destination: a crate-private `classic::gap_extrusion` module using the existing fixed-coordinate Classic materialized path model. The older `thick_polyline_to_multi_path`, thin-wall call site, public f64 extrusion scaffold, and downstream G-code ratio remain deferred.

Production files, split below 400 LOC:

- add `GapFill` to `crates/ares-core/src/project_slice/perimeters/classic/materialize/types.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/materialize.rs` to make `ExtrusionRole`, `Point3`, and `Polyline3` available only within `crate::project_slice` production code
- create the minimal root `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion.rs` with Task 2 declarations/re-exports; Task 3 extends it with orchestration/types/preflight
- add the `gap_extrusion` declaration to `crates/ares-core/src/project_slice/perimeters/classic.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/entity.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/variable_width.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/coverage.rs`

Test files:

- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/tests.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/tests/variable_width.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/tests/coverage.rs`

RED/GREEN slices:

1. Add literal empty, open, multi-polyline, exact-closure, and reversed-endpoint-width entity-formation tests. Assert entity variants, every fixed `Point3`, `GapFill` role, path order, widths/heights, and `mm3_per_mm.to_bits()`; run the variable-width conversion filter RED. Pin expected metadata literals once from a clean production run, remove temporary output, and never recompute expected values through production helpers inside the test.
2. Add below/equal/above `float(0.05 / scale.factor())` width-tolerance tests plus a line-splitting case. Pin inserted truncating coordinates and paired widths.
3. Add nonzero below-epsilon, exact-epsilon, zero-length, total-length equality, and above-epsilon cases at Normal and LargeBed. Assert that skipped scan lines remain in later point/length/width ranges, and that grouped/final emission uses strict `>`.
4. Add grouped midpoint-width flush and asymmetric final `a_width` flush literals that produce observably different flow bits.
5. Implement the mutable line-vector/index loop without simplifying the source control flow. Reuse the reviewed `geometry::medial_axis::scaled_epsilon` for epsilon, derive tolerance by unrounded division, preserve normalized-vector operation/cast order, use aligned internal `Flow::with_width`, and map its error only at the stage boundary.
6. Add literal coverage tests for path and loop delegation, exact `float(scale_(width / 2)) + 10.f`, multiple entities, and error propagation. Implement ordered delegation through the Task 1 open-offset wrapper.
7. Rerun both focused modules GREEN and then `cargo nextest run -p ares-core project_slice::perimeters::classic::gap_extrusion`.

## Task 3 — Add aligned O14 state and whole-project typed preflight

Upstream boundary: `PerimeterGenerator.cpp:1604-1624`, consuming the Task 2 conversion/coverage helpers and the already reviewed `MultiPoint.cpp:48-56`, `Polyline.hpp:256-277`, and `Polyline.cpp:637-646` prerequisites. Rust destination: the aligned O13 successor under `classic::gap_extrusion`. Infill-boundary generation beginning at `PerimeterGenerator.cpp:1628` remains deferred.

Production files:

- extend `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/types.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/gap_extrusion/preflight.rs`
- declarations/re-exports in `crates/ares-core/src/project_slice/perimeters/classic.rs`

Test files:

- add the `gap_extrusion` parent declaration to `crates/ares-core/src/project_slice/tests/perimeters/classic.rs` before the first RED run
- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap_extrusion.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap_extrusion/direct.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap_extrusion/options.rs`
- shared helpers only when reused by at least two test files

RED:

1. Add direct aligned-state tests for O13 `None`, present-empty, filtered-empty, and retained non-empty medial domains, stable surface/polyline/entity order, separate gap collections, exact ordered covered-width polygons from the emitted entities, exact post-difference `remaining` expolygons, and cloned unchanged `last` when no retained polyline exists. Expected geometry is literal and is not recomputed through production helpers.
2. Add two aligned records with distinct effective `RegionOptions.filter_out_gap_fill` values and assert different strict-`<` results. Cover zero, equality, above, and fractional fixed-unit thresholds for both scales.
3. Add negative, NaN, positive infinity, and negative infinity option tests requiring exact `InvalidInput("invalid Orca option filter_out_gap_fill")`.
4. For each candidate failure class—invalid derived flow, open-offset range, and difference range—pair an invalid `filter_out_gap_fill` record both before and after the candidate failing record. Assert the exact option error always wins and test-only observation confirms no O14 conversion or Clipper invocation occurred anywhere in the project.
5. Run `cargo nextest run -p ares-core project_slice::tests::perimeters::classic::gap_extrusion` and record RED.

GREEN:

1. Define the aligned successor types from the spec: moved O13 fields, filtered `MedialGapDomain`, separate ordered `GapFillCollection`, and `remaining: Vec<ExPolygon>`.
2. Recover each record's typed region options and `solid_infill_flow` through the same aligned traversal/prelude seam used by O10. Assert every object/record/surface index alignment.
3. Implement a validation-only first pass over all records. Do not allocate variable-width or Clipper geometry in this pass.
4. In a second staging pass, compute keep masks, converted entities, ordered coverage, the aligned onion `last` clone, and ordinary `difference_ex`. Stage every project result before moving any O13 field.
5. Map derived flow and geometry failures to the exact spec errors. On success, apply stable keep masks to the moved O13 polyline vectors and attach staged output; on error, leave the prepared O13 state untouched for iterative cleanup.
6. Rerun focused direct/options tests GREEN.

## Task 4 — Wire lifecycle, rollback seam, ownership, and cleanup

Production files:

- `crates/ares-core/src/project_slice/perimeters.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/incomplete_sink.rs`

Test files:

- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap_extrusion/lifecycle.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap_extrusion/ownership.rs`
- test-only internal probes only where actual observable ownership/error precedence cannot otherwise be proven

RED:

1. Capture the boxed O5 address, O11 expolygon buffers, O10 collection/entity/path/point buffers, and every surviving O13 point/width buffer; require exact identity/content after O14 success.
2. Independently trigger invalid derived flow, open-offset range failure, and difference range failure through ordinary internal inputs. Assert exact errors, no successor, and predecessor/drop-probe timing.
3. Add success and every error path to a constrained thread-stack test with a 10k-node nested predecessor; use 64 KiB on Unix and 256 KiB on Windows specifically for the O14 open-offset/variable-width success-and-error execution frames, and require iterative cleanup. This is not a platform-wide Clipper baseline: other constrained-stack tests remain at 64 KiB unless their own target-runner evidence requires a change.
4. Add a lifecycle invocation counter test that fails until public `slice_project` reaches O14 exactly once and still returns `ProjectSlicingIncomplete`.
5. Add a rollback-contract regression that the O13 preparation entry remains callable, closed offsets are unchanged, and the O13 ownership graph can still be consumed independently.

GREEN:

1. Add `finish_classic_gap_extrusion` and `prepare_post_classic_gap_extrusion` after O13.
2. Add iterative O14 sinks for gap entities, remaining expolygons, filtered medial state, O10 state, and boxed O5 state. Avoid recursive destruction of nested predecessor trees.
3. Change the public terminal consumer from O13 to O14 and keep the exact incomplete result.
4. Rerun focused lifecycle/ownership, O13, and closed-offset regressions GREEN.

## Task 5 — Pin the in-memory KSR O14 structure

File:

- `crates/ares-core/src/project_slice/tests/perimeters/classic/gap_extrusion/ksr.rs`

RED/GREEN:

1. Add an explicit-delimiter checksum containing object/record/surface boundaries; `source_index`; typed filter value and scaled threshold bits; O13 `None`/`Some` markers; every retained polyline point, width bit, and endpoint; entity count and Path/Loop markers; path point count/XYZ, role, width/height/mm3 bits; every ordered coverage polygon returned from the emitted entities; remaining contour/hole counts and every coordinate; and output order.
2. Compare O13/O11/O10 predecessor structure and allocation identities separately; do not let one checksum stand in for ownership assertions.
3. Assert KSR has at least one generated gap entity and nontrivial remaining geometry. First leave a failing checksum placeholder and run only the KSR test RED.
4. Obtain the one literal checksum from the production result, remove all temporary output, pin it, and rerun twice GREEN for repeatability.

## Task 6 — Documentation, verification, independent implementation review, and ship

1. Update this plan status and the paired spec status in lockstep, then update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` exactly as required by the reviewed spec. Keep all behavior after line 1624 explicitly deferred.
2. Run focused O14, all geometry offset, and O13–O5 regression filters.
3. Run fresh gates and save logs under `target/parity/task22o14/`:
   - `cargo nextest run --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo check --workspace`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`
   - `cargo fmt --all -- --check`
   - `git diff --check`
4. Audit every changed Rust/test file for `<400` LOC; audit for `unsafe`, `include!`, `include_bytes!`, source/hash/line pinning, fixture branching, oracle payloads, and staged files.
5. Launch fresh read-only implementation reviewers for requirements completeness, logic, edge cases, code quality, test coverage, and actual execution plus the required default-model OpenCode review. Convert every concrete finding into a fix list, apply fixes through the single main writer, rerun affected gates, and rerun all required reviewers until each returns the literal `VERDICT: APPROVE`.
6. Inspect the final diff and authoritative outputs. Commit in atomic Conventional Commit units, exclude `.pi-subagents/` and generated logs, push `main`, and verify `origin/main` equals local HEAD.
