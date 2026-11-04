# Tree support complete-output parity

## Goal and seam

Expanded multi-tool validation has a confirmed RED baseline/modifier input:
the GCode adapter lacks GCode.cpp4971-5075 role/region-to-extruder bucketing
and5241/5308 traversal/set_extruder. Header count-as-ID is a separate defect,
not the body fix. The original inherited-second-filament test and complete
comparator remain unchanged. This full source port is the
next independent milestone after the baseline body gap was reproduced on
`94d33b59`; the old modifier input was rejected rather than supported.
The current batch is explicitly single-tool; its support witnesses cannot
satisfy the multi-tool consumer boundary.

The finite merge batch includes the installed Normal/Grid/Snug and complete
Tree style/manual dispatch, canonical modifier flows and painted slab/vertical
projection. The final gates and separately deferred painting combinations are
listed in the plan; historical slice descriptions below are source boundaries,
not claims that already integrated behavior is still unported.

The final shared Fill slice follows `libslic3r.h::EPSILON/SCALED_EPSILON`
and `FillBase.cpp::could_take_*`, `take_limited`, `take_next`, overlapping
marking and cap emission. Pass the selected coordinate scale's epsilon
through `fill/base_support/` rather than fixed lattice constants. Do not
change accepted 3MF fixtures or the complete-output comparator.

The support-layer field slice ports Layer.hpp::lslices separately from
SupportLayer::support_islands into support_material/install_layers.rs and
the support_entry/skirt_emit planner consumers. Non-Organic source Tree
retains skirt-range lslices, empty actual islands and distinct base areas;
Common-produced Normal/Organic keeps its actual containment islands.
Include real public Slim tall-skirt 3MF output comparison, targeting
roof/floor leakage into the support retraction decision. Unrelated Tree
geometry, Snug/manual and platform acceptance remain mandatory adjacent
work, not reasons to relax the complete comparator.

The Tree containment slice ports Layer.hpp::SupportLayer base_areas,
PrintObject.cpp::add_tree_support_layer and GCode.cpp7620-7632
needs_retraction into support_material/install_layers.rs and the borrowed
gcode_emit motion view. Include exact whole-polyline/one-polygon containment
and the stInnerTree provenance; preserve the skirt-only lslices boundary.
Use the public exported Slim 3MF for a single-CPU native-witnessed travel
window regression, alongside all unchanged seven complete comparisons.
No Normal/Organic semantic expansion or weakened full-output acceptance.

The project-root slice ports TreeSupport.cpp::generate1724-1781,
create_tree_support_layers1157-1201 and final layer cleanup2640-2648 to
non_organic/generation.rs and raft_layers.rs, consumed by project.rs.
Include ordered smart overhang/contact/height/drop/smooth/drawing/toolpath
execution, normalized style, the complete raft/gap prefix, and whole-Print
brim/skirt inputs. The seven public 3MF cases remain the acceptance seam.
Source PrintObject.cpp4646-4676 owns supported-facet slab projection and
vertical enforcer points; ProjectVolume currently lacks this data, so the
facet model/parser and projection remain required before complete Tree
acceptance. Absence of that port is not permission to silently skip painted
support or to claim manual/auto parity. Full-region stBottom collection,
source object mesh bounds, exception/cancellation and platform execution
also remain mandatory. Adjacent generic slicing stages are dependencies,
not a replacement support pipeline; no discarded invocation before a guard.

The toolpath driver slice ports TreeSupport.cpp1210-1693 to
non_organic/toolpaths.rs and its fill/perimeter/interfaces/base/raft modules.
Its dependencies are source FillBase.cpp275-319 and FillRectilinear.cpp
3387-3446/3610-3634, MultiPoint.cpp48-55, BoundingBox.hpp/BoundingBox.cpp
defined-box merge, and ShortestPath.cpp1026-1068's optional-start/Loop
ordering. Include physical-height Flow, nominal interface densities,
RoofFirst original-area one-wall NoSort, per-area Base NoSort and Lightning,
first-layer sheath, all raft/gap paths and exact whole-destination ordering.
Existing ported Fill kernels are dependencies, not a generic Normal fallback.
Root, project all-region/painter/raft inputs, exception/cancel/worker behavior
and complete public style/manual/Snug output remain mandatory open acceptance,
not removed or deferred output scope. The public seam is the unchanged
seven-case 3MF full comparator with real options/native extrusion witness.

The next Loop slice ports TreeSupport.cpp1244-1290 `_make_loops` and
make_perimeter_and_inner_brim to non_organic/toolpaths/loops.rs; support
entity construction follows ExtrusionEntity.hpp638-651, chaining follows
ExtrusionEntity.hpp457-469 and Collection.cpp65-73, and G-code follows
GCode.cpp5752-5805/5979-5990. Include initial loops even for wall_count=0,
ordered child insertion, flow float narrowing, nearest projection and
native0.001mm snap, fitting-preserving split, seam gap and forward wipe.
Keep shared path/NoSort/MultiPath behavior intact. Reuse existing public
Slim/Strong/Hybrid RED and Organic output controls; no private/source-pin
tests or weakened comparison. Multi-path loops/adaptive-PA and complete
Tree producers/toolpaths/root remain required adjacent work before full
Tree/global option acceptance; no fabricated pre-guard consumer.

Normal Snug is an explicit acceptance case: export normal(auto)/snug using
PrintConfig.cpp's enum keys, verify the real 3MF settings, require native
support extrusion and compare the complete output. Port its behavior from
SupportMaterial.cpp; neither rejection nor Grid/Tree substitution satisfies
this seam. Keep existing Organic/Slim/Strong/Hybrid fixture settings and
complete-output assertions intact.

Non-Organic toolpaths port TreeSupport.cpp1210-1354 and1355-1693 into
support_material/tree/non_organic, with shared support entities corresponding
to ExtrusionEntity.hpp638-651 and GCode.cpp6178-6240. Actual loops retain
extrude_loop5744 behavior, including nearest projection, seam clipping and
wipe. Mixed loop/path per-area no_sort collections retain source nesting and
order through simplification, chaining and G-code; no closed-path substitute
or second output pipeline. All roof/floor/Roof1/base and raft-prefix behavior,
Lightning consumption and actual root integration are required before GREEN.

TreeSupport.cpp2640-2648 owns final drawing-layer removal. Remove heights
strictly below EPSILON after Lightning/hole work; retain surviving order
and typed group identity. The owning Vec supplies upper/lower adjacency,
not copied pointer fields. Actual raft-prefix installation and source
toolpaths/root remain following owners, not generic Normal dispatch.

Tree Lightning follows TreeSupport.cpp2444-2507 into drawing/lightning.rs
and Fill/Lightning/Generator.cpp104-145,236-285 into the existing Generator
support constructor. Include lower-layer geometry, ascending order/Z lookup,
native float density/width/radius arithmetic, supplied overhangs and fixed
45-degree support growth. Exclude the compile-time-disabled local-island
branch; cancellation, backup/install/root/toolpaths remain required adjacent
owners. Add a separate Strong+Lightning public3MF full-output RED with option
readback and native support witness; keep original Strong inputs unchanged.

The hole-propagation slice uses TreeSupport.cpp2508-2622, Point.hpp192,
MultiPoint.hpp53-68 and the existing Tree collision owner. Rust destination
is non_organic/drawing/hole_propagation.rs plus contour_intersection.rs.
Include source float thresholds, first closest-vertex ties, zero Points,
ordered lower groups, offset/translation state, Roof1 in-place identity and
clipping. Reuse the public Slim/Strong/Hybrid full-output RED; no private
geometry test seam. Lightning, backup/install, actual root and complete
output for every style/manual/Snug remain mandatory following slices.

