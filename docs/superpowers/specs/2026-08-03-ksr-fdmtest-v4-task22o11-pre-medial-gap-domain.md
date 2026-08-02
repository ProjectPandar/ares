# Task 22O.11: pre-medial Classic gap domain

## Fixed rewrite boundary

This task rewrites OrcaSlicer commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`, `src/libslic3r/PerimeterGenerator.cpp:1573-1581,1583-1585`. It stops immediately before executable line 1586, `ex.medial_axis(min, max, &polylines)`. Line 1582's `ThickPolylines polylines` declaration is omitted because its only result belongs to that deferred operation.

The supporting source seams are `ClipperUtils.hpp:17,19,27,392,423-424,449`, `ClipperUtils.cpp:582-585,755-756`, `ExPolygon.cpp:67-72`, `Polygon.cpp:99-105`, `MultiPoint.cpp:164-230`, and `Line.hpp:43-76`. The Rust destination is `ares-core::project_slice::perimeters::classic::gap_domain`, using the internal geometry rewrite.

## Included behavior

For each aligned O3 surface, empty `gaps` becomes typed `None`. A nonempty input computes the source-exact `f64` min/max bounds and `f32` offset casts, performs `opening_ex`, the second `offset2_ex`, ordinary `difference_ex` without a safety boolean, and in-place contour-then-holes Douglas–Peucker simplification. Geometry is staged transactionally before O10 ownership moves. Geometry range errors map to one stable `InvalidInput` message.

O11 moves O10 source indices, inactive provenance, and complete appended collections without rebuilding allocations. It retains the identical boxed O5 traversal predecessor. Error cleanup consumes O10 and both O5 tree families iteratively, including on a constrained stack. Public slicing reaches O11 and remains `ProjectSlicingIncomplete`.

## Deferred behavior

`ThickPolyline(s)`, `medial_axis`, tiny-fill filtering, `variable_width`, covered-width subtraction, gap-fill append, line 1627 onward, active thin walls/seams, and final G-code parity remain deferred. O11 is an upstream prefix, not an Ares-designed pipeline.

## Verification

Direct tests cover the empty/nonempty gate, arithmetic/casts, exact geometry and simplification; lifecycle tests cover predecessor and nested allocation identity; in-memory KSR tests pin domain structure checksum `-56719811695275585622325662811286152552` and unchanged O10 collection checksums. Production tests contain no source-text/hash assertion or runtime oracle.
