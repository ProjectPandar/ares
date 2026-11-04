# Spec: classic extra perimeters over overhangs

## Source and destination

OrcaSlicer2.4.2 revision8500fcdccaa10b5099ac20d252af3a7c560046f1:
`PerimeterGenerator.cpp:677-1109` owns `paths_touch`, `reconnect_polylines`,
`sort_extra_perimeters`, `generate_extra_perimeters_over_overhangs`, and
`PerimeterGenerator::apply_extra_perimeters`; classic call site:1672.
Rust destination: new modules under `project_slice/perimeters/classic/
extra_perimeters`, called at the existing infill-boundary surface stage.
Reuse the cached half-nozzle-expanded lower polygons (cpp:1177, hpp:151).
`Polyline.cpp:383-405` owns shared-endpoint append semantics. The existing
`Geometry/ConvexHull.cpp:11-43` Rust port moves unchanged from private skirt
geometry into `geometry/convex_hull.rs` for reuse, with all old consumers updated.
Current Ares inactive-guard scaffolding is a temporary compatibility shell;
replace it with the cited generation/insertion/fill-subtraction behavior, not
an independently designed perimeter pipeline and not a guard-only deletion.

## Included

Classic activation conditions, bbox-clipped lower slices, anchor/overhang regions,
convex-hull bridgeability test, recursive offsets and continuation loops,
medial-axis gap paths, reconnection, anchored-first dependency ordering and length
filtering; prepend generated OverhangPerimeter paths to the matching island
collection; subtract filled area from accumulated fill surfaces while preserving
fill_no_overlap semantics. Preserve upstream truncation, ordering and expression
semantics even when surprising (including the boolean min-distance assignment).

Bridge scope: port only
`BridgeDetector.hpp:122-126` polygons→floating-edges adapter locally inside the
perimeter module. Reuse existing `geometry::detect_bridging_direction`; do not
change it. If its shared algorithm needs correction, that is a separate bridge
slice. Existing clipping, offsets, hull, medial-axis and AABB
helpers are reused; do not duplicate these core algorithms.

## Deferred

Arachne extra-perimeter integration, other PerimeterGenerator features,
shared bridge algorithms, counterbore bridging, additional vendor causes and
lint cleanup. No generic guard removal before implementation. No new fallback,
unsupported branch masquerading as success, printer-specific special case,
comparator relaxation or skipped failing case.

## TDD and acceptance

Baselinebd49f5fd: frozen Anycubic Kobra3 Max0.6 case-SzNoeB and0.8 case-7ivcdp
are classic and both fail at the existing guard. Add original-Orca active overhang step fixtures
before implementation to exercise actual generated paths, not only tiny stock
geometric overhangs. Both enabled and disabled controls must be retained.

Require strict full ordered-byte equality, original Orca2.4.2 single-CPU reference,
stock target2 and known68+823 controls, workspace Nextest/fmt/default Clippy/
actual core+adapter WASM build and a clean probe-free diff.