Separate Strong/Hybrid public overhang cases use PrintConfig.cpp330-331
tree_strong/tree_hybrid and the SupportParameters.hpp190-204 normalization.
Export and verify actual 3MF options, require native support extrusion and
the existing complete-output comparator. Do not alter existing Organic/Slim
inputs or assertions; RED at dispatch records an open port, not acceptance.

The drawing finishing slice targets drawing/{hybrid_expansion,trimming,groups}
from TreeSupport.cpp2330-2444 and1904-1975. Include polygon-only stepped Hybrid
growth and repeated object trim, actual sharp-tail/region-bridge clearance,
ordered Base/Roof/Floor/Roof1 groups, source flags and small holes, adhesion
slices and printed interface IDs. Kind/index identity replaces pointers;
polygon overlaps retain ExPolygon.cpp156-201 boundary/containment semantics,
including parameter direction and boundary-only contacts.
do not substitute Normal trimming or discard groups. Existing public Slim
RED remains the seam. Hole/lightning, untyped backup, installation and real
root/output remain mandatory subsequent behavior, not acceptance omissions.

The next drawing prefix targets non_organic/drawing.rs plus drawing/
node_shapes.rs, collision.rs and areas.rs from TreeSupport.cpp1978-2214 and
1884-1904::avoid_object_remove_extra_small_parts. Include exact shape rounding,
first-layer brim, largest component with f32 max, current/future collisions,
Hybrid/virtual ownership, node roof mm caps and roof/base clipping order.
Preserve the agreed public Slim RED, not private shape kernels. Remaining
draw_circles2215-2650 behavior and real generation/toolpaths/project integration
remain mandatory for all-Tree acceptance; no guard change or fallback here.

TreeSupport.cpp2215-2332 and helper80-91 target drawing/bottom_interfaces.rs:
include complete downward true-surface search, sorted Z bands, footprint-only
gap clearing, support-index floor base selection and10-coordinate separation.
Borrow real object layer metadata and all-region stTop/stBottom sets, never
substitute adjacent support Z, raw slices or BottomBridge sets. Hybrid first
layer expansion, trim/groups, hole/lightning and root remain required next.

Second-pass2937-3113 targets non_organic/dropping.rs/neighbor_movement.rs;
2740-2757 line intersection caching targets line_cut.rs. Include clipped
polygon descent/centroid, all active rejection/movement/radius branches,
f32 reciprocal weighting followed by rounded coordinates, cache both line
directions, and direct parent intersection without cache substitution.
2651-3170 targets drop_nodes.rs, including signed sorted distance propagation,
radius precache, descending group/merge/drop and unsupported pruning. Omit
overwritten movement calculations, disabled blocks, debugging and profiling.
Actual drawing/root, native workers and cancellation ownership are required
next integration, not acceptance deferred outside the all-Tree contract.

First-pass merge2844-2934 targets non_organic/merging.rs. Include every active
polygon/pair/multi-neighbor branch and source next-node/collision operations;
constructor634-665, radius1801-1828 and move-cache2651-2680 target
drop_settings.rs. Replace temporary Vec merged-neighbor storage with VecDeque
for constant-time front insertion and ordered tail append. Exclude unused
size_t radius overload, logs and timers. Serial reference group traversal is
not native TBB scheduling acceptance; actual second pass, workers, driver,
draw/root and complete public Slim parity remain required, not optional scope.

TreeSupport.cpp2773-2847 per-part grouping and MST construction target
non_organic/grouping.rs, with position_map.rs preserving Point.hpp496-499
PointHash and the actual reference's bucket traversal/growth. Include
first-containing/closest-vertex ties (MultiPoint.hpp53-68), unsupported FIFO
inputs, duplicate-position overwrite, distinct ordered vertices and index
adjacency. Arena links replace native container pointers without O(N²)
front-vector inserts or randomized/sorted ordering. Omit debug MST storage.
Native worker/merge and drop/root consumption remain required within the
agreed public Slim RED; this dependency alone is not output acceptance.

The active movement dependency targets non_organic/movement.rs from
TreeSupport.cpp581-628 and69-77, ExPolygon.cpp204-223 and Point.cpp90-128.
Include offset/union, ordered projections, affine truncation, normal rounding,
source EPSILON and contour/hole boundary semantics. Include the consumed
Polygon.hpp196-205::to_lines behavior for one/two-point contours (first-point
projection). Exclude uncalled legacy
movement helpers and unrelated geometry refactors. Drop/root integration is
required next behavior; the existing public full-output Slim RED, not private
kernel tests or an Organic substitute, remains the agreed acceptance seam.

Drop dependencies target non_organic/spanning_tree.rs (MinimumSpanningTree.cpp
/hpp::prim and adjacent_nodes), pruning.rs (TreeSupport.cpp3133-3170) and
smoothing.rs (3172-3250). Include adjacency/tie order, actual arena chain
rewiring and source buffered smoothing. Omit unused preview leaves/vertices,
queue layer IDs, timers and logs; do not omit active graph or ordering.
Actual per-part container ordering, merge/move/drop consumers and native
workers remain required. These stay within the existing Slim RED, not private
kernel tests or independent GREEN features.

TreeSupport.cpp3402-3634 contact generation targets non_organic/contacts.rs
and sampling.rs, with libnest2d/placers/nfpplacer.hpp::EdgeCache as its sampling
dependency. Include native grid/insertion, Hybrid duplicate polygon references,
vertical enforcers and f32 node statistics. The per-layer implementation does
not establish native parallel insertion order; actual worker/root consumption
and ordered-byte Slim acceptance remain mandatory. No private sampling tests
or an Organic/Grid substitute are acceptance evidence.

SupportNode (TreeSupport.hpp) / TreeSupportData::create_node and
TreeSupport.cpp3251-3400::plan_layer_heights target non_organic/nodes.rs and
heights.rs. Include native inheritance/merged-parent identity, movement and
radius growth, layer schedule and contact redistribution/gap/interface updates.
Borrow the existing PlannedLayer records. Exclude the native static global
ownership, debug destructors and logs; defer actual layer allocation to the
required draw/output consumer, not an optional omission. Public Slim remains
RED while these dependencies lack the full generating consumer.

TreeSupportData geometry dependency (TreeSupport.cpp3650-3794) targets
tree/non_organic/data.rs. Include per-layer simplify/cumulative outlines,
native radius ceiling with unscaled EPSILON, collision offset/simplify and
lower-layer avoidance erosion/union caches. Borrow cached outputs, retain
first insertion; iterative dependency unwind replaces only recursion storage.
Defer node ownership and contact/drop/draw/toolpath/root consumption as
required next source slices, not optional acceptance exclusions. Exclude
unused recursions identity, profiler and disabled USE_SUPPORT_3D alternative.
The Slim full-output test remains RED until the full generator consumes this
dependency. Compilation alone is neither feature GREEN nor a merge gate.

Non-Organic style scope: SupportParameters.hpp190-204 normalization feeds
TreeSupport.cpp::TreeSupport::generate1724-1788. Destination:
support_material/tree/non_organic with actual project.rs dispatch. Include
generate_contact_points, plan_layer_heights, drop_nodes, smooth_nodes,
draw_circles and generate_toolpaths for Slim/Strong/Hybrid, preserving source
node/geometry/order semantics. Exclude preview/debug file I/O and UI only;
manual/enforcer and worker/runtime parity are deferred required slices, not
optional acceptance omissions. Existing layer/toolpath scaffolding remains
temporary compatibility storage around upstream owners. Start with a Slim
real3MF full-output tracer, then implement its complete native consumer chain;
do not bulk-add imagined internal tests or route it to Organic.

