# Pure-Rust Clipper2 boundary strategy

Status: **approved before implementation**. Target: OrcaSlicer
v2.5.0-beta, `345efec64c7b95a05ce5cf96df0dde66a158835c`.
This is a source-rewrite strategy, not a parity or platform PASS claim.

## Boundary and non-negotiable decisions

Replace the Clipper 6.4.2 engine behind `ares-core/src/geometry/clipper.rs`
with an in-tree, safe Rust port of the pinned `deps_src/clipper2/Clipper2Lib`:
`clipper.core.h`, `clipper.engine.h/.cpp`, and `clipper.offset.h/.cpp`.
The dependency identifies itself as **2.0.1** (`clipper.version.h:4`); its pinned
Git tree is `09f1baceae892df95ea07250dbb6b6d3dedd3d2b`.
`src/libslic3r/ClipperUtils.cpp/.hpp` and `ClipperZUtils.cpp:48–80` own the
application wrappers; dependency-version equivalence alone is insufficient.

Keep the existing geometry entry signatures only as temporary adapter shells
around these upstream operations. Replace their engine, offset, tree and Z
implementations together; do not retain a reachable Clipper1 backend, dual-engine
selector, native C++ dependency, or version/platform fallback. Keep implementation
inside `ares-core`, split by upstream responsibility before files exceed 400 lines.
Use indexed arenas for the upstream linked active/output records, retaining their
mutation, scanbeam, intersection and output traversal order. No new sort keys,
ring-start rotation or result canonicalization.

The pure-Rust candidate `larsbrubaker/clipper2-rust`, inspected at
`2f5e9d4249bc4d624773094ff6164335d9ff6ae7`, declares upstream **1.5.4** in
`src/version.rs`; it is not selected as an unchanged dependency. Reference code
reuse requires function-by-function reconciliation to the pinned 2.0.1 sources
and retained BSL-1.0 notices. The similarly named `clipper2`/`clipper2c-sys` FFI
route is excluded. The pinned Orca source, not another port's tests, is authority.

## Numeric contract

Copy scaled XY directly as `i64`; no rescaling through floating-point paths.
Port the declared `±(i64::MAX >> 2)` coordinate domain and upstream boundary error
behavior. Use Rust `i128` products for `ProductsAreEqual`/`CrossProductSign`
(`clipper.core.h:730–801`), preserving exact signs on browser WASM and native
platforms without compiler-specific intrinsics. Do not replace the engine's
floating arithmetic with a purportedly more exact algorithm: areas, slopes,
intersection calculations and offset normals retain upstream `f64` evaluation
order. Preserve each conversion separately: `Point64` construction uses
`std::round` semantics, while default `GetLineIntersectPt` interior results cast
with truncation (`clipper.core.h:117–165,956–979`). No global rounding policy,
FMA substitution, epsilon tuning or saturating/wrapping arithmetic workaround.

In particular, `clipper.offset.cpp:262–274::DoMiter` evaluates the full coordinate
expression before the `Point64` conversion (`clipper.core.h:127–134,176–183`).
`clipper2/offset/do_square.rs::do_miter` and `core/from.rs::Point64::from` use
`f64::round`, matching symmetric half-away-from-zero conversion: positive and
negative half ties do not use Clipper1's asymmetric `floor(x + 0.5)`.
The imported public witness from `14daa3d76` in
`classic_fully_overhanging_exterior_nearest.rs` additionally exercises classic
Nearest seams with a half-unit offset width. Its seam-extraction rewrite remains
owned by the separate `SeamPlacer.cpp::extract_perimeter_polygons` slice; adding
it to directed verification neither changes its assertion nor claims that every
failure is an offset defect.

## Wrapper policies to port literally

- **Stray holes:** `ClipperUtils.cpp:283–390` classifies by signed area, not tree
  depth alone. Recurse through misplaced contours/islands; collect clockwise
  contours encountered in an outer position. Reassign each stray to the smallest
  enclosing contour with strictly greater area, using the upstream bbox candidate
  pruning and ascending-area search. `path_encloses` ignores vertices within
  **3 scaled units** of the boundary, then uses the first clear vertex; if all
  touch, use the exact `interior_point` band/crossing algorithm
  (`:930–971`). An unresolvable sliver is not enclosed; an unattached hole remains
  the original clockwise contour. Do not reverse or discard it. The tolerance is
  source behavior, not permission to relax a comparator.
