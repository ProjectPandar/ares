# Task 22O.96 — infill chaining dependencies

Port pinned `ShortestPath.cpp:15-40,92-393,1026-1069`,
`ExtrusionEntityCollection.cpp:65-72,87-96`,
`ExtrusionEntityCollection.hpp:78-123`, and `FillBase.cpp:161-185` as pure
project-slice dependencies.

Requirements:

- extend classic shortest path with explicit-cursor constrained reversal and
  source nearest-neighbor fallback;
- expose endpoint/reversal operations for generated fill collections/paths and
  retained gap paths/loops;
- implement pure `chained_path_from` honoring `no_sort`;
- set no-sort for Monotonic/MonotonicLine and keep CrossHatch sortable;
- preserve reached KD-tree/priority-queue tie behavior;
- do not invoke chaining from `slice_project` and do not synthesize a cursor.

Focused RED/GREEN tests cover constrained starts, reversal, gap loops, internal
collection ordering, and pattern no-sort ownership. Files remain below 400 LOC
and use ordinary modules, never source-splitting macros.
