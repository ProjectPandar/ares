# Task 22O.15 — Classic Infill-Boundary Construction Spec

## Status

Implemented and locally validated after approved independent/OpenCode specification review. The literal KSR checkpoint is `136197013209006370081121271251125478104`; 49 focused O15 tests and geometry regressions, 5,540 workspace tests with 2 skipped, strict Clippy, workspace/native and both WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, and staging audits pass. The final independent six-dimensional implementation rereview and OpenCode rereview both returned `VERDICT: APPROVE`.

## Upstream source boundary

Pinned upstream: OrcaSlicer v2.4.2 commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

This milestone rewrites the next Classic perimeter slice in `OrcaSlicer/src/libslic3r/PerimeterGenerator.cpp:1628-1691`:

- the one-more-offset inset selection after gap extrusion;
- layer-sensitive `infill_wall_overlap` and `top_bottom_infill_wall_overlap` conversion;
- `ExPolygon::simplify_p` and aggregate `union_ex` preparation;
- narrow-infill collapsing through `offset2_ex`;
- `top_fills` growth, `fill_clip` intersection, and top-overlap union;
- ordered `stInternal` fill-surface append;
- the source no-op `apply_extra_perimeters` call reached by currently supported Classic configurations;
- both source branches that construct `fill_no_overlap`.

Directly reached definitions are:

- `libslic3r.h:52,92-94,124-125` and `PerimeterGenerator.hpp:135,161` for `EPSILON`, `scale_`, `unscale<T>`, and raw `m_scaled_resolution`;
- `Config.hpp:1165-1178` for `ConfigOptionPercent::get_abs_value`;
- `ExPolygon.cpp:223-248`, the complete Douglas–Peucker definition at `MultiPoint.cpp:164-229`, and the StrictlySimple NonZero flat-path repair reached by `simplify_polygons`;
- `ClipperUtils.hpp:19-27,344-348,391-393,509-520,548-553` and `ClipperUtils.cpp:560-588,788-824,1019-1031` for the exact default Miter offset/offset2, intersection, and union overloads;
- `Surface.hpp:9-55,245-269` and `SurfaceCollection.hpp:74-85` for internal-surface construction and ordered append;
- `PerimeterGenerator.cpp:1087-1114` for the included guard and deferred activated body of `apply_extra_perimeters`;
- the overlap option definitions in `PrintConfig.cpp:4148-4172`.

The Rust destination is a new crate-private `classic::infill_boundary` successor after `classic::gap_extrusion`, plus a sibling `simplify_p`-shaped helper under `ares-core::geometry`. The already reviewed Rust Clipper boolean/offset kernels and Douglas–Peucker implementation are reused unchanged. The current `geometry::append_simplified_expolygon` compatibility scaffold performs a per-expolygon `union_ex`; O15 must neither change it nor use it at this call site.

## Included behavior

