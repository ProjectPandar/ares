# Adaptive cubic noncrossing multiline paths

## Upstream boundary

Source revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/Fill/FillAdaptive.cpp:1287–1466`: `noncrossing::{SweepLine,Junction,LevelPath}`, `clip_profile`, `path_points`, `polyline_length`, and `point_along`.
- `FillAdaptive.cpp:1468–1842`: `multiline_paths`, including sweep-frame clipping, collinear merging, junctions, short dead arms, bend pairing, cuts, neighbouring-wall pushes, dependency-ordered endpoint trimming, and world-frame rounding.
- `FillAdaptive.cpp:1897–1905`: `Filler::_fill_surface_single` selects the sweep from `layer_id / max(thickness_layers,1)`, expands the component bounds, and uses nonempty routed paths before `multiline_fill` and polygon intersection.
- `Geometry.hpp:219–288`: floating-point Liang–Barsky clipping.
- `AABBTreeLines.hpp:227–242,361–373` and `AABBTreeIndirect.hpp:639–667`: indexed floating-line radius queries, strict squared-distance predicates, and line access for trimming.

This extends [per-region octree selection](adaptive-region-octree-selection.md) at the existing filler boundary. Region admission, spacing arithmetic and region-ID selection remain unchanged.

## Rust destination

All implementation remains private to `ares-core`:

- `fill/adaptive/multiline.rs` coordinates the source algorithm. Its `math`, `sweep`, `junctions`, `paths`, `cuts`, `profile`, and `trim` submodules retain the upstream numeric operations and intermediate structures, with each source module below400 lines.
- `fill/adaptive/filler.rs` calls the port for multiline output before the existing multiline offset/crop/connect tail. A private `SurfaceGeometry` carries Z, layer ID and thickness-layer count. Raw segments use the existing geometry `Line` representation.
- `project_slice/fill_entities/adaptive.rs::{append,anchoring_lines}` forwards the current layer ID and representative surface thickness. `fill_entities/generate.rs` and `prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs` only supply the existing layer identity. Neither path changes grouping or region membership.
- `geometry/line_distance_tree/floating.rs` and its query module expose the missing private radius query and indexed line access over the existing floating AABB tree. Tree ownership/building and other queries are unchanged. Existing fixed-GCC sorting retains the upstream equal-key ordering.

The old segment-to-two-point-polyline shortcut is replaced at the multiline call boundary, not kept as an alternative implementation. Single-line output and the source's empty-result semantics remain as specified by `Filler::_fill_surface_single`; there is no fixture dispatch or independent geometry route. Existing Ares data types are compatibility shells around the cited upstream structures, not a new slicing pipeline.

## Included semantics

1. Clip in the expanded component box before sweep rotation; retain double precision, family admission tolerance, collinear averaging and sort order.
2. Preserve junction detection order and three dead-arm passes. Pair crossing arms bottom-up and trace left-to-right paths with terminal junctions.
3. Apply bend cuts, room limits, neighbouring pushes and profile windows with the source tolerances and operation order.
4. Build the existing floating AABB index once for each routing call. Preserve dependency traversal, three trimming passes, step/bisection limits, endpoint regrowth, minimum path length and final integer rounding.
5. Select the sweep using actual layer identity and representative thickness in both emission and sparse anchoring. Do not substitute layer-vector position or infer identity from Z.
6. The shared `Filler` follows the same source call for either selected octree kind. This does not change SupportCubic spacing/preparation or claim full SupportCubic parity; it introduces no kind-specific alternate filler.

## Deferred behavior

Octree mesh construction, separated bodies, belt transforms, SupportCubic per-region preparation, cache structure, grouping/order/membership, extrusion materialization, estimators, headers, formatting and unrelated connection algorithms remain outside this boundary. The existing octree generation and multiline offset/crop/connect tail are consumed without redesign.

## Tests

Use the existing `crates/ares-cli/tests/orca_parity/adaptive_octree_spacing.rs` cases without changing fixtures, assertions or comparator. The multilined region changes line multiplicity while retaining the established complete grouped-fill handoff and per-region spacing premise. Its remaining output mismatch exercises the omitted routing stage. Density and both controls protect the preceding spacing slice and single-line behavior.

Compare complete public Native/API/CLI output in both configured runtime modes, normalizing only the generator line. Retain configuration, header, timing and progress commands. Existing shared-filler controls check that forwarding and the common call do not change unrelated preparation semantics. No stored coordinate/count expectations or reduced-output comparator is introduced.

## Failure modes

1. A layer-vector index or missing thickness changes sweep selection despite identical raw segments.
2. Integer clipping, altered rotation order, stable sorting or different collinear averaging changes family membership and junction order.
3. Missing terminal arms or shortened-line junction rebuilding leaves incorrect paths through nearby crossings.
4. Reordered cuts/pushes or broader profile windows changes sharp triangle tips and neighbouring-wall clearance.
5. Integer-rounded AABB primitives, inclusive radius checks or a linear scan changes trimming semantics or complexity.
6. A different dependency traversal, omitted regrowth pass or premature minimum-length removal changes stopping endpoints.
7. Truncation instead of world-coordinate rounding changes final lattice points.
8. Retaining the old multiline shortcut, routing by fixture/kind, or repairing grouping/formatting in this module would replace the cited source boundary with different behavior.
