# Task 22O.12: ThickPolyline medial-axis prerequisite

## Fixed rewrite boundary

This task rewrites OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`: `src/libslic3r/Line.hpp:15-19,202-212`, `Polyline.hpp:14-17,256-287`, and `Polyline.cpp:637-679`. `PerimeterGenerator.cpp:1586`, `ExPolygon.cpp:261-369`, and `Geometry/MedialAxis.hpp/.cpp` are consumer citations only: they establish the required medial-axis output model but are not executed by O12.

The Rust destination is crate-private `ares-core::geometry::{line,polyline}`. Rust flattens the C++ `Polyline` inheritance into `ThickPolyline { points, width, endpoints }` without adding an inheritance compatibility shell.

## Included behavior

O12 ports safe reached `ThickLine` construction, `ThickPolyline` default/reverse/clear, ordered thick-line projection, closed `start_at_index` rotation, and `to_thick_polylines`. It preserves the source invariant of two ordered widths per segment, complete width reversal, endpoint-flag swapping, and `clear` leaving endpoint flags unchanged. Internal source preconditions remain trusted; no repair or fallback is added.

The C++ zero-argument `ThickLine` leaves inherited endpoints unspecified and is neither reached nor represented with unsafe Rust.

## Deferred behavior

`ExPolygon::medial_axis`, Boost.Polygon-compatible Voronoi construction/topology, `MedialAxis::build`, edge validation and chaining, endpoint extension, filtering/reconnection, tiny-gap filtering, `variable_width`, and downstream gap extrusion remain deferred. A partial MedialAxis shell would fabricate a seam because its meaningful methods depend directly on Voronoi cells, edges, twins, and rotations.

O12 is a source-owned prerequisite, not a new Ares pipeline stage. It does not alter O11, the KSR checksum, lifecycle wiring, or the public `ProjectSlicingIncomplete` result. `PerimeterGenerator.cpp:1586` remains the next unexecuted production line.

## Verification

Literal tests pin constructor widths, default flags, reverse/clear order, segment-to-width pairing, closed rotation, conversion order, and exact cardinality. O11 regression, strict Clippy/check, Tier-1 WASM, rustfmt, and workspace Nextest remain required. No dependency, unsafe code, filesystem/runtime oracle, fixture identity branch, or source-text/hash assertion is allowed.