The FillBase.cpp1312-1325::BoundaryInfillGraph::dir_prev/dir_next and
2425-2455 vertical-arch loop rewrite targets fill/base_support/link.rs.
Include source could_take availability and current-endpoint trimmed-length
gating before taking the previous endpoint; retain the remaining connector
phases. This slice restores availability checks using the existing epsilon;
the shared epsilon16 versus native100 discrepancy remains a separate required
boundary case, not a claim of exact threshold parity. Defer other discrepancies to source-cited
slices. Acceptance stays at the existing real3MF complete-output seam,
without private graph tests or changed fixtures/assertions.

Rewrite Orca Tree support, not an Ares-designed support pipeline. The agreed
seam is real 3MF project bytes/options through the existing core slicer to
complete G-code bytes, paired with actual Orca on the identical input.
Normal support changes must remain unmerged until Tree is complete.

After the drawing slice, prioritize TreeSupport3D.cpp::generate_support_areas
integration at support_material/project.rs over further isolated unused kernels.
Remove support_type/support_style entry guards through actual upstream style
dispatch and enter public complete-output comparisons; fix remaining behavior
from those differences, without incomplete-output acceptance or other-style
fallback. All tree dead_code warnings must be eliminated before integration
submission, with no allow(dead_code) suppression.

## Organic root integration

Shared FillSupportBase sampling must include FillRectilinear.cpp3610-3633 /
ExPolygonWithOffset394-397 float-to-coord_t offset truncation and native
float trigonometry at the rotate-back boundary. The Rust owner remains
raft/fills/support_base.rs, not a separate Tree approximation. Validate through
the unchanged complete-output seam plus existing Normal/raft controls; this
numeric repair alone does not complete styles, worker/cancel or Tree parity.

Connect TreeSupport3D.cpp::generate_support_areas (3405-3645), its active
TreeSupport.cpp::detect_overhangs consumer, model-volume precalculation,
initial tips/pathing/nodes, complete organic_draw_branches and shared
SupportCommon.cpp::generate_interface_layers/generate_raft_base/
generate_support_layers/generate_support_toolpaths to support_material/tree
and project.rs. MutablePolygon.cpp/.hpp::smooth_outward, clip_narrow_corner
and indexed MutablePolygon/range are required drawing/interface dependencies,
not a standalone milestone. Reuse the existing SupportCommon tree hollow-wall
rewrite in raft/fills/tree rather than a Grid/Normal substitute. Preserve
source Organic-vs-other-style dispatch, time-dependent overhang cutoff,
worker-dependent merge partitioning, source top/interface layer ownership,
bed clipping and ordered paths. Included Organic/default auto/manual root
must execute through the unchanged public real3MF complete-output seam.
Other dispatched tree styles, parallel/cancel execution, all-printer/options
and final runtime parity remain mandatory adjacent acceptance; none may be
silently routed to Organic or Normal. No commit until full required Tree
behavior and zero tree dead_code are verified; no lint suppression or relaxed
fixtures/assertions. Normal behavior must remain unchanged by shared consumers.

## Organic branch drawing and contact aggregation

Port TreeSupport3D.cpp:3646-4132::recover_pending_branch_roofs and the active
organic_draw_branches mesh/contact/aggregation consumer to organic/drawing.rs
with traversal, mesh_slices, contacts and aggregation children. Consume actual
collision smoothing and indexed branch extrusion, then the existing
TriangleMeshSlicer.cpp::slice_mesh/make_loops Positive boundary. Retain native
f32 XY scaling, midpoint planes, root/tip Z ranges, depth-first parent order,
bed/collision safety clipping, tiny-fragment removal, trimmed-front contact,
gracious and non-gracious model landings, verylost descent, roof recovery,
contact-only contributions and per-tree-before-cross-tree union order. Share
Common.hpp inverse Z across smoothing/drawing and preserve model-volume cache
clear ownership from TreeModelVolumes.hpp. Include source-owned propagation
and contact retention, not a Normal/legacy fallback. Defer the final
4133-4174 outward smoothing/layer allocation to its required MutablePolygon.cpp
source slice; do not substitute offset/simplification or expose incomplete
drawing at the project entry. Worker/cancel contracts, root/style dispatch,
actual G-code consumer and complete output/runtime acceptance remain required.
Reuse the agreed real3MF/core/complete-output seam without private kernel tests,
source pins or changed fixtures/assertions. This prefix is not behavior GREEN.

## Organic active collision smoothing

Port TreeSupport3D.cpp:3030-3292::organic_smooth_branches_avoid_collisions and
3673-3736::organic_draw_branches flattened down-links to organic/smoothing.rs
with cache, spheres, constraints, movement and links children. Include the
currently enabled TREE_SUPPORT_ORGANIC_NUDGE_NEW behavior: zero-forced cache
radius, locked spheres, Z-span propagation, linked-layer reachability, native
collision and Laplacian nudges, previous-position iterations, stopping rule and
f32 scaled writeback. Include Common.hpp::layer_idx_ceil/floor raft-aware inverse
Z mapping. Rewrite AABBTreeLines.hpp::build_aabb_tree_over_indexed_lines /
squared_distance_to_indexed_lines, AABBTreeIndirect.hpp::Tree<2,double> build /
bounded recursive query and Line.hpp::line_alg::distance_to_squared in the
existing geometry/line_distance_tree/floating boundary; preserve float bounds,
QuickSelect ordering, containing-left-before-right traversal and strict ties.
Do not substitute integer nearest_mm or copy graph geometry. Represent an
uninitialized no-hit point by absence; it cannot generate a positive collision
depth or nudge. Omit unused cached collision-radius storage, disabled OpenVDB
and diagnostics. Serial kernel scheduling preserves native previous-position
reads; actual worker/cancel and parallel performance, drawing/root and full
Tree runtime acceptance remain mandatory. Public seam remains unchanged real
3MF/core/complete ordered-output tests; no private/source-pin coverage or guard.

## Organic branch mesh extrusion

Branch geometry retains TreeSupport3D.cpp::discretize_circle2899-2908 and
extrude_branch2966/2986 float trigonometry. For caps, widen the f32 cosine
only for the double center expression; compute radius-times-sine in f32.
The double acos discretization and source f32 angle accumulation remain
unchanged. Complete output is still required at the existing public seam.

Port TreeSupport3D.cpp:2807-3030::triangulate_fan, triangulate_strip,
discretize_circle and extrude_branch to organic/branch_mesh.rs with circle,
triangulation and concrete vector-operation children. Use the existing graph's
borrowed SupportElement path; retain indexed f32 vertices/u32 faces for the
existing mesh_slicer seam rather than expanding triangle soup. Include native
closest/strict-tie zigzag face ordering, f32 circle accumulation and radius
unscaling, f64 path direction/interior bisectors, lower/upper hemispheres,
layer-zero buildplate flat foot and returned Z span. Share the existing
TreeSupportCommon.hpp::layer_z port with InterfacePlacer. Exclude the unused
move_bounds argument, disabled circles_intersect alternative and debug OBJ
output. Defer active collision smoothing, mesh slicing/drawing/root and complete
Tree runtime/output acceptance; valid positioned paths and hemisphere ring
preconditions remain actual caller obligations, not fallback geometry.
Reuse the agreed real3MF/core/complete-output tests without private mesh tests
or changed fixture/assertions. This unconsumed source slice alone is not GREEN.

## Organic branch positioning

