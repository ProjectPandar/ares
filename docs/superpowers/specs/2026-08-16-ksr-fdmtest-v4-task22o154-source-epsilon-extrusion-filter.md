# Spec: Task 22O.154 source-epsilon extrusion filtering

## Observable contract

A fitted perimeter whose first retained linear segment is shorter than OrcaSlicer's geometric epsilon must not emit a zero-extrusion command. In the KSR slice, travel to `X136.839 Y100.592` is followed by the real extrusion to `X133.903`; `G1 X136.839 Y100.592 E0` is absent.

Filtering applies to fixed-speed lines, fitted arcs, and variable-speed segments. Skipped variable-speed points remain in the retained wipe polyline, while the next emitted length starts at the prior emitted G-code point.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/libslic3r.h:51-53` and `src/libslic3r/GCode.cpp:6992-7003,7049-7085,7133-7155`: line and arc moves shorter than `EPSILON = 1e-4` mm are skipped before extrusion output.

Cooling, timing, object identifiers, wipe allocation, and later exact G-code differences are deferred.
