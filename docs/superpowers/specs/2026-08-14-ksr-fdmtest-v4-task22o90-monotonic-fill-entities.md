# Task 22O.90 — monotonic fill entities

Port pinned `Fill.cpp:1213-1374` and `FillBase.cpp:133-155` for Monotonic and
MonotonicLine. Dispatch grouped fills, derive O89 parameters only from retained
3MF/effective graph state and pinned calculations, and emit ordered collections
with exact role/mm3-per-mm/width/height.

Focused tests cover InternalSolid Monotonic and Top MonotonicLine geometry,
metadata, source anchor difference, ordering, repeatability, graph immutability,
non-supported pattern no-fallback, and atomic geometry errors. Separate modules,
<400 LOC, no source-splitting macros.

Deferred: other fillers/thin fill, lifecycle, motion, G-code.
