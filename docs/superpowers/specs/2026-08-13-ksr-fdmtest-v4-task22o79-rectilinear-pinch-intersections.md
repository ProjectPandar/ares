# Task 22O.79 — rectilinear pinch intersections

Port pinned `FillRectilinear.cpp:1216-1312` over O78 linked sections.

Insert phony OuterHigh/OuterLow midpoint pairs only for disconnected adjacent
InnerHigh/InnerLow runs, preserve insertion order, and remap every affected
same-line and neighboring horizontal link. Phony records use invalid identity
and links. Nonpinched sections remain byte-for-byte unchanged.

Focused tests cover no-op, one pinch, multiple pinches, midpoint truncation,
all three reindex directions, repeatability, and source geometry immutability.
Separate modules, <400 LOC, no source-splitting macros.

Deferred: monotonic regions/traversal, fillers, entities, lifecycle, G-code.
