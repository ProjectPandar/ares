# Spec: Task 22O.159 zero-initialized monotonic link limit

## Observable contract

Normal project monotonic fill preserves valid contour links without imposing a perimeter-link length cap. The KSR first bottom-surface extrusion therefore continues from `G1 X139.407 Y83.135 E.02413` through the source contour at `G1 X145.616 Y89.344 E.32744` instead of extending to the outer offset and retracting.

The behavior is derived from generated surface geometry and project fill parameters. Production code does not inspect fixture names, expected G-code, coordinates, or digests.

## Upstream boundary

This slice ports the executed initialization order in OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillBase.hpp:181-194` and `src/libslic3r/Fill/Fill.cpp:1234-1277,1334-1356`: a new `Fill` starts with `spacing == 0`, computes `link_max_length` before per-expolygon spacing is assigned, and therefore passes a zero link limit into `src/libslic3r/Fill/FillRectilinear.cpp:1051-1052,1182-1193`. A zero limit skips length-based `TooLong` classification while retaining contour validity checks.

Ironing's separately assigned link limit, extrusion formatting, cooling, timing, and later exact G-code differences are deferred.