- **Collinear points:** ordinary closed/open wrappers explicitly set
  `PreserveCollinear(false)` (`:393–416`), despite Clipper2's different default.
  Z clipping uses its caller's flag. Preserve the engine's unconditional duplicate
  and 180-degree-spike removal and self-intersection repair
  (`clipper.engine.cpp:1551–1586`); retaining collinear points does not retain spikes.
- **Offset setup:** `ClipperUtils.cpp:419–430` uses `float` delta at the wrapper
  boundary. For round joins, positive `miterLimit` is arc tolerance clamped to
  `abs(delta)*0.25`; nonpositive values select **0.25**, without that clamp.
  Otherwise miter limit is at least **2**, and arc tolerance is
  `min(capTolerance > 0 ? capTolerance : 0.25, abs(delta)*0.25)`.
  Port the pinned offset engine's arc subdivision, rounding, orientation and final
  union (`clipper.offset.cpp:262–307,480–510,599–680`), not Clipper1's formulas.
- **Shortest edges and winding:** use `abs(delta*0.005)`
  (`ClipperUtils.hpp:47`, `.cpp:235–264,479–572`). Drop points only at squared
  distance **strictly less** than the threshold from the last retained point;
  zero threshold removes exact duplicates. Trim the closed trailing seam first;
  reject polygon paths with fewer than three retained points. Preserve the strict
  bbox group test, isolated-CW delta reversal, degenerate negative-offset handling,
  fallback union fill rule, and ExPolygon contour/hole winding preparation
  (`:433–525,582–598`). These are beta wrapper behavior, not an old-engine fallback.
- **Z paths:** `ClipperZUtils.cpp:48–80` executes NonZero clipping, distinguishes
  open subjects, and appends closed outputs before open outputs. Port engine
  endpoint-Z precedence (`clipper.engine.cpp:831–868`); the wrapper suppresses the
  user fill callback whenever the intersection XY coincides with any input endpoint.
  Preserve subject-before-clip endpoint priority and callback argument order.
  Otherwise pass the four original endpoints and copy back only the callback's Z.

## Integration seams and structural rewrite rules

The removed `StrictlySimple` policy is not emulated. The four tree-support calls
in `model_volumes.rs`, `organic/area_growth.rs` (two), and
`organic/drawing/finalize.rs` use `polygons_simplify` without the old trailing
`false`; these are signature adapters only. The owning functions remain
`ClipperUtils.cpp::simplify_polygons` and `Polygon.cpp::polygons_simplify`.

`Fill/FillBase.cpp:2947–3004::multiline_fill` calls the same offset wrapper with
miter joins, the default miter limit, round caps and `0.002 * delta` cap tolerance;
its scaled delta is `float`. Replace the former standalone round-offset and
`Clipper2Truncate` emulation at that call, without changing the offset list or
spacing algorithm. Remove obsolete Clipper1 open-path error arms at callers;
retain their `CoordinateOutOfRange` mappings unchanged.

All Rust nesting/size limits apply to the vendored port, without lint exemptions.
Extract helpers, early returns or exhaustive matches while preserving upstream
comparison, mutation and traversal order. Record each extraction against its
owning upstream function segment; do not add fallback branches to make a split
convenient. The Clipper1-only private tests and their now-orphaned helpers,
registrations and sort-trace observer are removed where retirement makes them
uncompilable; their legacy engine/state/error contracts are not reconstructed for
tests. The approved public geometry witnesses and runtime comparators remain
unchanged. Strict all-target Clippy is required before implementation handoff,
independently of the separately scheduled source-private-test deletion batch.
Current helper correspondence:

