# Task 22O.14 — Classic Variable-Width Gap Extrusion Spec

## Status

Implemented, validated, independently reviewed, and approved. The post-fix workspace run passed 5,491 tests with 2 skipped; strict Clippy, native/workspace checks, both WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, and staging audits passed. Independent Codex and OpenCode re-reviews both returned `VERDICT: APPROVE`.

## Upstream source boundary

Pinned upstream: OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the next complete Classic perimeter behavior reached from `PerimeterGenerator.cpp:1604-1624`:

- strict tiny-gap filtering at `PerimeterGenerator.cpp:1604-1608`;
- `VariableWidth.hpp` and `VariableWidth.cpp:99-234`, specifically `thick_polyline_to_extrusion_paths_2` and `variable_width`;
- the reached `Flow::with_width`, rounded-rectangle spacing, and `mm3_per_mm` behavior in `Flow.hpp` and `Flow.cpp`;
- the reached `ExtrusionPath`, `ExtrusionLoop`, and `ExtrusionEntityCollection::polygons_covered_by_width` behavior in `ExtrusionEntity.hpp`, `ExtrusionEntity.cpp:68-71,347-351`, and `ExtrusionEntityCollection.cpp:99-103`;
- the open-polyline offset wrapper and its two cleanup levels in `ClipperUtils.hpp:21-31,336-338` and `ClipperUtils.cpp:267-293,333-357,412-428`;
- its directly reached Clipper 6 `ClipperOffset::AddPath`, orientation, execution, open-butt generation, square joins, and positive-fill cleanup in `deps_src/clipper/clipper.cpp:3371-3810`;
- the already ported and reviewed length/pairing prerequisites from `MultiPoint.cpp:48-56`, `Polyline.hpp:256-277`, and `Polyline.cpp:637-646`, reused without changing their O12/O13 semantics.

The Rust destination is a general crate-private open-butt path offset extension under `ares-core::geometry::clipper`, a crate-private Classic variable-width conversion module, and a new aligned `classic::gap_extrusion` successor after `classic::medial_gap`.

## Included behavior

1. Resolve `filter_out_gap_fill` from each aligned effective typed `RegionOptions` record in the 3MF-derived resolved configuration. Run a whole-project option-validation prepass before any O14 variable-width or Clipper geometry. A non-finite or negative value returns `SliceError::InvalidInput("invalid Orca option filter_out_gap_fill")`; this error has precedence over every O14 geometry/flow error. Compute the unrounded `f64` threshold as `filter_mm / scale.factor()`; do not use `checked_scale`, an integer conversion, or truncation. Retain a medial `ThickPolyline` exactly when `polyline.length() >= scaled_threshold`; equality survives and stored order is unchanged.
2. Preserve `ThickPolyline::thicklines()` order and two-width-per-segment pairing. Convert each retained polyline with source `thick_polyline_to_extrusion_paths_2` semantics and a tolerance equal to `float(scale_(0.05))` for the active scale.
3. Preserve the mutable line-list loop exactly:
   - initialize the running minimum and maximum from the first line's `a_width`;
   - when `line.length() < SCALED_EPSILON`, skip only the current extrema/break processing; keep that line in the mutable list so later flushes preserve its stored point order and include its actual length/width contributions. `SCALED_EPSILON` is the unrounded `f64` expression `1e-4 / scale.factor()`, yielding `100.0` for Normal and `10.0` for LargeBed; exact equality is processed rather than skipped;
   - compare the current line's `b_width` to both running extrema using strict `>` tolerance;
   - flush `[start_index, i)` with the source length-weighted midpoint-width average, but emit it only when total length is strictly `> SCALED_EPSILON`;
   - reset both extrema to the breaking line's `a_width` before its individual split decision;
   - split an individual over-tolerance line into `ceil(abs(a_width-b_width)/tolerance)` segments, using source double normalized-vector arithmetic and truncating Eigen-style coordinate casts, insert the segments in place, decrement the logical index, and reprocess the inserted first segment;
   - update extrema only in the source non-break branch;
   - flush the final range with the source asymmetric `sum(length * a_width) / sum(length)` formula, again emitting only when total length is strictly `> SCALED_EPSILON`.