1. Recover each aligned record and surface only through the preserved O14/O5 predecessor graph. Use the O3 onion surface's final `effective_loop_number`, the O2 top-split surface's `top_fills` and `fill_clip`, the O1 prelude record's `external_spacing`, `perimeter_spacing`, and `solid_infill_spacing`, the preserved typed resolved print config's raw `resolution`, and the original aligned perimeter input record's `layer_id`, upper-layer presence, and effective typed `RegionOptions`. Do not reparse raw option maps or duplicate fixture values.
2. Stage a whole-project numeric preflight before any O15 simplification or Clipper operation. For each surface, compute the source inset before overlap: `0` when `effective_loop_number < 0`, integer `external_spacing / 2` when it equals `0`, otherwise integer `perimeter_spacing / 2`.
3. Only when the pre-overlap inset is positive, compute the overlap basis as checked signed `coord_t` arithmetic `inset + solid_infill_spacing / 2`. On layer zero or when the aligned upper-slices pointer is absent, derive `infill_peri_overlap` from typed `top_bottom_infill_wall_overlap` and keep `top_infill_peri_overlap = 0`. Otherwise derive the ordinary overlap from typed `infill_wall_overlap` and the top overlap from typed `top_bottom_infill_wall_overlap`. Preserve the full source conversion sequence rather than algebraically cancelling it: `double(basis) * scale.factor()`, then `* percent`, then `/ 100`, then `/ scale.factor()`, then checked truncation to `coord_t`. Subtract the ordinary overlap from `inset` only after both conversions succeed.
4. The whole-project numeric preflight must make every reached source signed-integer operation defined before geometry: the overlap basis, `min_perimeter_infill_spacing`, post-overlap inset, any unary negation of inset, integer `min_perimeter_infill_spacing / 2 - infill_peri_overlap`, and selected `-inset - infill_peri_overlap`. Evaluate with checked or wider arithmetic while preserving the source result, and reject any non-representable intermediate with `SliceError::InvalidInput("Classic infill-boundary overlap is outside the supported coordinate range")` before any simplification or Clipper call. Compute `min_perimeter_infill_spacing` as the checked truncating source expression `coord_t(solid_infill_spacing * (1. - INSET_OVERLAP_TOLERANCE))`, with `INSET_OVERLAP_TOLERANCE = 0.4`. Preserve signed integer division versus floating half at every call site.
5. Compute source `m_scaled_resolution` once from typed resolved print config as `max(resolution, EPSILON) / scale.factor()`, without the O1 arc-fitting multiplier. Add an exact `ExPolygon::simplify_p`-shaped helper that copies contour then holes, closes each path, runs the existing Douglas–Peucker implementation at this raw `m_scaled_resolution`, removes the duplicate endpoint, and performs the reached StrictlySimple NonZero `simplify_polygons` repair to flat ordered paths. It returns ordered `Polygons`; it must not perform the unreached per-expolygon `union_ex`/PolyTree grouping pass from `ExPolygon.cpp:250-253`. Append every O14 `remaining` expolygon's polygons in order, then run one aggregate NonZero `union_ex` to form `not_filled_exp`. Never reuse the arc-adjusted O1 `surface_simplify_resolution` (`0.2 * m_scaled_resolution` when arc fitting is enabled).
6. Construct ordinary infill with the exact float-cast deltas `float(-inset - min_perimeter_infill_spacing / 2.)` and `float(min_perimeter_infill_spacing / 2.)`, default Miter joins, and miter limit `3.0` through `offset2_ex`.
7. Always compute `top_infill_exp` in source order as the intersection of aligned `fill_clip` with `top_fills` offset by `float(double(external_spacing / 2))`. When and only when the original `top_fills` vector is nonempty, offset `top_infill_exp` by `float(double(top_infill_peri_overlap))` and union it with ordinary infill. Preserve both `i64 -> f64 -> f32` narrowing chains rather than casting directly from `i64` to `f32`.
8. Append every resulting infill expolygon in order as a `RegionSurface::internal`, preserving the source defaults: kind `Internal`, thickness `-1`, thickness layers `1`, bridge angle `-1`, and extra perimeters `0`.
9. Preserve the reached `apply_extra_perimeters(infill_exp)` guard as a no-op for the supported configuration envelope. Its source activation requires all of: `!spiral_mode`, aligned lower slices present, typed `detect_overhang_wall`, typed `extra_perimeters_on_overhangs`, positive typed `wall_loops`, and `layer_id > raft_layers`. The existing Classic preflight independently rejects every spiral record and rejects the remaining activating conjunction. Inactive `extra_perimeters_on_overhangs = true` records caused by absent lower slices, disabled overhang detection, nonpositive wall loops, or `layer_id <= raft_layers` remain accepted only when all other Classic preflight rules pass. The activated helper body is explicitly deferred rather than hardcoded or silently emulated.
10. Construct `fill_no_overlap` after the logical internal-surface append and inactive extra-perimeter guard. If integer `min_perimeter_infill_spacing / 2 > infill_peri_overlap`, use `offset2_ex` with exact deltas `float(-inset - min_perimeter_infill_spacing / 2.)` and `float(min_perimeter_infill_spacing / 2 - infill_peri_overlap)`. Otherwise use `offset_ex` with `float(double(-inset - infill_peri_overlap))`. When `top_fills` is nonempty, union this result with unexpanded `top_infill_exp`. Append outputs in source surface order.
11. Represent the result per aligned record: moved O14 surfaces remain ordered and retain O13/O11/O10/O5 ownership; the record additionally owns ordered internal fill surfaces and ordered `fill_no_overlap` expolygons. Preserve `None` record slots, source indices, all existing gap-fill state, and exact surviving allocations.
12. Stage every O15 overlap value, simplified polygon set, offset/intersection/union result, internal surface, and no-overlap result for the whole project before moving O14 ownership. Map all simplification/Clipper failures to `SliceError::InvalidInput("Classic infill-boundary geometry is outside the supported Clipper range")`. On any error expose no successor and iteratively consume untouched O14 state.
13. Wire the actual public `slice_project` lifecycle through O15 exactly once, then intentionally continue returning `ProjectSlicingIncomplete`.

## Explicitly deferred

- The activated `apply_extra_perimeters` body in `PerimeterGenerator.cpp:1087-1114` and its helper chain in `PerimeterGenerator.cpp:885-1086`, including overhang detection, bridge direction, path sorting, perimeter-collection mutation, and fill-surface subtraction. Its activation remains rejected by the existing source-cited Classic preflight; only the exact reached guard and inactive call are included here.
- The unreached `ExPolygon::simplify` wrapper at `ExPolygon.cpp:250-253`; O15 reaches `simplify_p` and performs only one later aggregate `union_ex`.
- `PerimeterGenerator.cpp:1695` onward, including `add_infill_contour_for_arachne`, counterbore/no-bridge processing, surface classification, and downstream perimeter-engine work.
- Arachne's sibling infill-boundary call site; the KSR project selects Classic and the current Classic preflight rejects Arachne.
- Downstream fill generation, sparse/solid/top/bottom classification, `gap_fill_flow_ratio`, seams, ordering, motion, G-code emission, and post-processing.
- Any legacy STL/rectangle pipeline adapter, public f64 extrusion scaffold, Orca runtime/FFI, fixture branch, or reference-G-code replay.

