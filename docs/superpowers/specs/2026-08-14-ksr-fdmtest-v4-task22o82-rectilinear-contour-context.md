# Task 22O.82 — rectilinear contour context

Port pinned `FillRectilinear.cpp:357-457,759-993` retained ownership. Produce a
single owned slice with rotated source geometry, outer contours before inner
contours, inner/outer identity, and vertical intersections indexed into that
same inventory.

Focused tests cover rectangle and donut inventory order, offset coordinates,
intersection index addressability, repeatability, nonmutation, and range-error
atomicity. Separate modules, <400 LOC, no source-splitting macros.

Deferred: perimeter measurement/emission, chaining, entities, lifecycle,
G-code.
