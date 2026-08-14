# Task 22O.85 — monotonic region costs

Port pinned `FillRectilinear.cpp:1989-2077,2179-2188`. Compute both orientation
costs over O84 topology using valid perimeter half-cost, straight split-gap
cost, source f32 order, and retained `CoordinateScale`; normalize by subtracting
the common minimum.

Focused tests cover symmetric zero difference, asymmetric orientation, split
gaps, exact f32 bits, both coordinate scales, repeatability, and immutable slice
input. Separate modules, <400 LOC, no source-splitting macros.

Deferred: path matrix/ant chaining, polylines/entities, lifecycle, G-code.
