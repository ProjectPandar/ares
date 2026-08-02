# Task 22O.13 — Classic Gap Medial-Axis Extraction Spec

## Status

Implemented and validated after approved Task 22O.12; the final review-fix cycle is pending only independent reapproval.

## Upstream source boundary

Pinned upstream: OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the behavior reached by `PerimeterGenerator.cpp:1586`:

- `ExPolygon.cpp:261-369` — `ExPolygon::medial_axis(double, double, ThickPolylines*)`;
- `Geometry/MedialAxis.hpp`;
- production behavior in `Geometry/MedialAxis.cpp:458-707`;
- the segment-Voronoi topology consumed by that code from `Geometry/Voronoi.hpp` and Boost.Polygon Voronoi;
- `Geometry/VoronoiOffset.hpp` and the reached inside/outside annotation path in `Geometry/VoronoiOffset.cpp:646-971`, including its directly reached category/site/contour helpers;
- directly reached `Line.hpp`, `Line.cpp:53-57`, `MultiPoint.hpp:43`, `MultiPoint.cpp:48-56`, `Polygon.hpp:68-69,196-205`, `Polygon.cpp` point-projection helpers and `158-169`, `ExPolygon.cpp:119-128,380-385`, and `Polyline.hpp`/`Polyline.cpp` helpers.

The Rust destination is `ares-core::geometry::medial_axis` plus a new Classic successor stage after `classic::gap_domain`.

## Included behavior

1. Construct the segment sites in exact `ExPolygon::lines()` order: contour first, followed by each hole; within each polygon, stored adjacent edges come first and the closing edge comes last. The distinct `Polygon::intersection` helper retains its own closing-edge-first search order.
2. Build a Boost.Polygon-compatible integer segment Voronoi diagram using the pinned pure-Rust `boostvoronoi = 0.12.1` port.
3. Preserve and verify directed half-edge enumeration, consecutive even/odd twin pairing, source indices/categories, vertex endpoints, primary/finite flags, twin traversal, and rotational-neighbor order from `MedialAxis::build` and `process_edge_neighbors`.
4. Port the reached `Voronoi::annotate_inside_outside` semantics rather than substituting direct point-in-polygon classification: reset typed vertex/edge/cell categories, identify on-site and secondary-edge contour vertices, seed infinite edges, classify segment-related finite edges by source-line side tests, and propagate the remaining point-point categories in the same stored/queue order. `MedialAxis::build` accepts an edge only when either endpoint category is exactly `Inside`.
5. Port `validate_edge` exactly for valid Voronoi diagrams:
   - convert Voronoi `Point(double, double)` sites used by seed/growth and validation `Line` construction with source `std::round` semantics (half away from zero); endpoint-extension Eigen `cast<coord_t>` conversions remain truncation toward zero;
   - compute endpoint widths as twice the distance to the source segment or source endpoint;
   - preserve the segment-orientation fold, `PI / 8` gate, scaled epsilon gates, `min_width`/`max_width` predicates, and directed width swapping;
   - preserve source line and cell ordering.
6. Build `ThickPolyline` values in seed-edge order, consuming exactly one active neighbor, handling zero/multiple neighbors, forward/backward growth, width order, endpoint flags, and closed-loop endpoint suppression.
7. Port `ExPolygon::medial_axis` post-processing:
   - pass `CoordinateScale` explicitly and derive source `SCALED_EPSILON` as `1e-4 / scale.factor()` scaled coordinate units (`100` for Normal and `10` for LargeBed);
   - classify fractional Voronoi vertices before integer conversion in the annotation path; use the source conversion at each call site (`std::round` for Voronoi `Point(double, double)` construction, truncation for endpoint-extension Eigen casts);
   - global `max_w` via source `fmaxf` semantics;
   - endpoint extension by `max_width`, including strict polygon boundary projection `< scaled_epsilon²`, contour/hole boundary rules, the two-point midpoint rule, and first contour-intersection ordering;
   - compute `ThickPolyline::length` by summing adjacent stored segments in order through source `Line::length`, then remove endpoint polylines shorter than `2 * max_w`;
   - perform the source greedy reconnect loop only when removal occurred.
8. Add `classic::medial_gap` with aligned `PreparedMedialGapObject/Record/Surface` and `PreparedPostClassicMedialGap` successors. Each surface moves its complete O11 surface and adds `medial: Option<MedialGapDomain>`, where `MedialGapDomain` owns the moved `PreMedialGapDomain` and its aggregated `Vec<ThickPolyline>`. O11 `None` maps to `None`; a non-empty O11 domain that produces no axis maps to `Some` with an empty vector. Process and append expolygons and their polylines strictly in stored order. Preserve all nested O10 collections without flattening/reallocation and keep the boxed O5 predecessor.
9. Pass `CoordinateScale` from the boxed traversal predecessor into the geometry API. Stage all medial-axis outputs before moving O11 ownership. A `boostvoronoi` builder error or an adapter topology-invariant failure returns `SliceError::InvalidInput("Classic medial-axis Voronoi construction failed")`, consumes the old state iteratively, and exposes no partial successor.
10. Empty O11 `pre_medial` remains typed absence and produces no medial-axis payload.
11. Keep the public project lifecycle intentionally at `ProjectSlicingIncomplete`; do not generate gap extrusion or G-code in this milestone.

