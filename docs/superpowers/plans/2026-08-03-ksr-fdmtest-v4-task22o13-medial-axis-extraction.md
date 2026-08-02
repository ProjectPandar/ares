# Task 22O.13 — Classic Gap Medial-Axis Extraction Plan

Spec: `docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o13-medial-axis-extraction.md`

## Validation contract

The successor must reproduce the valid-diagram production path reached by Orca `PerimeterGenerator.cpp:1586`, preserve O11/O10/O5 ownership and ordering, remain transactional and stack-safe, compile on WASM, and leave the public lifecycle at `ProjectSlicingIncomplete`. Every implementation slice below is RED before GREEN.

## Task 1 — Pin, qualify, and adapt segment Voronoi topology

Files:

- `Cargo.toml`
- `crates/ares-core/Cargo.toml`
- `Cargo.lock`
- `THIRD_PARTY_NOTICES.md`
- `crates/ares-core/src/geometry/tests/medial_axis.rs`
- `crates/ares-core/src/geometry/tests/medial_axis/diagram.rs`
- module declaration in `crates/ares-core/src/geometry/tests.rs`
- `crates/ares-core/src/geometry/medial_axis/diagram.rs`

RED:

1. Declare the test modules and add fixed integer-site tests that fail until the adapter exists.
2. Pin literal edge enumeration IDs, even/odd twin pairs, cell IDs/source indices/source categories, vertex endpoints, primary/finite flags, `rot_next`, face `next`, and incident-cell cycles.
3. Add a fixed point-point case whose literal face/rotation order will drive annotation queue propagation.
4. Run `cargo nextest run -p ares-core geometry::tests::medial_axis::diagram` and record the expected failure.

GREEN:

1. Add `boostvoronoi = { version = "=0.12.1", default-features = false }` to workspace dependencies and consume it from `ares-core`.
2. Add the source-order adapter and the minimum topology accessors required by the tests.
3. Update `THIRD_PARTY_NOTICES.md` for the BSL-1.0 crate and record that its uncalled filesystem-reader utility cannot be feature-disabled.
4. Rerun the focused test and both WASM checks.

## Task 2 — Add source-shaped line and polygon helpers

Files:

- `crates/ares-core/src/geometry/line.rs`
- `crates/ares-core/src/geometry/polygon.rs`
- `crates/ares-core/src/geometry/expolygon.rs`
- `crates/ares-core/src/geometry/polyline.rs`
- separate matching files under `crates/ares-core/src/geometry/tests/`

For each numbered slice, write the literal failing test, run its focused Nextest filter, add only the production behavior needed, and rerun GREEN:

1. `geometry::tests::line`: finite segment intersection, parallel rejection, source-shaped truncating and rounding call sites, distance/projection, length, and orientation.
2. `geometry::tests::polygon`: adjacent-then-closing line order, distinct closing-first intersection order, projection, and strict boundary predicate.
3. `geometry::tests::expolygon`: contour-then-hole lines and contour/hole boundary behavior at Normal/LargeBed epsilon scales.
4. `geometry::tests::polyline`: adjacent stored-segment `ThickPolyline::length` summation.

## Task 3 — Port the valid-diagram medial-axis core

Production files split below 400 LOC:

- `crates/ares-core/src/geometry/medial_axis.rs`
- `crates/ares-core/src/geometry/medial_axis/diagram.rs`
- `crates/ares-core/src/geometry/medial_axis/annotate.rs`
- `crates/ares-core/src/geometry/medial_axis/annotate/propagate.rs`
- `crates/ares-core/src/geometry/medial_axis/validate.rs`
- `crates/ares-core/src/geometry/medial_axis/chaining.rs`
- exports in `crates/ares-core/src/geometry.rs`

Test files:

- `crates/ares-core/src/geometry/tests/medial_axis/annotate.rs`
- `crates/ares-core/src/geometry/tests/medial_axis/validate.rs`
- `crates/ares-core/src/geometry/tests/medial_axis/chaining.rs`
- `crates/ares-core/src/geometry/tests/medial_axis/postprocess.rs`
- `crates/ares-core/src/geometry/tests/medial_axis/error.rs`

RED/GREEN slices:

