# Task 22O.81 — monotonic region neighbors

Port pinned `FillRectilinear.cpp:2079-2179`. Given linked sections and O80
regions, populate sorted unique left/right region-neighbor indices and repair
asymmetry exactly.

Focused tests cover a linear chain, one-to-many overlap, duplicate suppression,
symmetry, no-neighbor identity, repeatability, and immutable section input.
Separate modules, <400 LOC, no source-splitting macros.

Deferred: path lengths, ant chaining, polylines/entities, lifecycle, G-code.
