# Clipper2 source authority and reconciliation

Integration work in progress: the geometry adapters now select this engine only;
Clipper1 state/offset/round-policy machinery is removed. The approved public beta
RED precedes the engine import in submitted history. This is not a parity,
platform, lint or completed source-audit claim.

Authority: OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`deps_src/clipper2/Clipper2Lib` (Clipper2 2.0.1; dependency tree
`09f1baceae892df95ea07250dbb6b6d3dedd3d2b`). Seed:
`larsbrubaker/clipper2-rust` commit
`2f5e9d4249bc4d624773094ff6164335d9ff6ae7` (declares 1.5.4).
Original upstream 1.5.4 source tag resolves to
`ef88ee97c0e759792e43a2b2d8072def6c9244e8`.
Retain the original Angus Johnson copyright and the accompanying
[Boost Software License](LICENSE-Boost-1.0); the Rust seed was developed by
MatterHackers. The seed is reference code, not an independent geometry authority.

## Reconciliation ledger

Paths below are relative to the pinned dependency's `Clipper2Lib` directory.
This ledger records source work, not test acceptance. The complete engine and
wrapper audit remains open.

| Upstream hunk/function | Rust destination | Reconciliation |
|---|---|---|
| `include/clipper2/clipper.core.h:127–134,176–183`, floating `Point64` construction; `src/clipper.offset.cpp:262–274::DoMiter` | `core/from.rs::Point64::from`, `offset/do_square.rs::do_miter` | Round the completed coordinate expression with `f64::round` before the integer cast, retaining symmetric half-away-from-zero behavior at negative and positive half ties. Do not reuse Clipper1 `floor(x + 0.5)` or the intersection function's truncation. |
| `include/clipper2/clipper.core.h:730–801`, products/sign | `core/from.rs`, `core/transform_paths.rs` | Exact Rust i128 products on all architectures; remove pointer-width dispatch. |
| `clipper.core.h:803–846`, vector arithmetic/distance | `core/from.rs`, `core/get_bounds_paths.rs` | Correct seed's premature floating conversions to the original subtraction/product cast positions. |
| `clipper.core.h:849–869`, `Area` | `core/get_bounds_paths.rs` | Replace seed's alternate shoelace accumulation with the original paired accumulation. |
| `clipper.core.h:956–979`, renamed `GetLineIntersectPt` | `core/get_bounds_paths.rs`, engine/offset call sites | Rename both coordinate variants; integer differences precede f64 conversion, interior casts truncate, computed Z is zero. |
| `clipper.core.h:1008–1050`, `SegmentsIntersect` | `core/transform_paths.rs` | Seed already contains the beta parametric inclusive/exclusive branch; retained for source audit. |
| `clipper.core.h:1052–1072`, `GetClosestPointOnSegment` | `core/transform_paths.rs` | Cast the displacement before coordinate addition, retain integer subtraction and nearbyint sites. |
| `clipper.core.h:1136,1150`, `PointInPolygon` | `core/is_collinear.rs` | Seed already uses exact cross-product signs in both changed branches. |
| `src/clipper.engine.cpp:2294–2303`, `MoveSplits` | `engine_fns/path2_contains_path1_outpt.rs` | Retain the seed's destination exclusion and preserve the source record's cleared, allocated split-list state. |
| `clipper.engine.cpp:2967–3018`, `CheckSplitOwner`/`RecursiveCheckOwners` | `engine/fix_self_intersects.rs` | Iterate the live split vector by owner/index, including recursive growth; do not clone its membership or silently reparent to the root. |
| `clipper.engine.cpp:2917–2954,3020–3079`, output walks | `engine_fns/path2_contains_path1_outpt.rs`, `engine_public/is_hole.rs` | Preserve original starts, reverse traversal, tiny-triangle admission and optional-open-output branch. |
| `src/clipper.offset.cpp:301–303`, `DoRound` | `offset/do_square.rs` | Port Orca's rounded arc count, minimum one, replacing seed's ceiling count. |
| `clipper.offset.cpp:378–404`, concave crossing/`OffsetPolygon` | `offset/do_square.rs` | Port the edge-length bound, join point and callback/duplicate guards. |
| `clipper.offset.cpp:514–567`, `DoGroupOffset` | `offset/do_group_offset.rs` | Walk owned paths without seed-only per-path cloning; retain the single-point callback and its original int-sized square extent conversion. |
| `clipper.engine.cpp:637–742,862–866,925–932`, input/minima | `engine/input_paths.rs`, `engine/horzjoin.rs` | Port the original linked input walk and minima registration order, including flat open paths, closing duplicates and duplicate-minimum suppression; reserve the owning vertex arena directly instead of cloning paths/index lists. |
| `clipper.engine.cpp:2139–2184,2373–2380,2416–2499,2734–2768`, scanbeam/intersections | `engine/add_local_max_poly.rs`, `engine/do_horizontal.rs`, `engine/intersections.rs`, `engine/do_intersections.rs`, `engine/execute_internal.rs` | Replace seed-specific early exits/current-X cache flags with the pinned control flow. Preserve the original intersection sort comparator and the existing audited GNU std::sort tie behavior; minima/horizontal stable sorts remain stable. |
| `clipper.engine.cpp:1551–1586,1646–1706`, collinear/self-intersection repair | `engine/convert_horz_segs_to_joins.rs`, `engine/fix_self_intersects.rs` | Explicit advance/restart/disposed outcomes preserve the source loop exits, point removal, callback and split registration order. |
| `clipper.engine.cpp:1811–1903`, open `IntersectEdges` | `engine/intersect_open_edges.rs` | Match the pinned local-minimum vertex condition; remove the seed-only extra `point == edge.bot` restriction. Preserve closed-edge splitting, toggles, matching-minimum joining and callback order. |
| `clipper.engine.cpp:1911–2123`, closed `IntersectEdges` | `engine/intersect_edges.rs` | Split winding updates, fill projections and hot/cold output branches without changing predicates, output/callback order or record swaps. |
| `clipper.engine.cpp:2193–2370`, horizontal segment joins | `engine/horz_segments.rs`, `engine/convert_horz_segs_to_joins.rs`, `engine/horz_joins.rs` | Preserve stable segment sorting, live linked-point advancement, splice order, containment precedence and ownership/split updates. |
| `clipper.engine.cpp:2382–2472,2563–2732`, scanbeam merge/horizontal walk | `engine/intersections.rs`, `engine/do_horizontal.rs` | Extract native loop segments without new sort keys or endpoint inequalities; preserve adjacent-run crossing order, SEL base updates, maxima priority and open/closed deletion order. |
| `clipper.offset.cpp:599–660`, constant error-code state | `offset/set_arc_tolerance.rs`, `offset/floating_point_tolerance.rs` | Remove the unused error-code API/member and constant-false clearing branch: the pinned source and port never assign it a nonzero value. |
| `clipper.engine.h/.cpp`, Orca node allocator | indexed Rust arenas | Use Rust-owned Vec storage, not native TBB allocation; no pointer-order dependency or native allocator ABI. |
| dependency namespace separation and `clipper2_z.hpp/.cpp` | shared Rust XY/Z engine | Z payload is compiled in, XY equality still ignores Z; no Cargo engine-selection feature or fallback. |

