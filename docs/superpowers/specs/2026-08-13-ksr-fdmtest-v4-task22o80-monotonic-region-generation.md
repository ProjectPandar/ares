# Task 22O.80 — monotonic region generation

Port pinned `FillRectilinear.cpp:1590-1629,1711-1931` over O79 sections.
Return ordered regions with exact left/right boundary indices and source flip
parity. Preserve seed order, valid vertical-run traversal, exclusive adjacent
overlap tests, consumed state, and stop conditions.

Focused tests cover one rectangular run, odd/even width flip parity, separated
runs, hole/multiple overlaps, consumed no-repeat, repeatability, and input
immutability through an owned working copy. Separate modules, <400 LOC, no
source-splitting macros.

Deferred: region neighbors/lengths, chaining, polylines, entities, lifecycle,
G-code.