## State and ownership

- Add `PreparedPostClassicInfillBoundary`, aligned object/record/surface types, and record-owned `Vec<RegionSurface>` plus `Vec<ExPolygon>` outputs.
- The surface moves all O14 fields without copying surviving point, width, entity, collection, or polygon buffers.
- The top-level successor retains the exact boxed O5 predecessor already owned by O14.
- Success and every stable error path use iterative sinks and must fit a 64 KiB test stack with both predecessor tree families deepened to 10,000 nodes.

## Tests and acceptance criteria

1. Literal unit tests distinguish all inset branches, layer-zero/last-layer versus middle-layer overlap option selection, mandatory negative representable percentages, ordinary- and top-overlap conversion overflow/non-finite intermediates, Normal/LargeBed cast boundaries, floating versus integer half deltas, and both no-overlap branches. Source-derived maximal/minimal boundary tests prove the basis, 0.6 minimum-spacing conversion, unary negation, and `-inset - overlap` else expression remain representable; do not invent unreachable scalar states. Separate real failure tests cover post-subtraction overflow and `min_perimeter_infill_spacing / 2 - infill_peri_overlap` overflow with zero O15 geometry invocations.
2. Literal simplification tests prove contour-before-holes order, the required per-expolygon StrictlySimple flat-path repair without premature PolyTree grouping, aggregate union order, narrow-region collapse, top-fill intersection/expansion, internal-surface defaults, and stable multi-surface append order. An arc-fitting-enabled case must use geometry that distinguishes raw `m_scaled_resolution` from O1's one-fifth adjusted tolerance, proving O15 uses the raw value while O1 remains unchanged. A source-order observation test must prove: each `simplify_polygons`; one aggregate `union_ex`; ordinary `offset2_ex`; top offset; intersection even for empty original `top_fills`; conditional top-overlap offset/union based on the original vector; logical surface append; inactive extra-perimeter guard; selected no-overlap offset; and conditional union with unexpanded `top_infill_exp`. An empty-top-fill failure probe must catch an implementation that skips its mandatory offset/intersection calls.
3. Typed 3MF mutation tests independently change `infill_wall_overlap` and `top_bottom_infill_wall_overlap`; output changes must come from aligned effective records. Every numeric-preflight failure must beat simplification, offset, intersection, and union candidates positioned before and after it, with zero O15 geometry invocation.
4. Direct aligned-stage tests cover `effective_loop_number < 0`, `== 0`, and `> 0`, absent upper slices, empty and nonempty top fills, empty remaining input, and every exact stable error.
5. Ownership tests preserve O14/O13/O11/O10 and boxed O5 allocations. Success plus overlap error and each independently injected geometry failure clean up iteratively on a 64 KiB stack.
6. KSR reaches O15 deterministically and pins a literal full-structure checksum containing object/record/surface delimiters, typed overlap-derived values, every moved O14 field, internal-surface defaults and polygons, fill-no-overlap polygons, and predecessor checks.
7. The public lifecycle invokes O15 once and remains incomplete. Focused O15 and O14-O5 regressions, full workspace Nextest, strict all-target Clippy, workspace check, both WASM checks, rustfmt, `git diff --check`, LOC, forbidden-pattern, dependency, and staging audits pass. Final evidence records the existing Tier-1 Windows, macOS, and Linux CI matrix in addition to local browser-WASM checks.
8. Every Rust source and test file remains below 400 LOC; no `unsafe`, `include!`, `include_bytes!`, source-text/hash/line pinning tests, binary oracle payload, runtime Orca dependency, or fixture-identity branch is introduced.
9. Independent spec and plan reviews approve before implementation. After implementation, an independent six-dimension reviewer returns either a concrete fix list or `VERDICT: APPROVE`; main-thread fixes are re-reviewed until approval. A separate OpenCode review must also approve.

## Documentation and migration

Update `docs/architecture/option-parity-v4.md` and `docs/roadmap.md` with the exact O15 source/destination seam, typed overlap provenance, numeric/cast rules, inactive extra-perimeter boundary, transactional ownership, verification evidence, and next source boundary.

O15 introduces no public API, persisted-format, dependency, or compatibility migration. Rollback restores the O14 lifecycle terminal and removes only O15 state/wiring plus the exact simplification helper, while preserving all reviewed O1-O14 behavior.
