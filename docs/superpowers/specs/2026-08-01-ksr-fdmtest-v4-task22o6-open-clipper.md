# Task 22O.6: Exact Open-Path Clipper Infrastructure

## Decision and source boundary

Task 22O.6 is a source-cited Rust rewrite slice pinned to OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Its only Rust production destination is `crates/ares-core/src/geometry` and, specifically for clipping, `geometry::clipper`.

Included upstream behavior:

- `src/libslic3r/ClipperUtils.cpp:835-934`: `_clipper_pl_open`, `_clipper_pl_recombine`, `_clipper_pl_closed`, and only the polygon overloads of `diff_pl` and `intersection_pl` needed by the next perimeter slice.
- `deps_src/clipper/clipper.cpp:756-949`: open `AddPath`/`AddPathInternal`, including two-point acceptance, duplicate and collinear handling, flat paths, minima, bounds, skipped terminal edge, zero winding, and LML construction.
- `deps_src/clipper/clipper.cpp` execution/open-output branches around `1137-1152`, `1992-2017`, `2218+`, `2800-2860`, all output-affecting open branches in contribution/intersection/horizontal/maxima/join/fixup/orientation/strict-simple handling, and PolyTree building/extraction around `4119-4179`.
- Matching declarations and `IsOpen`/`m_HasOpenPaths` semantics in `deps_src/clipper/clipper.hpp`.
- The source `Polyline` geometry concept and `Polygon::split_at_first_point` exact closure duplicate needed to pass polygon contours through the open engine.

The existing closed-only `ClosedClipper` becomes `Clipper` with no compatibility alias. Existing closed clipping semantics, order, topology, and active full-range `double`/Rust `f64` determinants remain unchanged. Open paths are subjects only; flat output is rejected whenever open input has been added; PolyTree records represent open paths as open paths rather than polygons.

## Required API and behavior

`geometry::Polyline` owns an ordered point vector, is valid at two or more points, and does not normalize, deduplicate, or implicitly close. `Polygon::split_at_first_point` preserves every vertex and appends the first point once.

`Clipper` exposes open subject ingestion and PolyTree execution. Every output-affecting upstream `IsOpen` branch is ported, including zero-winding edge behavior, open record ownership, intersection and horizontal transitions, maxima, open fixup, and exclusions from polygon-only joins, hole ownership, orientation, and strict-simple repair. Open PolyTree outputs are source-order root records and have typed polyline access/extraction. Closed topology and `into_expolygons` remain exact and never reinterpret open records.

The source-scoped polygon `intersection_pl` and `diff_pl` wrappers append the first point to each closed subject polygon, execute with NonZero fills through PolyTree, extract open paths, and recombine fragments using the exact nested `i`/`j` loop, four-branch priority, erase, and retry order from `ClipperUtils.cpp`.

## Verification

Direct semantic fixtures assert exact point order, output-record order, and orientation for input rejection, flat-output rejection, closure duplication, all recombination branches and repeated priority, wholly inside/outside paths, multiple crossings, endpoint/tangent/coincident-boundary behavior, horizontals, repeats/collinear points, large valid coordinates sensitive to the `f64` determinant, and mixed closed/open PolyTree roots. Existing closed Clipper tests remain required.

## Explicit deferrals

This milestone does not change the public slicing lifecycle. Task 22O.5 remains the terminal `ProjectSlicingIncomplete` stage. `PerimeterGenerator.cpp:153-228` extrusion/materialization is reserved for O7, including extrusion entity types and numeric metadata, bbox preparation at that call site, shortest-path ordering/reversal, loop construction, and traversal output. Also deferred are thin walls, recursive child emission, fill/gaps, seams, infill, motion, writer/post-processing, G-code, and complete Task 22O or KSR parity. O6 does not emit placeholder traversal results.