Port TreeSupport3D.cpp:2550-2804::create_nodes_from_area, set_points_on_areas,
simple/gracious model contact and remove_deleted_elements to organic/nodes.rs
with model_contact.rs and compaction.rs children. Include base projection,
ascending parent marking/point preservation, native orphan/tip deletion and
parent-result invalidation, upward gracious landing until the first failed
intersection or merge point, source simple landing when no gracious placement
exists, and tail-move deletion with exact parent-index maps. Borrow graph
geometry and cached placeable areas; use layer/node indices instead of aliased
native pointers, without cloning the graph. Native simple landing is source
behavior, not a Normal-support fallback. Exclude UI/log and debug-only
connectivity diagnostics; defer cancellation, actual root caller/index contracts,
triangulation/smoothing/drawing and complete Tree runtime/output acceptance.
Reuse the unchanged real3MF/core/complete-output seam; no private graph tests,
source-pin coverage, relaxed fixtures or held submission.

## Organic layer pathing

Port TreeSupport3D.cpp:2446-2552::create_layer_pathing to organic/pathing.rs.
Include descending layer traversal, source merge scheduling/literal coordinate
ratios, actual increase_areas_one_layer and merge_influence_areas consumers,
stable removal of vanished/bypassed areas, bypass-before-merge insertion and
safe-unioned graph append. Preserve new_element sampled before final append,
the greater-than-one merge condition and count-before/unchanged-vector-length
schedule outcome. Own the source SupportElements graph, not another node store;
borrow adjacent layers disjointly and move output geometry/parents without copies.
Exclude only source progress/timing/UI diagnostics; tiny-area diagnostics do
not reject or skip native output. Defer cancellation API, actual worker sourcing/
parallel order and branch positioning/drawing/root to their named consumers.
Use the unchanged public real3MF/core/complete-output tests, not private graph
tests or a partial output replacement. Full Tree acceptance remains open.

## Organic merge dispatch

Port TreeSupport3D.cpp::merge_influence_areas_leaves/two_sets/merge_influence_areas
to organic/merge_areas.rs with compaction and ordering children. Include the
actual pair consumer, retry-after-merge, native move compaction, unchanged node
vector length, two/four-element worker-dependent buckets and adjacent rounds.
Port AABBTreeIndirect.hpp::build_recursive/partition_input input ordering with
exact median-of-three QuickSelect and X-first axis ties. This consumer never
queries tree nodes, so do not allocate unused generic AABB query storage.
Accept worker count as a derived runtime input, not a hardcoded single worker.
Defer its actual execution-context source, native parallel performance/order
validation, cancellation API, pathing/draw/root and complete output acceptance.
The trusted source caller supplies more than one successful grown node; numeric
equivalence covers native integer-valid geometry, not overflow extremes.
Reuse the unchanged real3MF/core/complete-output RED seam; no private tests,
fixture relaxation, replacement merger, unsupported guard or held submission.

## Organic pair merging

Port TreeSupport3D.cpp:2044-2256::merge_support_element_states/
merge_influence_areas_two_elements to organic/pair_merge.rs and state.rs child,
plus the consumed default-distance1458-1567::move_inside_if_outside projection
to organic/move_inside.rs. Include gracious/minXY compatibility, bbox rejection,
real-radius growth limits, route overlap/erosion thresholds, closest boundary
ordering, roof recovery, foot normalization, native avoidance ordering, source
parent concatenation/area clearing and final bbox union. Native merge state
defaults reset bits/type/radius/print_z/result; increased_to_model_radius is set
by the pair consumer before the node is observable. No dummy node fallback.
Defer leaf/set compaction, source AABB ordering/worker-dependent buckets,
create_layer_pathing/draw/root, cancellation API and actual consumer execution.
No substitute merger, private tests or fixture/assertion relaxation.

## Organic one-layer growth selection

Port TreeSupport3D.cpp:1781-2042::increase_areas_one_layer and the consumed
SupportElementMerging fields1749-1767 to organic/layer_growth.rs and focused
movement/settings-order children. Include native projected-radius movement,
ordered configuration deduplication, paired min-distance retries, cached
slow/fast offsets, open-line error-area expansion, first-success state updates,
merge bypass and parent reset on failure. Keep the source area vectors as
owned outputs and borrow wall geometry without an extra collision copy.
Defer merging/AABB consumers, pathing/drawing/root and actual cancellation
API; only source debug/UI diagnostics are excluded. Include enabled
TREE_SUPPORTS_TRACK_LOST state writes: successful error-recovery sets lost;
complete growth failure sets verylost. The latter is consumed by the active
draw lower-extension logic at TreeSupport3D.cpp:3950-3955, not debug-only.
Serial exclusive cache borrowing is not a claim of native parallel performance
or output parity. Existing real 3MF complete-output tests remain the seam;
no private kernel tests or replacement fixture/assertions.

## Organic single-area growth

Port TreeSupport3D.cpp:1588-1733::increase_single_area and its
SupportElementInfluenceAreas output to organic/single_area.rs and its radius
child. Include TreeSupport3D.hpp::propagate_down/getEffectiveDTT/radius helpers,
movement against wall restrictions, route correction, radius catch-up and
buildplate foot growth. Reuse native radius quantization; preserve the ignored
return of the const-input polygon simplifier at source1626, without changing
the input influence area. ClipperUtils.cpp::diff_clipped bounds the subtracting paths.
Preserve strict/non-strict area comparisons and native Coord narrowing.
Defer increase_areas_one_layer, merging, create_layer_pathing, graph drawing and
root execution to subsequent complete source slices; no placeholder consumers.
Public Default/angle-zero complete-output tests remain RED at the existing
entry guard until that root exists. Component compilation is not behavior GREEN.

## Organic initial influence generation

Port the active TreeSupport3D.cpp:1246-1456::generate_initial_areas body and
1017-1071::generate_raft_contact/finalize_raft_contact in organic/initial_areas
with generation and raft_contact child modules. Borrow PrintObject's layer
slices, raft expansion and raft-prefixed overhangs; enumerate native support
indices, compute connection length and force-roof selection, then consume
owned union_ex roof/regular components in source order through roof_sampler.
Include raft contact Z/expansion, clearing pre-contact tips/contact slots and
ordered filtering of tips inside expanded raft. Reuse EdgeGrid's closed-edge
signed nearest distance kernel: EdgeGrid.cpp:1047-1165 and1178-1271 share the
strict radius search and signed distance outcome needed by this caller; no
new contains-only approximation or all-geometry scan. Keep threshold narrowing,
bbox padding and original native computational conditions.
Disabled #if0 insertion-lag code, its unused max_overhang_insert_lag setup,
and debug/UI diagnostics are excluded; source
cancellation callbacks remain deferred to the actual cancellable API boundary
(current SlicerContext has no cancellation contract). Parallel scheduling and
its output ordering, graph propagation/drawing, root installation, Fill failure
mapping and real caller numeric/index contracts remain mandatory for runtime
acceptance. No partial geometry return is a substitute for full Tree output;
keep the original public Default/angle-zero and KSR tests unchanged.

## Organic overhang roof sampling

Port TreeSupport3D.cpp:1099-1235::sample_overhang_area into
organic/roof_sampler.rs and its roof_stack child. Consume raw infill lines,
Euclidean tip sampling, status classification and InterfacePlacer/RichInterfacePlacer
directly. Include collision/avoidance padded roof descent, minimum-area stop,
previous-roof next-layer filtering, empty-fill roof truncation, source outline
sampling when too few points remain, buildplate routing and pending roof depth.
Keep ordered ownership transfers and native float/coordinate arithmetic; the
outline sampling is source behavior, not a Normal-support fallback. Branch graph,
branch drawing, generator/root installation and typed Fill invariant-failure
mapping remain required before complete output acceptance. The unchanged public
Default/angle-zero RED remains the test seam; no private/source-pin test or guard
removal is accepted as a replacement. Numeric/caller contracts still need actual
generator execution, not merely compile-only coverage.