ClipperD, its floating-coordinate PolyTree and its Z callback are excluded from
this port boundary. PointD remains only as numeric support for engine/offset
calculations; integer path coordinates are not routed through ClipperD scaling.

The application wrapper authority is `src/libslic3r/ClipperUtils.cpp/.hpp` and
`ClipperZUtils.cpp:48–80`, not the seed. Integration correspondence:

| Upstream owner | Rust adapter | Included behavior |
| --- | --- | --- |
| `ClipperUtils.cpp:235–264,419–598` | `clipper/offset.rs`, `clipper/offset/{execute,expolygon}.rs` | Strict shortest-edge filtering, seam trim, float delta, arc/miter setup, bbox grouping, isolated-CW sign, declared ExPolygon roles, direct tree output and fallback-union fill rule. |
| `ClipperUtils.cpp:283–390,935–971` | `clipper/polytree.rs`, `clipper/polytree/stray_holes.rs` | Signed-area roles, recursion through misplaced nodes, smallest strictly-larger enclosing contour, rounded-boundary test and interior-band fallback; unattached holes retain their winding. |
| `AABBTreeIndirect.hpp:129–242` | `clipper/polytree/stray_holes/bbox_index.rs` | Source inflated bounds, integer centroid, axis ties, median-of-three partition and left-before-right candidate traversal, including equal-area ordering. |
| `ClipperUtils.cpp:393–416,1011–1118,1220–1287` | `clipper/{backend,boolean_ex,boolean_paths,polyline}.rs`, `clipper/polytree/outside_in.rs` | NonZero booleans, caller fill overrides, collinear flag, open outputs and recombination, direct PolyTree conversion and source outside-in chaining. |
| `ClipperUtils.cpp:1303–1315,1354–1570`, `Polygon.cpp:685–705`, `ExPolygon.cpp:231–267`, simplification and variable offsets | `clipper/{variable_offset,simplify}.rs`, `geometry/simplification.rs` | Preserve the higher-level mitered path construction; replace negative-outer repair with Positive/Negative C2 unions and direct Difference tree output. Do not emulate StrictlySimple. |
| `ClipperZUtils.cpp:48–80`, `.hpp::ZPoint/ClipperZIntersectionVisitor`, `PerimeterGenerator.cpp:321–385::clip_extrusion` | `clipper/{z,z_clip}.rs`, `geometry/line_split.rs`, `geometry/region_expansion/wave_seeds.rs` | Subject-priority inherited Z, endpoint callback suppression, Z-only copyback, closed-before-open output, source-index tags and intersection-pair registration. Width interpolation retains the source's three-component squared-norm ratio and subject-projection repair; it does not invent a zero-length fallback for trusted input segments. The adapter's Eigen-style ZPoint equality includes Z; only engine Point64 equality ignores it. |
| `Polygon.cpp:65–94,738–789` | `geometry/polygon.rs`, `clipper/point_in_polygon.rs` | Polygon winding uses Slic3r signed area, not a removed Clipper1 area helper. Retain Slic3r's integer ray determinant and border-result convention, separately from C2's wide point-in-polygon predicate. |
| `Algorithm/RegionExpansion.cpp::wavefront_initial/wavefront_step/wavefront_clip`, `Support/SupportCommon.cpp::shrink_expolygon_with_contour_idx` | `geometry/region_expansion/propagate.rs`, `clipper/z_clip.rs` | Replace mutable Clipper1 offset options and orientation repairs with the matching beta overloads; preserve callers' higher-level traversal/recovery logic. |
| `Fill/FillBase.cpp:2947–3004::multiline_fill` | `fill/multiline_offset.rs` | Replace the separate round/Clipper2Truncate emulation with the sole wrapper's float delta, miter joins and round cap tolerance; offset-list/spacing logic stays at the caller. |

The unwitnessed, unfixed NaN miter-limit edge is `std::max(NaN, 2.0)` retaining NaN at `ClipperUtils.cpp:425` versus `f64::max` selecting `2.0` at `clipper/offset.rs:104`.

Unused seed exception/decimal-scaling conveniences, unused rectangle/point
queries, seed-only output builders, and unused PolyTree parent-query APIs are
excluded rather than warning-suppressed. Integer engine ownership records and
the child traversal used by adapters remain. Structural extraction must obey the
repository nesting limits without exemptions; the architecture strategy records
upstream segment-to-helper correspondence. Tree attachment/outside-in adapters
require explicit CoordinateScale; documented no-scale facade chains pass Normal
constants as temporary normal-only shells, not large-printer parity or an
implicit mode. See the architecture strategy for exact deferred call sites. The complete function-by-function
reconciliation, lint cleanup and strict public verification remain open.