4. Build every variable-width path with fixed-coordinate `Point3` values at `z = 0`, role `GapFill`, and exact flow metadata. The `Flow::with_width` input is `unscale<float>(scaled_width) + flow.height() * float(1 - PI/4)`, preserving the source `f32` and `f64` cast/order sites. Use the aligned record's `solid_infill_flow`; `gap_fill_flow_ratio` is not applied at this source boundary and remains deferred to G-code emission.
5. Preserve entity formation and order. If the first generated path's first XY point equals the last generated path's last XY point, emit one gap-fill loop containing all paths in order. Otherwise emit each generated path as a separate entity in path order. Keep gap fill as a separate ordered collection; do not flatten it into O10 perimeter collections.
6. Extend the existing source-cited ClipperOffset rewrite with open paths and the reached `OpenButt` end type. For each input path, clear the offset engine, preserve consecutive duplicate/short-edge removal with the source strict `distance² < shortest_edge_length²` rule (equality survives), generate square joins, walk forward interior corners, append the exact two-point end cap, reverse normals and walk reverse interior corners, and append the exact two-point start cap. Execute that one path through ClipperOffset's Positive-fill union cleanup, append its ordered polygons to the raw aggregate, then run the wrapper-level NonZero union over the aggregate. Preserve fixed rounding, shortest-edge factor `0.005`, and input/output ordering. Existing closed offset behavior must remain unchanged.
7. Compute covered-width polygons for each generated path in entity/path order using the source expression `float(scale_(path.width / 2)) + 10.f`, default square joins, default open-butt ends, and line miter limit `0`. Loops delegate to their paths in order; the gap collection delegates to entities in order. Do not pre-union across entities beyond each open-offset wrapper's source union.
8. Clone the aligned onion-stage `last` expolygons as the O14 fill remainder. If the filtered polyline list is non-empty, subtract the ordered covered-width polygons with ordinary `difference_ex`; otherwise retain `last` unchanged. Emit the resulting `remaining` expolygons and the separate gap-fill entity collection on each aligned surface.
9. Stage option validation, keep masks, converted entities, coverage, and all differences for the whole project before moving O13 ownership. On success, filter the owned O13 medial polyline vector without reordering, move the complete O13/O11/O10 surface state, preserve nested surviving allocations and the boxed O5 predecessor, then attach gap entities and remaining geometry.
10. After the option-validation prepass, map open-offset or difference failures to `SliceError::InvalidInput("Classic gap-extrusion geometry is outside the supported Clipper range")`. Map invalid derived flow to `SliceError::InvalidInput("Classic variable-width gap flow is invalid")`. Tests must independently trigger invalid flow, open-offset range failure, and difference range failure without fixture branching or test-only production behavior. On any error, expose no partial successor and iteratively consume the untouched O13 state.
11. Preserve `None` versus `Some(empty)`: O13 `medial: None` remains `None`; a present medial domain may become empty after filtering. Empty filtered output creates an empty gap collection and leaves the aligned `last` clone unchanged.
12. Wire the actual public `slice_project` lifecycle through O14 exactly once, then intentionally continue returning `ProjectSlicingIncomplete`. This milestone does not claim complete Classic perimeters or final G-code parity.

## Explicitly deferred

- `PerimeterGenerator.cpp:1628` onward: infill-boundary inset, simplification, collapsing, surface classification, no-overlap surfaces, and extra perimeters.
- The rare medial-axis invalid-diagram detector/repair already deferred by O13.
- The older `thick_polyline_to_multi_path` path and its overhang-bridge special case; this milestone ports only the production `_2` function called by `variable_width`.
- Thin-wall `variable_width` call sites outside the reached gap-fill branch.
- `gap_fill_flow_ratio` application in `GCode.cpp`, speed planning, infill generation, seams, brim/skirt/support ordering, motion planning, and G-code emission.
- Any public compatibility adapter through the legacy Ares `gap_fills`, public `extrusion_entity`, STL/rectangle pipeline, or Orca runtime.

## State and destination types