## Organic interface and tip placement

Port TreeSupportCommon.hpp:683-760::InterfacePlacer and layer_z/initialize,
TreeSupport3D.cpp:851-1013::RichInterfacePlacer, and the tip-owned fields of
TreeSupport3D.hpp::SupportElementState/SupportElement in organic/interface_placer,
rich_interface_placer and support_element modules. Include owned roof append,
contact/interface/base-interface depth routing (including the reserved layer),
native raft/physical Z, status-specific next-layer splits, hash-cell tip
deduplication, pending roof recovery and ovalisation flags. Source Polygon.cpp
make_circle uses error25, not a fixed25 vertices; retain circle truncation,
integer radius scaling and translated unscaled influence circles.
Use append-only owned layer storage and stable indices instead of raw pointers;
preserve source insertion order without geometry copies just for storage.
Include only layer creation fields/types required by this placer; SupportLayer
merge/reset, other layer types, graph propagation/merging and branch drawing
are deferred to their actual consumers. Roof sampler/root execution and
the prior Fill failure-type mapping still remain required; no private/source-pin
test, guard removal, Normal fallback or component GREEN replaces public output.
Source numeric/zero-hash-step and layer-index caller contracts remain to audit
before all-domain/runtime acceptance; do not add new clamps or rejections.

## Organic raw infill lines

Port TreeSupport3D.cpp:656-738::generate_support_infill_lines in
organic/infill_lines.rs. Include plain union_ex components (no raft closing
or safety offset), fresh Fill defaults, actual layer_id, component-local
contour bbox center, source roof ternary precedence, float spacing/density,
and selected SupportBase/Rectilinear/Honeycomb fillers. Reuse the existing
source FillSupportBase kernel by extracting it without arithmetic changes;
the raft shell keeps its preprocessing, reference and disabled alternation.
SupportParameters.hpp selects only those three patterns here, not the contact
Grid/Concentric pattern. Keep errors typed: Rust Clipper range errors are not
native InfillFailedException and must not be silently discarded.
The component-local catch of InfillFailedException (TreeSupport3D.cpp733-734,
FillRectilinear.cpp920::ASSERT_THROW) is deferred until the shared Fill rewrite
has an explicit corresponding invariant-failure category. ClipperError is
not that category. The current Result does not establish equivalence for
native infill invariant failures; auditing/mapping them at the real caller
is required before complete Tree acceptance.
Roof sampler/placer runtime consumption, branch graph and root installation remain required;
unchanged complete-output Default/angle-zero tests are the acceptance seam.

## Organic tip line sampling and classification

Rewrite TreeSupport3D.cpp:350-647::get_avoidance_status,
convert_lines_to_internal, evaluate_point_for_next_layer_function,
split_lines, polyline_sample_next_point_at_distance and
ensure_maximum_distance_polyline under organic/tip_lines.rs and
organic/tip_sampling.rs with an actual next_point child module.
Include ordered FastSafe/Fast buildplate/model classification, invalid-point
splits, next-layer status-specific queries, even-odd Polygon.cpp::contains
(border true), circular Euclidean sampling, source coordinate truncation,
closed-line farthest-point tie order, spacing reduction and fixpoint handling.
Consume the owned temporary infill/outline polylines rather than cloning
them just to rotate or clip; preserve points/order and native arithmetic.
Defer UI diagnostics, not the source fixpoint progression/break condition.
Actual infill generation, roof stack sampler, RichInterfacePlacer, branch
graph/pathing and runtime root remain required. Keep the agreed complete
public Default/angle-zero RED and original KSR/spacing assertions unchanged;
do not add source-pin/private-path tests or infer GREEN from compilation.
Signed squared-distance overflow, float-to-coordinate extremes, degenerate
closed singleton input and next-layer index preconditions must be audited
at real source callers before all-domain acceptance; no invented clamp/guard.

## Initial Organic overhang-area preparation

Continue `TreeSupport3D.cpp:1327-1380,1426-1444::generate_initial_areas`
in `tree/organic/initial_areas.rs`. Include least-restrictive collision/Fast
avoidance selection, union-ex/Miter rounding-error padding, regular-overhang
safe expansion, forbidden remaining-area compensation, source step selection,
roof extraction and safety-offset separation. Retain the source condition
that gates minimum-roof filtering on minimum-support area, even though it
looks asymmetric. Port its mesh-group defaults from TreeSupportCommon.hpp:
minimum roof area1mm^2, minimum support area0, support offset0.
The two returned area lists are the source task's regular/roof locals for
its following sampler, not a new support pipeline or printable Tree result.
Roof sampling, RichInterfacePlacer, raft contacts, branch graph/pathing and
installation remain required. The unchanged real Default/angle-zero tracers
remain the public RED seam; no private/source-pin coverage or new guards.

## Organic collision-safe expansion continuation

`TreeSupport3D.cpp:749-847::safe_union/safe_offset_inc` owns the shared
influence-area expansion used by `generate_initial_areas` and branch pathing.
Rewrite it in `support_material/tree/organic/area_growth.rs`, retaining the
existing project-bytes/complete-G-code acceptance seam. Included: narrow-area
union recovery using closed polylines, lazy subject-bbox collision clipping,
integer step selection and minimum step count, Round offsets, per-step
collision differences, periodic/final simplification using Common.hpp591's
strictly_simple=false and the final
unchecked-distance policy. Preserve the source negative-parameter geometry
return; UI diagnostics are deferred. No new rejecting guard is permitted.
Actual tip placement/pathing callers, roof placement, branch drawing and
support layer/toolpath installation remain required subsequent slices. This
unconsumed kernel cannot make the existing Tree public RED a GREEN or justify
merging Normal support. Do not add source-pin/private-path tests.

## Source-to-Rust scope

SupportBase outer-offset continuation rewrites
`FillRectilinear.cpp:391-437::ExPolygonWithOffset` and
`:2920-2958::make_fill_lines` at the existing `raft/fills/support_base.rs`
boundary. Include the outer-offset parameter, zero inner offset, retained
outer contours and OuterLow/OuterHigh pairing, with both Tree and raft real
parameter producers updated. Shared two-contour Rectilinear behavior remains
unchanged. Other connector/output differences remain required; retain the
same complete-output public tests and their unchanged fixtures/assertions.

SupportBase empty-line continuation is owned by
`FillRectilinear.cpp:3610-3634::FillSupportBase::fill_surface` and
`FillBase.cpp:2247-2296::connect_base_support`. Existing
`raft/fills/support_base.rs` must pass retained contours with zero vertical
lines to the consumed connector so its native empty-contour perimeters can
be emitted. Include this caller contract only; adjacent offset-contour and
remaining connector differences stay required, not acceptance exemptions.
Keep the same agreed project-byte/complete-output tests and original fixtures.

Tree path wipe continuation is owned by `GCode.cpp:6104-6127::extrude_path`
and `:320-358,435-497,7659-7670::calculateWipeRetractionLengths/wipe/retract`.
Destination: existing `gcode_emit/motion/support.rs`, resolved motion options,
and `motion/travel/wipe_sequence.rs`. Include Tree support/interface/transition
single-path forward storage, the source 0.2mm closest-end tail selection,
and zero-length eligible wipe lifecycle. Keep MultiPath joined reverse storage,
Normal and ironing path behavior unchanged. Other support toolpath/entity
differences and complete-output acceptance remain required subsequent work.
The unchanged public Tree project-byte/complete-G-code comparisons remain the
tracers; no private source-pin test or comparator normalization is added.

