# ARD0093 — Source-owned support connector arc consumption

Status: independently verified prerequisite; combined runtime verification is separate.

## Source boundary and destination

`OrcaSlicer/src/libslic3r/Fill/FillBase.cpp::connect_base_support`,
`ContourIntersectionPoint::could_take_prev/next`, `take_limited`, `take_next`,
`mark_boundary_segments_overlapping_infill` and long-cap emission own
`ares-core/src/fill/base_support/{connect,link,take,mark,caps,arches,extend}.rs`.
`libslic3r.h::EPSILON/SCALED_EPSILON` supplies the coordinate-scaled threshold.

## Decisions and scope

- Consume availability and trimming at the current endpoint before selecting
  the neighboring endpoint passed to take_next. A neighboring arc's available
  length is not the current arc's length.
- Vertical means equal X, matching the source directional test.
- Pass epsilon from the selected CoordinateScale through marking, availability,
  trimming and emitted finite arcs/caps: normal 100, large-bed 10. Do not fix
  the value at 16 or reuse normal-only 100 in the large-bed marking radius.
- Keep native traversal and scalar arithmetic, with per-call costs
  (`Fill/FillBase.cpp:2697-2701` at `345efec64`).
  No new allocations, fallback, guard or global state is introduced.

Included: these consumed connector repairs in the existing Fill module.
Deferred: additional epsilon-sensitive public configurations and broader Fill
option coverage; existing complete-output controls are regression evidence,
not proof that a threshold-sensitive RED input has been constructed.
Normal/Tree integration remains the separate combined ARD0084/0086 batch.

Execution evidence belongs in the matching plan.