1. Add fixed Orca-derived literal vertex/edge/cell categories and eligible-edge assertions for contour, hole, and epsilon boundaries, including point-point queue propagation; run `cargo nextest run -p ares-core geometry::tests::medial_axis::annotate`, then port annotation.
2. Add failing direction-sensitive literal width and rejection cases; run the `validate` filter, then port `validate_edge` with fractional annotation coordinates and source `std::round` conversion for Voronoi `Point(double, double)` sites.
3. Add failing branch, single-neighbor, zero-neighbor, multi-neighbor, reverse-width, and closed-loop literal tests; run the `chaining` filter, then port seed traversal.
4. Add failing literal rectangle, narrow concave region, hole, endpoint extension, two-point midpoint, short removal, greedy reconnect, global-width, Normal/LargeBed scale, and empty-input tests. Assert complete `ThickPolyline` points, width bit patterns, endpoint flags, and order. Run the `postprocess` filter, then port `ExPolygon::medial_axis` post-processing.
5. Add a failing geometry-level test passing an expolygon with a repeated adjacent point and requiring the typed zero-length-source error; run `cargo nextest run -p ares-core geometry::tests::medial_axis::error` and record RED. Then add the source-precondition check before invoking the dependency and rerun GREEN. This provides a deterministic failure without fixture branching or test-only production behavior.
6. Map builder failures, zero-length source segments, and adapter topology-invariant failures to the typed geometry error required by the RED test. Do not claim completed-diagram validity detection or add repair/fallback.

## Task 4 — Add the O13 Classic successor test-first

Production files:

- `crates/ares-core/src/project_slice/perimeters/classic/medial_gap.rs`
- `crates/ares-core/src/project_slice/perimeters/classic/medial_gap/types.rs`
- `crates/ares-core/src/project_slice/perimeters/classic.rs`
- `crates/ares-core/src/project_slice/perimeters.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/incomplete_sink.rs`

Test files:

- `crates/ares-core/src/project_slice/perimeters/classic/medial_gap/tests.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/medial_gap.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/medial_gap/direct.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/medial_gap/lifecycle.rs`
- `crates/ares-core/src/project_slice/tests/perimeters/classic/medial_gap/support.rs`

RED:

1. Add direct tests for exact aligned object/record/surface types, stored expolygon aggregation, `None` versus `Some(empty)`, and the complete literal matrix from Task 3.
2. Add ownership tests capturing O11 expolygon buffers, nested O10 collection/entity/path/point buffers, and boxed O5 predecessor addresses before `finish`; assert exact identity/content after success.
3. Build a synthetic O11 surface containing a repeated adjacent point so its source lines contain a zero-length segment. Assert the exact mapped error `InvalidInput("Classic medial-axis Voronoi construction failed")`, no successor/partial surface is observable, and predecessor drop probes remain untouched until the failure cleanup path runs.
4. Add constrained-stack success/error tests proving iterative destruction, plus a public-lifecycle test that fails until O13 is invoked and still returns `ProjectSlicingIncomplete`.
5. Run `cargo nextest run -p ares-core project_slice::tests::perimeters::classic::medial_gap` and record RED.

GREEN:

1. Define aligned successors. Each surface moves the complete O11 surface and adds `medial: Option<MedialGapDomain>`; `MedialGapDomain` owns the moved `PreMedialGapDomain` plus stored-order aggregated polylines.
2. Flow `CoordinateScale` from the boxed traversal predecessor, stage every object/record/surface/expolygon before moving O11 ownership, and preserve `None` versus `Some(empty)`.
3. Map errors to the exact stable `SliceError`, iteratively consume the untouched O11 state on failure, and expose no partial successor.
4. Wire `prepare_post_classic_medial_gap` into the actual public lifecycle and terminal sink without advancing into gap extrusion.
5. Rerun the focused filter GREEN.

## Task 5 — Add and pin the KSR structural oracle

Files:

- `crates/ares-core/src/project_slice/tests/perimeters/classic/medial_gap/ksr.rs`

RED/GREEN:

1. Add a deterministic checksum that mixes object/record/surface delimiters, each surface `source_index`, every `None`/`Some` marker, ordered predecessor `min`/`max`, explicit expolygon counts, contour point counts, hole counts, each hole point count, every ordered contour/hole point, every polyline delimiter and point count, point coordinate, width count and each `f64::to_bits`, endpoint flag, and output order.
2. First assert repeatability and O11/O10 predecessor preservation while leaving an explicit failing placeholder for the final checksum; run only the KSR filter and record RED.
3. Run once to obtain the literal, pin it, remove all temporary printing/output, and rerun GREEN.

## Task 6 — Documentation, verification, and review loop

1. Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` with the exact included/deferred boundary and evidence.
2. Run focused O13 and O11–O5 regressions.
3. Run:
   - `cargo nextest run --workspace`
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo check --workspace`
   - `cargo check -p ares-core --target wasm32-unknown-unknown`
   - `cargo check -p ares-wasm --target wasm32-unknown-unknown`
   - `cargo fmt --all -- --check`
   - `git diff --check`
4. Audit changed Rust/test files for `<400` LOC and forbidden patterns.
5. Launch fresh independent six-dimension and external-model implementation reviewers. Apply their concrete fix list through one writer, rerun affected validation, and re-review all required reviewers until each returns approval.
6. Commit the reviewed milestone atomically and push only after all gates pass.