| Upstream owner | Rust destination | Included behavior |
| --- | --- | --- |
| `PrintObject.cpp::_generate_support_material`, `SupportParameters.hpp:190-204` | `support_material/project.rs`, private `tree/` configuration | Type/style normalization and dispatch |
| `Support/TreeSupport.cpp/.hpp::TreeSupport`, `SupportNode`, `TreeSupportData` | `support_material/tree/` contacts, nodes, layers, toolpaths | Auto/Manual, Hybrid/Slim/Strong, overhang detection, contact sampling, drop/smooth/draw, height planning and real paths |
| `Support/TreeSupportCommon.hpp::TreeSupportMeshGroupSettings/TreeSupportSettings` | private `tree/organic/` settings | Organic movement/radius/distance/layer calculations |
| `Support/TreeModelVolumes.cpp/.hpp::TreeModelVolumes` | private `tree/model_volumes/` | Collision, Slow/FastSafe/Fast avoidance, placeability and wall restrictions |
| `Support/TreeSupport3D.cpp/.hpp::SupportElementState/SupportElement`, `generate_support_areas`, `organic_draw_branches`, `generate_tree_support_3D` | private `tree/organic/` areas, pathing, branches | Organic influence-area growth/merging, node placement, collision-safe smoothing, mesh slicing and roof recovery |
| `Support/SupportCommon.cpp` layer/toolpath installation, `GCode.cpp::collect_layers_to_print/extrude_support` | existing installed support and G-code modules | Real support entities, ordered output, paired object/support layers |
| `Support/SupportMaterial.cpp:592/599/1164/1430/1613/1786`, grid pattern rotation and annotation handling | existing normal project adapter and geometry modules | All modifier regions, actual enforcers/blockers, nonzero angle |

Default Tree resolves to Organic, not Slim. Incompatible Normal/Tree style
choices first normalize exactly as `SupportParameters.hpp`. Existing normal
grid and raft adapters remain temporary shells for their upstream slices;
they must not become Tree fallbacks. Files remain below 400 lines, split by
source-owned behavior, without include-based splitting.

First dispatch tracer: a real overhang with NormalAuto + Organic must match
Orca's complete output after that owner normalizes its style to Grid. Retain
a Grid control and actual native positive support extrusion. Requested style
remains in exported config; normalization is generator-local, not a config
serialization rewrite. This does not prove any Tree branch generation.

The first small active Organic tracer exports Ender-3's real overhang model
with enabled TreeAuto, requested Default style and no raft. Verify exported
settings, one ModelPart object and actual native positive support extrusion;
compare the entire output through the existing ordered-byte seam. This drives
Tree's actual Default-to-Organic generator path, without changing the original
KSR fixture/assertions or permitting Normal fallback/empty support output.

Included: all listed computational behavior and style choices, both Tree
types and the shared Normal repair list. Deferred: UI/terminal progress,
debug drawing and viewer-only preview state. Adjacent multi-tool/sequential,
painted brim and unrelated viewer parity are not completed by this milestone.

## Acceptance

The settings slice ports Organic's `TreeSupportMeshGroupSettings`
configuration, `TreeSupportSettings` derived radius/movement/XY/Z values and
raft Z vector from `TreeSupportCommon.hpp:36-98,267-357,545-563` under
`tree/organic/`. Feed the quantizer's ignored buckets from the source tip-radius
enumeration in `TreeModelVolumes.cpp:158-183`. The existing complete-output
seam also exercises organic branch angle zero, whose source slow-angle clamp
has an inverted bound. Retain actual native output and expose unresolved
native semantics; do not claim range-minimum parity from component compilation.
Complete collision/avoidance integration, branch generation and installation
remain required adjacent slices, not replaced by settings or the existing guard.
Configuration aggregation uses all supplied printing-region options and
`PrintRegion.cpp::flow`'s non-first-layer external width, through
`perimeters/flow.rs::resolve_nominal_external_perimeter_flow`. Integration must
supply the actual object's printing-region set, nominal support flows and
normalized print resolution. Preserve the signed buildplate start-layer
narrowing and job-group two-pass zero-gap scan; a constructor boolean alone
does not implement that group lifecycle. Roof-only defaults and interface
priority consumers remain deferred to their algorithm slices.

The initial collision-cache component ports `TreeModelVolumes.cpp:835-865`
and `TreeModelVolumes.hpp::ceilRadius/getRadiusNextCeil` to
`tree/model_volumes/radius.rs`. Include base/zero radius, linear and exponential
buckets, truncating coordinates, ignored sorted buckets and XY-delta semantics.
The settings factory derives ignored buckets, but constructor queries start
with an empty ignored set until the actual precalculation lifecycle installs
them. Numerical tests validate only the radius component, not Tree's still-RED
public output contract.

The current collision slice ports the enabled single-outline constructor
subset and `calculateCollision` (`TreeModelVolumes.cpp:56-153,416-580`), lazy
`getCollision`/lower-bound queries and `RadiusLayerPolygonCache` storage
(`TreeModelVolumes.hpp:174-291`). Include raft-prefix strictly-simple outlines
with per-contour winding preserved (`Polygon.cpp:672`), rounded top/bottom
layer spans, tapered upper XY clearance, radius-zero model landing areas and
source-indexed blockers. Preserve non-overwriting insertion and the source
highest-layer query's layer-zero exception. The enabled single-outline source
last-mesh comparison remains false; do not add its inactive blocker/final
simplification path. No full constructor, movement limits, avoidance, hole-free
collision integration, cancellation callbacks, precalculation,
generator integration or output acceptance is claimed by this component.

The guide-volume query slice ports hole-free collision query/calculation
(`TreeModelVolumes.cpp:315-329,582-614`) and placeable query/calculation
(`:359-373,726-766`) into `model_volumes/holefree.rs` and `placeable.rs`.
Include the already-snapped radius contract, union before Round shrinking,
literal five-coordinate adjustment, source resolution/strict simplification,
incremental landing cache, empty enabled machine border at layer zero and
nonzero-radius shrinking by full XY separation. Preserve source zero-request
cache-miss returning collision, including its cache side effects. Defer batch
parallel scheduling, cancellation callbacks, avoidance, wall restrictions,
precalculation, generator consumers and complete-output acceptance; private
query compilation is not a new public GREEN seam.

The avoidance task slice rewrites `TreeModelVolumes.cpp:331-357,615-716`
under `model_volumes/avoidance.rs` and `avoidance/movement.rs`/`propagation.rs`.
Include source enum order Slow/FastSafe/Fast, six caches split by buildplate
and model destination, layer-zero collision return, full/minimum XY radius
identities and FastSafe-to-Fast large-radius dispatch. Individual tasks use
constructor move-minus-two limits, bottom-up serial propagation, source Round
substeps with collision union after every step, optional placeable subtraction
and winding-preserving strict simplification. Explicit `round_up_divide<int>`
in `Utils.hpp:394` narrows float operands before integer ceil division; retain
float last-step math and the scaled0.05 redistribution boundary. Release
zero-movement runs zero substeps, not a new error or manufactured motion.
`ClipperUtils.cpp:728::union_(Polygons,Polygons)` is the owner of the borrowed
two-subject NonZero union in `geometry/clipper/boolean_paths.rs`; avoid copying
the collision list into a temporary concatenation at every offset step.
Defer TBB batch scheduling/early-return block semantics, cancellation,
precalculation, wall restrictions, actual generator integration and complete
output acceptance. No claim that serial lazy task execution proves scheduling
or all-range equivalence; radius/zero-gap lifecycle and query order remain
integration obligations for the original public output seam.
The selected movement kernel currently covers ordinary finite i32 arithmetic
only. Float operands beyond i32 conversion range and ceil-sum signed overflow
have no proven native contract; Rust saturation/debug overflow must not be
claimed equivalent to C++ out-of-range conversion or signed-overflow behavior.
Real official-release inputs/boundary evidence are required before claiming
that domain, not a new guard or silently substituted arithmetic.