| Rust helper boundary | Pinned source segment |
| --- | --- |
| `geometry/chain_points.rs` and `geometry/chain_points/exact.rs::chain_segments_exact` | `ShortestPath.cpp:103–137,164–176,1565–1575`; shared two-through-four-point subset DP, preserving both orientation states, source Euclidean costs, ascending state/candidate scans and strict `<` ties; larger sets retain the existing greedy implementation. See `chain-points-small-set-exact.md` |
| `clipper/ordering/gcc.rs::move_median_to_front` | Retain the already audited GNU `std::sort` median helper used at the pinned engine's sort sites; group only its three candidate indices in a tuple, removing the existing argument-count exemption without changing comparisons or swaps |
| `polytree/stray_holes/bbox_index.rs::{partition,median,split}` | `AABBTreeIndirect.hpp:162–218::Tree::partition_input`; median-of-three, pivot partition, restoration and narrowing retain their original order |
| `polytree/stray_holes/bbox_index.rs::Index::{new,build,visit}` | `AABBTreeIndirect.hpp:129–155,228–242` and `ClipperUtils.cpp::c2_attach_stray_holes`; preserve inflated wrapper bounds, integer centroid truncation, axis ties and left-before-right traversal |
| `clipper2/engine/winding_counts.rs::closed_wind_count` and the two winding-count methods | `clipper.engine.cpp:1037–1148`; preserve the predecessor search, three initialization cases and outside-all/direction tests; do not silently break an invalid AEL chain |
| `clipper2/engine/winding_counts.rs::{toggle_closed_count2,accumulate_closed_count2}` | `clipper.engine.cpp:1096–1112`; the two original closed-edge walks remain separate, with identical counter-write and next-link-read order |
| `clipper2/engine/winding_counts.rs::{count_open_crossings,accumulate_open_crossings}` | `clipper.engine.cpp:1117–1147`; preserve the local-counter EvenOdd walk and the in-place non-EvenOdd accumulation |
| `clipper2/engine/is_contributing_open.rs::insert_left_edge` | `clipper.engine.cpp:1183–1215::InsertLeftEdge`; finish empty/head insertion before the resident walk, retaining paired-right-edge advancement and link-write order |
| `clipper2/engine/add_local_max_poly.rs::close_local_max_outrec` | `clipper.engine.cpp:1433–1445::AddLocalMaxPoly`; same-record owner assignment, uncoupling, saved output point and real-owner lookup retain their order |
| `clipper2/engine/is_contributing_open.rs::set_closed_minimum_sides` | `clipper.engine.cpp:1370–1398::AddLocalMinPoly`; preserve no-hot-predecessor and hot-predecessor side/owner branches |
| `clipper2/engine_public/is_hole.rs::append_built_path` | `clipper.engine.cpp:3020–3079::BuildPaths64/BuildTree64`; factor only BuildPath64 admission/push, leaving closed cleanup, output starts and live record traversal at their original call sites |
| `clipper2/engine/input_paths.rs::{add_input_path,initial_input_direction,mark_input_extrema}` | `clipper.engine.cpp:647–742::AddPaths_`; preserve duplicate suppression, closing count, first direction, linked extrema walk and open/closed final flags |
| `clipper2/engine/find_and_add_local_minima_closed.rs::dispose_out_pts` | `clipper.engine.cpp:334–345::DisposeOutPts`; break the circle, walk the null-sentinel chain and clear the record; arena slots are retained until engine cleanup |
| `clipper2/engine/insert_local_minima_into_ael.rs::{insert_local_minimum,minimum_bounds_need_swap,insert_minimum_right_bound}` | `clipper.engine.cpp:1224–1340::InsertLocalMinimaIntoAEL`; one minimum at a time, descending-before-ascending allocation, original horizontal/slope swap tests, contribution, right-bound crossings and scanline admission |
| `clipper2/engine/insert_local_minima_into_ael.rs::reset_horz_direction` | `clipper.engine.cpp:2537–2561::ResetHorzDirection`; early nonvertical return leaves the vertical maxima search unchanged |
| `clipper2/engine/do_intersections.rs::{make_intersection_adjacent,advance_scanbeam_edge,finish_open_maximum}` | `clipper.engine.cpp:2474–2499,2734–2820`; preserve adjacency search/swap, live next-edge traversal, open-end cleanup and the distinct open/closed maxima deletion orders |
| `clipper2/engine/horz_segments.rs::{update_horz_segment,set_horz_heading}` | `clipper.engine.cpp:2193–2241`; retain open/closed endpoint walks, heading assignment, duplicate-left marker admission and invalid-segment clearing |
| `clipper2/engine/convert_horz_segs_to_joins.rs::join_horizontal_pair` | `clipper.engine.cpp:2244–2291::ConvertHorzSegsToJoins`; stable valid-segment sorting, opposite-direction overlap tests, live left-point advancement and duplicate insertion order |
| `clipper2/engine/horz_joins.rs::{process_horz_join,assign_horz_split_owner,assign_horz_join_owner}` | `clipper.engine.cpp:2305–2370::ProcessHorzJoins`; splice before split/join ownership, preserve point swaps/fixups, containment precedence and split-list writes |
| `clipper2/engine/convert_horz_segs_to_joins.rs::{collinear_step,CollinearStep}` | `clipper.engine.cpp:1551–1586::CleanCollinear`; distinguish advance, restart after removal and disposed-record return without changing duplicate/spike predicates or cleanup order |
| `clipper2/engine/fix_self_intersects.rs::{self_intersection_step,SelfIntersectionStep,register_split_owner}` | `clipper.engine.cpp:1646–1706::DoSplitOp/FixSelfIntersects`; preserve local intersection checks, callback/area order, disposal and restart points, and split ownership registration |
| `clipper2/engine/intersect_open_edges.rs::{intersect_open_edges,try_join_open_minimum}` | `clipper.engine.cpp:1811–1903::IntersectEdges`; split the closed edge before admission, preserve contribution toggles and Z order; use only the pinned local-minimum vertex predicate, removing the seed-only additional `point == edge.bot` restriction |
| `clipper2/engine/intersect_edges.rs::{update_intersection_winding,update_same_type_winding,normalized_winding}` | `clipper.engine.cpp:1911–1963,2044–2063::IntersectEdges`; retain counter-write order and the two source fill-rule projections, with int-sized absolute/negation before second-count widening |
| `clipper2/engine/intersect_edges.rs::{intersect_hot_edges,intersect_single_hot_edge,intersect_cold_edges,set_intersection_z}` | `clipper.engine.cpp:1975–2123::IntersectEdges`; preserve max/min/output creation, callback argument and invocation order, and final record swaps |
| `clipper2/engine/do_horizontal.rs::{sweep_horizontal,horizontal_reaches_end,cross_horizontal_neighbor,finish_horizontal_maximum,finish_open_horizontal}` | `clipper.engine.cpp:2563–2732::DoHorizontal`; preserve endpoint inequalities, maxima priority, per-crossing mutation order, saved next neighbor and distinct edge deletion order |
| `clipper2/engine/intersections.rs::{merge_sel_pass,merge_sel_runs,add_sel_crossings,update_sel_run_base}` | `clipper.engine.cpp:2416–2472::BuildIntersectList`; retain stable adjacent-run merging, live SEL links, backward crossing registration and base/jump updates; invalid internal chains are not silently skipped |
| `clipper2/engine/intersections.rs::clamp_intersection_to_scanbeam` | `clipper.engine.cpp:2382–2413::AddNewIntersectNode`; preserve the parallel fallback, steep-edge priority, scanbeam bounds, closest-point casts and TopX selection |
| `clipper2/offset/floating_point_tolerance.rs::update_lowest_path_area` | `clipper.offset.cpp:40–71::GetLowestClosedPathInfo`; retain zero-area rejection, cached area/sign updates and lowest-vertex admission order |
| `clipper2/offset/do_group_offset.rs::offset_single_point` | `clipper.offset.cpp:514–546::DoGroupOffset`; preserve callback timing, rounded ellipse count, int-sized square extent, sign and reverse-path handling |
| `clipper2/offset/set_arc_tolerance.rs::execute_internal` | `clipper.offset.cpp:599–660::ExecuteInternal`; remove only the unobserved error-code member and branch: both pinned source and retained integer port initialize it to zero and never assign a nonzero value |
| `clipper/offset/expolygon.rs::append_offset_hole` | `ClipperUtils.cpp::c2_offset_expolygons`, retained-hole block; admission precedes area inspection, zero-area pop and positive-area reversal |
| `clipper/polytree/outside_in.rs::attach_misfit` | `ClipperUtils.cpp:1244–1268`; one misfit's area/bbox/enclosure candidate walk, first-parent insertion and moved flag |

