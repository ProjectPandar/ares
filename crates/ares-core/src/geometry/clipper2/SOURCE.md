# Clipper2 source authority and reconciliation

Work in progress on a scratch branch; not connected as the geometry backend and
not a parity claim. The public beta RED commit must precede this implementation
in submitted history.

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
| `clipper.engine.cpp:637–742,862–866,925–932`, input/minima | `engine/find_and_add_local_minima_closed.rs`, `engine/horzjoin.rs` | Port the original linked input walk and minima registration order, including flat open paths, closing duplicates and duplicate-minimum suppression; reserve the owning vertex arena directly instead of cloning paths/index lists. |
| `clipper.engine.cpp:2139–2184,2373–2380,2416–2499,2734–2768`, scanbeam/intersections | `engine/add_local_max_poly.rs`, `engine/do_horizontal.rs`, `engine/do_intersections.rs`, `engine/execute_internal.rs` | Replace seed-specific early exits/current-X cache flags with the pinned control flow. Preserve the original intersection sort comparator and the existing audited GNU std::sort tie behavior; minima/horizontal stable sorts remain stable. |
| `clipper.engine.h/.cpp`, Orca node allocator | indexed Rust arenas | Use Rust-owned Vec storage, not native TBB allocation; no pointer-order dependency or native allocator ABI. |
| dependency namespace separation and `clipper2_z.hpp/.cpp` | shared Rust XY/Z engine | Z payload is compiled in, XY equality still ignores Z; no Cargo engine-selection feature or fallback. |

ClipperD, its floating-coordinate PolyTree and its Z callback are excluded from
this port boundary. PointD remains only as numeric support for engine/offset
calculations; integer path coordinates are not routed through ClipperD scaling.

The wrapper implementation belongs to `src/libslic3r/ClipperUtils.cpp/.hpp` and
`ClipperZUtils.cpp:48–80`. It is not supplied by the seed. Ordinary booleans,
stray-hole ownership, shortest-edge/arc policy and coincident-Z callback suppression
must be ported and publicly verified before this becomes the sole backend.