- Original KSR `gcode_header_widths::support_material_width_precedes_first_layer_width_when_enabled`
  passes without changing its input, fixture or assertions.
- Its complete G-code matches fresh Orca output, excluding only the generator
  line; actual support extrusion exists, not just corrected headers.
- Public full-output cases exercise Tree Auto/Manual and each source-dispatched
  style; assert effective settings and geometry activation, retain both streams,
  inputs/errors/manifests even on failure.
- Nonzero Normal angle, annotation volumes and multi-region modifiers receive
  real upstream output and their own public full-output E2E witnesses.
- Existing passing slicing projects remain passing; no UnsupportedProjectFeature
  guard or silent omission is accepted as the implementation.
- fmt, Clippy, nextest workspace no-fail-fast and Tier-1 builds/browser checks
  run on final code.

Native reference nondeterminism remains explicit; retain variants instead of
weakening the comparator.

## Wall restrictions slice

Port `TreeModelVolumes.cpp:375-401::getWallRestriction` and
`:769-833::calculateWallRestrictions` into private
`tree/model_volumes/wall.rs`. Include the radius-zero upper collision,
snapped-radius lower collision, distinct regular/minimum-XY caches, strict
simplification, and source recomputation of the highest cached layer without
overwriting existing entries. Radius quantization is independent of the
requested XY cache. Layer-zero release behavior returns collision directly.
Defer `precalculate` batch scheduling, cancellation, bed geometry, and the
Tree generator caller. The existing public complete-output RED remains the
acceptance test; this unconsumed component is not a GREEN or mergeable slice.

## Precalculation boundary

Port `TreeModelVolumes.cpp:158-253::precalculate` radius/layer simulation and
collision, holefree, placeable, avoidance and wall dependency order into
`model_volumes/precalculate.rs`. Install ignored tip radii only here, not in
the constructor. Snapshot all avoidance task starts before executing any
task, following `calculateAvoidance:615-653`. Use ordered Rust radius keys;
native unordered-map/TBB execution order and blocked-range early-return
behavior are deferred, not claimed equivalent. Timing logs, cancellation,
bed geometry and generator integration remain deferred. This boundary must
not be accepted as public output GREEN before real generation consumes it.

## Smart-overhang first-pass kernel

Port enabled `TreeSupport.cpp:795-910::detect_overhangs` into
`tree/overhangs.rs` and `tree/overhangs/first_pass.rs`. Include layer-wide
extrudable opening/intersection, manual/enforced-layer bounds, inclusive
threshold angle with89-degree cap, lower-height float narrowing, initial
sharp tails and cantilever classification. `MultiPoint.hpp:52-74` defines
the cantilever distance as nearest vertex, not nearest segment. Reuse the
existing source-owned ExPolygon overlap predicate, not Normal detection.
Pass an elapsed-seconds callback into the kernel and read it after raw
overhang calculation, preserving the source observation point and
30-second/100-polygon branch without native clock I/O in core. The actual
clock/cancellation/TBB coordinator and shared flag scheduling remain deferred.
Annotations/bridge filtering,
ordered type assignment and generator consumption are subsequent boundaries;
the first pass must never be exposed as complete support output. Existing
public ordered-output RED remains the agreed acceptance seam.

Extend the same kernel slice with `TreeSupport.cpp:917-983` sequential sharp
tail propagation in `tree/overhangs/propagation.rs`: supported intersection,
6mm bounding-box threshold, first overlapping lower tail's f32 height,
16mm accumulated cap, and both-axis5mm/eroded-new-overhang termination.
Use the existing source-owned empty-bounds and overlap geometry primitives.
The input width must be config `line_width` absolute against nozzle[0], not
auto Flow width. Fresh layer state is processed once per source phase;
coordinator lifecycle/cancellation, clusters and final output remain deferred.
The5mm growth comparison constructs `Point(double,double)` (`Point.hpp:197`),
which rounds before integer conversion; do not use new_scale truncation.

## Tree-local overhang clusters

Port the active `OverhangCluster` constructor/`push_back_if_intersects` and
first-match insertion from `TreeSupport.cpp:705-792`, plus grouping/filtering/
collection at985-1052, into private `tree/overhangs/clusters.rs`. Stable raw
polygon indices replace pointers; BTreeMap preserves one first member per
layer and ascending collection order. Include initial offset0, previous-layer
membership only, inclusive bbox rejection and source overlap predicate,
merged geometry updates even on duplicate-layer emplace, sharp-tail/cantilever
exemptions, erosion and either-axis small-size test. Keep raw and collected
layer overhangs separate, preserving cluster order rather than unioning output.
Unused height/intersects/debug methods need not be reproduced. Annotation and
bridge filtering, final type assignment, cancellation/coordinator and actual
generation remain deferred; this private stage is not public output parity.

## Tree bridge-contact filtering

Port `PrintObject.cpp:4306-4436::remove_bridges_from_contacts<ExPolygons>`
as called by `TreeSupport.cpp:1091-1094` into `tree/remove_bridges.rs`.
Include the `break_bridge=false` path: every region's perimeters, float
extrusion/bridge widths, supported endpoints, maximum bridge length with
literal10-coordinate margin, bottom-bridge angle != -1 and strict bbox-size
limits, unsupported-edge subtraction and final safety-offset difference.
Borrow real region-owned perimeter/surface data; never repeat aggregate data
for every region. The Normal SupportMaterialInternal function is not this
owner. Defer break_bridge=true (not used by this Tree call), caller flow/data
assembly, annotations, final types and generator integration to subsequent
source-owned slices. Existing public complete-output tracers remain the
acceptance seam; private helper tests/source-shape assertions are not added.

## Tree final overhang pass

Port TreeSupport.cpp1050-1125 into tree/overhangs/finalize.rs after cluster
collection. Include sharp-tail diff_clipped (ClipperUtils.cpp684-686 bbox
prefilter with SCALED_EPSILON), base-layer tails, float-height modulo rule,
blocker union before 0.2f-derived offset, auto-only critical regions, real
Tree bridge kernel before enforced contacts, 0.8mm enforcer expansion and
detected/enforced/sharp-tail ordering. Replace polygon-pointer type map with
parallel per-layer enum vectors and preserve highest-overhang state. Inputs
must contain actual layer slices, all region bridge views and complete
annotation projections; no empty placeholder annotations/region data at the
production boundary. Defer painted-facet/slab/vertical-point assembly,
cancellation/coordinator and generator consumption to following slices.
Existing public output tracers remain RED until those consumers are real.
## Non-Organic top-gap scalar rewrite slice

Port `TreeSupport.hpp::TreeSupport::top_z_distance` (float) and
`TreeSupport.cpp::TreeSupport` (658–659) to the existing non-Organic
`generation.rs` scalar initialization. Included: narrow configured top gap
to source f32 before its minimum-height clamp and subsequent contact,
height, radius and drawing consumers. The f64 Rust call-boundary scalar
is a compatibility carrier of this source-rounded value, not a different
numeric contract. Deferred: unrelated scalar conversions, Organic Tree,
remaining manual/Snug/Lightning behavior. Validate through the existing
exported public Hybrid 3MF complete-output comparison; do not alter its
fixture, comparator or extrusion precision. Re-run public Slim/Strong and
containment controls plus workspace regression.
## Concentric hole winding rewrite slice

