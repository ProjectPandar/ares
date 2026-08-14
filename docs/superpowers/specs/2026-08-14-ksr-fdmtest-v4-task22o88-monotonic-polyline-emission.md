# Task 22O.88 — monotonic polyline emission

Port pinned `FillRectilinear.cpp:2584-2753`. Convert O87 chains into ordered
polylines using exact outer/inner endpoints, valid O83 perimeter arcs, vertical
runs, split handling, duplicate removal, scale-aware near-zero filtering, and
phony-pinch merge semantics.

Focused tests cover rectangular zigzag exact points, reversed orientation,
disconnected split paths, phony merge, empty input, repeatability, and immutable
slice/regions/path. Separate modules, <400 LOC, no source-splitting macros.

Deferred: filler orchestration/rotation, extrusion entities, lifecycle, G-code.