- Add `GapFill` to the crate-private Classic materialized extrusion role; do not route through the unrelated public f64 extrusion model.
- Add a crate-private heterogeneous gap entity enum with `Path(ExtrusionPath)` and `Loop(Vec<ExtrusionPath>)`, plus an ordered gap collection.
- Add aligned `PreparedGapExtrusionObject`, `PreparedGapExtrusionRecord`, `PreparedGapExtrusionSurface`, and `PreparedPostClassicGapExtrusion` types. Each surface owns the moved O13 fields, its filtered medial domain, the separate gap collection, and the cloned/subtracted `remaining` expolygons.
- Recover typed region options and input flow through the exact aligned traversal/prelude record path already used by O10; no global `SliceOptions` parsing or fixture value substitution is allowed.

## Constraints

- No reading or replaying reference G-code at runtime or in production tests.
- No fixture identity/name/hash/layer-count branches, Orca runtime/FFI, or hardcoded KSR output.
- No `unsafe`, `include!`, `include_bytes!`, source-text/hash/line pinning tests, or binary oracle payloads.
- Production and test Rust files stay below 400 LOC and tests use real `mod` files.
- Preserve Tier-1/WASM portability, exact scaled integer/floating cast placement, Clipper ordering, and transactional cleanup.
- Existing O1–O13 behavior remains unchanged except that O14's owned view contains the source-required filtered medial list and fill remainder.
- Do not change dependencies for this milestone.

## Documentation impact

- Update `docs/architecture/option-parity-v4.md` with the exact O14 source/destination seam, typed option provenance, two-level open-offset cleanup, numeric rules, transactional ownership, and explicitly deferred behavior.
- Update `docs/roadmap.md` with the O14 exit criteria, verification evidence, and the next source boundary at `PerimeterGenerator.cpp:1628`.
- Keep the O14 spec/plan status synchronized with implementation and review state.

## Rollback

O14 introduces no public API, persisted-format, dependency, or compatibility migration. A deployment rollback restores `slice_project` and `perimeters` lifecycle consumption to the approved O13 successor and removes only O14 stage wiring/types plus the O14-only open-path offset API. The pre-existing closed-offset implementation and O13/O11/O10/O5 ownership graph must remain unchanged and continue passing their reviewed regressions.

## Acceptance criteria

1. Literal open-offset tests cover one-point source behavior, consecutive duplicates, below/equal/above shortest-edge threshold, a straight segment with exact butt caps, square-join left/right bends, reversed input, per-path Positive cleanup followed by aggregate NonZero union with a case distinguishing the two levels, Normal/LargeBed deltas, and unchanged closed-offset regression outputs.
2. Literal variable-width tests cover empty input, zero-length and nonzero below-epsilon lines, exact-epsilon scan processing, strict total-length equality rejection versus above-epsilon emission, below/equal/above `0.05 mm` tolerance, an in-line split, a grouped-range flush, the asymmetric final flush, multiple polylines, reversed widths, open entities, a closed loop, exact fixed points, roles, widths/heights, and `mm3_per_mm.to_bits()` values. Expected values are literals, not production-helper recomputations.
3. Direct stage tests prove typed per-record option provenance with different thresholds; invalid negative, NaN, and infinite thresholds; whole-project option-error precedence over geometry; strict `<` filtering; zero/equal/above and fractional fixed-unit thresholds at Normal/LargeBed; stable order; `None` versus `Some(empty)`; exact gap entity order/metadata; exact covered polygons; and exact remaining expolygons.
4. Ownership tests preserve the boxed O5 address, O11 expolygon allocations, O10 collection/entity/path allocations, and all surviving O13 polyline point/width allocations. Transactional tests separately trigger invalid option, invalid derived flow, open-offset range, and difference range failures, prove no successor is observable and exact stable errors, and prove iterative cleanup on a 64 KiB stack.
5. KSR reaches O14 deterministically and pins a literal full-structure checksum with object/record/surface and option/filter delimiters, filtered medial values, entity variants, path point/role/width/height/mm3 bits, coverage-result/remaining polygons, and explicit O13/O11/O10 predecessor checks.
6. The actual public lifecycle invokes O14 once and remains incomplete. Focused O14 and O13–O5 regressions, full workspace Nextest, strict Clippy, workspace check, both WASM checks, rustfmt, `git diff --check`, LOC, forbidden-pattern, and no-staged-file audits pass.
7. A fresh independent six-dimension review covers requirements completeness, logic correctness, edge cases, code quality, test coverage, and actual execution. The independent reviewer and OpenCode reviewer must each return `VERDICT: APPROVE`; findings are fixed by the main writer and all reviewers rerun until approval.