Source boundary: `OrcaSlicer/src/libslic3r/ClipperUtils.cpp:992–1016`,
`traverse_pt_outside_in` / `union_pt_chained_outside_in`, consumed by
`FillConcentric.cpp::_fill_surface_single`. Rust destination:
`geometry/clipper/polytree.rs::PolyTree::outside_in`. Included: reverse each
hole contour unconditionally after ownership transfer, matching source
PolyTree winding. Remove the non-source open-chain area predicate/helper.
Deferred: M73 processor timing, unrelated Clipper traversal and broader
Tree/Normal gates. Existing public roof-base complete comparison stays
unchanged; an additional native-witnessed public extrusion-direction window
locks down this specific behavior without accessing private geometry.
## Normal Snug morphology rewrite slice

Source boundary: `SupportMaterial.cpp::SupportGridParams` (618–628),
`SupportGridPattern::extract_support` (845–858), `fill_contact_layer`
(1852–1865), and `SupportCommon.cpp::generate_interface_layers` (64–97,166).
Destination: support_material/pattern.rs for source style dispatch,
interface_grid.rs for density-specific interface morphology, and project.rs
for the actual public Normal Snug entry. Included: 2 mm Square/0 closing,
source float-scaled smoothing, no Grid trimming/sample filtering for Snug,
separate top/bottom interface radii, original-overhang top projection and
existing Snug no-reduction behavior.
Grid geometry remains the same source implementation. Deferred: unrelated
Tree/manual/Lightning and other SupportMaterial gaps, not the unchanged
public normal(auto)/snug complete-output acceptance requirement.

## Lightning unsupported-cell stable-sort slice

Owner: `Fill/Lightning/DistanceField.cpp::DistanceField`'s radius/hash
comparison and libstdc++ `bits/stl_algo.h::__stable_sort_adaptive`,
`__merge_sort_with_buffer`, `__chunk_insertion_sort`. Destination:
`geometry/clipper/ordering/stable.rs`; replace the width-one generic merge
scaffold, preserving the sole DistanceField consumer and caller comparator.
Included: normal half-sized buffer, seven-cell insertion runs, paired merge
passes and final stable half merge (left-buffer forward for equal halves,
right-buffer backward when the ceil-sized left half is longer).
Deferred: C++ allocation-failure paths,
remaining numeric/sampling/locator/propagation contracts. Acceptance seam is
the unchanged public Strong Lightning exported 3MF complete comparator;
debugger node/cell captures remain diagnostic evidence, not private tests.

## Lightning locator equal-key order slice

Owner: `Fill/Lightning/Layer.cpp::{fillLocator,generateNewTrees,
getBestGroundingLocation}`, `Layer.hpp::SparseNodeGrid`, fixed libstdc++
`hashtable.h::_M_insert_multi_node`. Destination:
`fill/lightning/layer.rs::SparseNodeGrid::equal_range`; retain append storage
and replace insertion-order exposure with newest-first equivalent-key
traversal. Included: no-hint insertion and first strictly better tied
grounding candidate. Deferred: numeric sampling precision, propagation and
conversion gaps. Unchanged Strong Lightning public 3MF full output is the
regression seam; debugger candidate traces are diagnosis only.

## Lightning realign visitor state-update slice

Owner: `Fill/Lightning/TreeNode.cpp::{lineSegmentPolygonsIntersection,
Node::realign}`. Destination: `fill/lightning/tree_geom.rs`'s intersection
helper, replacing the temporary nearest-current-ip scaffold with the
source's previous-stored-point distance/update sequence. Included: explicit
zero initial stored point from the selected reference, unchanged edge visit
order, strict improvement and radius gate. Deferred: other C++ build
initialization guarantees, adjacent numeric/reconnect/conversion gaps.
Regression seam remains the unchanged public Strong Lightning 3MF full
comparator; no geometry filter, private path assertion or fixture change.

## Lightning intersection actual-call compatibility slice

Owner: `TreeNode.cpp::lineSegmentPolygonsIntersection`'s call to
`Geometry.hpp::segment_segment_intersection`, `libslic3r.h::EPSILON` and
`Point.hpp::Point(Vec2d)`. Destination: `fill/lightning/tree_geom.rs`.
Replace the temporary mathematically corrected endpoint-difference call
and truncated return with supplied second endpoints as v1/v2 and rounded
Point construction. Included: source1e-4 denominator threshold, actual call
arguments and return conversion. Deferred: other sampling/reconnect/
conversion gaps. Preserve old-point state, all grid visits and original
public complete-output assertions; existing full RED is the regression seam.

Lightning sampling opening: port the implicit `jtMiter`/miter-limit3 call
from `Fill/Lightning/DistanceField.cpp::DistanceField`,
`ClipperUtils.hpp::DefaultJoinType/DefaultMiterLimit` and
`ClipperUtils.cpp::offset2_ex(ExPolygons)` to
`fill/lightning/distance_field.rs::DistanceField::new`. Included: both-stage
Miter join selection and removal of incorrect Square-default documentation.
Deferred: general rasterizer/offset cleanup contracts beyond this call.
Verify using the unchanged public complete-output Strong Lightning RED;
no manual sample point or relaxed comparison.

Organic worker-domain comparison: `TreeSupport3D.cpp::merge_influence_areas`
uses `tbb::this_task_arena::max_concurrency()` to select geometry-affecting
bucket sizes. Destination is `tree/organic/merge_areas.rs`, with the worker
input from `generation.rs::workers`. Included: reproducible single-CPU validation
must apply the same CPU domain to both test process and immutable Orca;
keep production worker derivation unchanged. Deferred: default multiworker
reproducibility and all-platform runtime coverage. Keep original complete
output assertions, default-domain artifacts and all existing controls.

Painted support facet slice: upstream bbs_3mf.cpp `paint_supports`,
Model.cpp::FacetsAnnotation string import/strict extraction,
TriangleSelector.cpp serialized split decoding and strict selected geometry,
PrintObject.cpp::project_and_append_custom_facets and
TriangleMeshSlicer.cpp::slice_mesh_slabs. Destinations: project/model_xml.rs,
project/facets_annotation.rs and project_slice/support_material/facet_projection.rs
(split functional children below400LOC), replacing the no-annotation mesh
import shell and connecting real enforcer/blocker/vertical input to Tree.
Included: leaf/split encoding, strict support states, transforms/slab polygons
and vertical enforcer points. Deferred: brush UI, seam/MMU output consumers,
not shared support-required decoding. First tracer: new annotated overhang
3MF/tree(manual)/Slim through the existing exported3MF and complete-output
seam; original fixtures/assertions untouched. Finite prerequisite acceptance
does not close manual painting or allow the held Normal/Tree batch to merge.

Vertical Manual contact tracer: follow TreeSupport.cpp3460-3474/3595-3602
and Layer.hpp335::zs_from_layers for centroid binning by slice_z; retain
source scaled skin direction and prior-layer sharp-tail contact placement.
Destination: support_material/tree/non_organic/contacts.rs, with existing
facet_projection transform and mesh_slicer/slabs centroid owners. Include
public right vertical leaf-face ENFORCER input with actual native support
activation and unchanged complete-output assertion; defer other vertical
geometry/transform/style combinations only to subsequent required tracers,
not a full Manual completion claim. Do not replace real facets by volumes.