## Explicit wrapper scale and bounded deferrals

`PolyTree::into_expolygons`, outside-in traversal and stray-hole attachment take
an explicit `CoordinateScale`. `AABBTreeIndirect.hpp:231–237::BoundingBoxWrapper`
uses `SCALED_EPSILON`: its integer inflation is **100** at normal scale and **10**
at large-printer scale. Outside-in's floating bbox checks use the same project's
scaled epsilon (`ClipperUtils.cpp:1244–1268`). No coordinate inference, global
state, implicit Normal default, `Option` mode, or widened tolerance is allowed.

Call-site paths below are relative to `crates/ares-core/src`. Already available
project scales are passed by
`project_slice/support_material/tree/overhangs/finalize.rs::SmartOverhangs::finalize_layer`,
`project_slice/perimeters/arachne/walls.rs::generate_surface_walls`, and
`project_slice/perimeters/classic/extra_perimeters/mod.rs::generate`. Outside-in scales come from
`fill/concentric_sparse.rs::fill_concentric_sparse`,
`project_slice/raft/fills/patterns/concentric.rs::concentric`, and
`brims/ears.rs` plus `project_slice/gcode_emit/brim/{combined,owner}.rs` through
`brims/ears/paths.rs::from_area`. These are scale argument
adapters, not changes to their support, fill, wall or brim algorithms.