## Explicitly deferred

- Orca's rare completed-diagram validity detector, rotation repair, and morphological-closing retry from `Voronoi.cpp`/`MedialAxis.cpp:464-472`. This slice reports only errors returned by `boostvoronoi` construction and explicit adapter topology-invariant failures; it does not claim to detect an Orca-invalid diagram that the dependency completes successfully. A future source-cited robustness slice must add detection and repair without a legacy fallback.
- `PerimeterGenerator.cpp:1604-1608`: tiny-gap filtering.
- `PerimeterGenerator.cpp:1611-1625`: `variable_width`, covered-width subtraction, and gap-fill append.
- Infill-boundary generation beginning at `PerimeterGenerator.cpp:1628`.
- Arachne, seams, remaining print planning, motion, and G-code emission.
- Curved-edge visualization/discretization debug-only code.

## Dependencies and platform disposition

- Add `boostvoronoi = { version = "=0.12.1", default-features = false }` to workspace dependencies and `boostvoronoi = { workspace = true }` to `crates/ares-core/Cargo.toml`.
- Add target-specific `getrandom = { version = "=0.3.4", features = ["wasm_js"] }` to `ares-core` for `wasm32`, unifying the already-transitive version and qualifying the `boostvoronoi` → `cpp_map` dependency chain for browser builds.
- The pinned BSL-1.0 crate and its non-optional dependency closure are pure Rust. Local inspection of the published 0.12.1 source confirms the required public equivalents: `Edge::{twin,cell,vertex0,is_primary}`, `Diagram::{edge_get_vertex1,edge_is_finite,edge_rot_next,edge_rot_next_iterator}`, and `Cell::{source_index,source_category,contains_point,contains_segment}`. Ares imports only builder/diagram/geometry APIs. `boostvoronoi` unconditionally compiles an unused filesystem-reader utility that cannot be feature-disabled; Ares never calls or re-exports it, and both browser-target checks are mandatory acceptance evidence.
- The adapter uses the crate's local `Diagram` only within synchronous per-surface staging. It does not cross thread/async boundaries, so the crate's non-`Sync` interior model does not change Ares's public API or concurrency contract.
- Update `THIRD_PARTY_NOTICES.md` with the linked `boostvoronoi` BSL-1.0 dependency and pinned version.

## Constraints

- No reading or replaying the reference G-code at runtime or in production tests.
- No fixture identity/name/hash/layer-count branches and no Orca runtime/FFI.
- No `unsafe`, `include!`, `include_bytes!`, source-text/hash/line pinning tests, or binary oracle payloads.
- Production and test Rust files stay below 400 LOC and tests use real `mod` files.
- Preserve Tier-1/WASM compatibility and exact integer/floating cast placement.
- Existing O1–O12 behavior and checksums remain unchanged.

## Acceptance criteria

1. Literal geometry tests cover stored-order adjacent-segment length summation, a rectangle, a narrow concave region, a hole, branch chaining, endpoint extension, short-polyline removal/reconnect, width direction, scaled epsilon at Normal/LargeBed, and empty input.
2. Adapter tests pin literal edge enumeration IDs, consecutive twin pairing, source indices/categories, vertex endpoints, primary/finite flags, and `rot_next` order for fixed inputs. Fixed Orca-derived literals cover finite-vertex categories and eligible-edge decisions at contour, hole, and epsilon boundaries.
3. Tests assert literal `ThickPolyline` points, widths, endpoint flags, and output ordering; expected values are not recomputed through the production algorithm.
4. Lifecycle tests prove a deterministic builder/topology error is transactional, success/error cleanup is iterative on a constrained stack, O11 allocation identity is preserved, O10 collection content/identity is preserved, and the boxed O5 predecessor is preserved.
5. The KSR 3MF reaches the new stage deterministically and pins a literal full-structure checksum including object/record/surface boundaries and all medial-axis values.
6. Focused O13 tests, O11–O5 regressions, full workspace Nextest, strict workspace Clippy, workspace check, `ares-core` and `ares-wasm` WASM checks, rustfmt, `git diff --check`, LOC audit, and forbidden-pattern audit pass.
7. A fresh independent six-dimension review—requirements, logic, edge cases, code quality, test coverage, and actual execution—returns approval after any required fix/re-review cycles.
