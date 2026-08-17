# Spec: Task 22O.146 aligned collection seam target

## Observable contract

For the KSR project, the first inter-path retract keeps the source wipe sequence and then emits `G1 X145.539 Y95.848 Z.6 F60000`. The target, lift height, and feedrate come from generated perimeter geometry, aligned-seam selection, and loaded travel/retraction options; production code must not inspect fixture names or reference G-code.

## Upstream boundary

This slice ports the relevant behavior from OrcaSlicer 2.4.2 `src/libslic3r/GCode/SeamPlacer.cpp:1107-1166,1370-1393`, `src/libslic3r/KDTreeIndirect.hpp:280-324`, and `src/libslic3r/GCodeWriter.cpp:685-780`.

Included behavior:

- exact source-style KD median partitioning and radius traversal for nearby seam candidates;
- layer embedding context only when the source print layer contains multiple perimeter-bearing regions;
- locality-preserving selection of a not-much-worse aligned seam among sibling perimeters in one extrusion collection;
- the deferred spiral-lift Z coordinate on the travel that consumes that lift.

Deferred behavior: exact fitted-arc grouping after the travel, cooling, timing, and all later byte differences.

## Acceptance

A focused `slice_project` test asserts the exact wipe moves, spiral lift, and following XYZ travel. The prior next-layer lifted-travel contract remains green, the changed core crate passes Clippy, and every changed Rust source remains below 400 lines.