The following existing no-scale signatures remain **normal-only temporary
compatibility shells** around the single C2 backend. Each conversion explicitly
passes `CoordinateScale::Normal`; no default or fallback chooses it. Their
large-printer attachment behavior (100 rather than upstream 10) is deferred and
must not be claimed as parity. Do not expand project-scale plumbing here.

| Deferred chain | Exact conversion/plumbing point |
| --- | --- |
| ExPolygon Difference/Intersection/Xor | `clipper/boolean_ex.rs::run` after `backend::clip_tree` |
| ExPolygon-to-Polygon boolean and safety-clip variants | `clipper/boolean_ex.rs::run_paths` after `backend::clip_tree` |
| Polygon boolean-to-ExPolygon and safety-clip variants | `clipper/boolean_ex.rs::run_polygons` after `backend::clip_tree` |
| ExPolygon union | `clipper/boolean_ex.rs::union_expolygons` tree conversion |
| Polygon subject / ExPolygon intersection | `clipper/boolean_ex.rs::intersection_polygons_ex` tree conversion |
| Polygon safety-offset union | `clipper/boolean_ex.rs::union_safety_offset_ex` offset-tree conversion |
| Single/multiple ExPolygon offsets | `clipper/offset/expolygon.rs::{offset_expolygon,offset_expolygons}` normalized-tree conversions |
| Sequential/opening/closing ExPolygon offsets | `clipper/offset/expolygon.rs::offset2_ex_with_interstage` final tree conversion |
| Variable inward offset | `clipper/variable_offset.rs::variable_offset_inner_ex` Difference-tree conversion |
| General union-to-ExPolygon | `clipper/polytree.rs::union_ex` tree conversion |
| Bridge closing without a scale argument | `project_slice/prepare_infill/external_surfaces/merge_bridges.rs::merge_bridges` offset-tree conversion |
| Raft closing without a scale argument | `project_slice/raft/fills.rs::closing_polygons` offset-tree conversion |
| Surface opening without a scale argument | `project_slice/prepare_infill/surface_type_detection/geometry.rs::opening` final offset-tree conversion |

Separately, the pre-existing
`geometry/region_expansion/wave_seeds/aabb.rs::Item::centroid` evaluates
`min + max / 2`, unlike pinned `AABBTreeIndirect.hpp:237`'s `(min + max) / 2`.
Do not fix it in this slice. It requires a separate public RED with an observable
fixture before a source-cited correction; record it independently at handoff.

The existing `brims/ears/paths.rs::from_area` also runs an additional pre-loop
Douglas–Peucker pass after the initial half-spacing offset, unlike
`Brim.cpp:862–873::tryExPolygonOffset`. Its isolated removal left all eight
public API byte streams unchanged against a clean baseline, so it was reverted
rather than retained as an unverified parity fix. This caller discrepancy is a
brim follow-up, not an offset-engine fallback or an accepted geometry GREEN.

## Scope and verification gate

Include closed booleans with all wrapper fill rules, PolyTree-to-ExPolygon output,
open clipping, Z clipping, offset joins/caps, safety offsets and sequential offsets
used by existing geometry consumers. Existing higher-level variable-offset and
simplification adapters must call this single engine and retain only behavior
still owned by the beta wrappers. Remove orphaned Clipper1 state/repair/round-policy
machinery rather than emulating its deleted options.

Defer TriangleMeshSlicer canonical ordering/coplanar handling to the separate
subsequent slice. Do not bundle walls, fill, support, configuration, GUI, file I/O,
ClipperD convenience APIs, rectangle clipping or triangulation not needed by this
wrapper boundary. No new public geometry API solely for testing.

After spec approval, first commit a targeted **public RED** witness through an
existing public slicing/path API or CLI parity case, with immutable beta-oracle
bytes or independent literal expectations. Select explicit nested/touching-hole,
collinear/spike, offset-threshold/arc-cap, and coincident/new-intersection Z cases;
new tests contain no private-module calls, mocks, loops or computed expectations.
Source-only diagnostic probes may localize failures but cannot replace the public
RED or claim official application parity. New case/function/fixture names describe
behavior; existing names are not renamed by this slice. Any required instrumented producer must
be separately identified and approved; old producer outputs cannot become beta
expectations by relabeling.

Then port the source boundary, run only the selected RED/GREEN filter,
`cargo fmt --all -- --check` and
`cargo clippy --workspace --all-targets -- -D warnings`, and submit the
implementation REVIEW-REQUEST. Preserve ordered outputs and exact assertions;
unresolved unrelated migrations are reported, not hidden by skips or tolerances.
Browser WASM/native integration and FULL gates belong to the integrator after
cutover. This document authorizes neither a FULL run nor implementation before
strategy review.
