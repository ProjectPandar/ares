# Spec: Task 22O.245 rectilinear fractional offsets

## Observable contract

The KSR first-layer rectilinear path emits `G1 X132.813 Y87.283 E.03177` between the adjacent project-derived moves, matching OrcaSlicer 2.4.2. Offset distances retain the fractional scaled coordinate represented by the source `float`; production code does not inspect fixture names, reference G-code, or known coordinates.

## Upstream boundary

Rewrite OrcaSlicer 2.4.2 `src/libslic3r/libslic3r.h:91-94` and `src/libslic3r/Fill/FillBase.cpp:100-125` at `fill::rectilinear::surface::scaled_offsets`. The `scale_` macro divides by `SCALING_FACTOR`, and the explicit `float` conversion retains the fractional coordinate. It does not truncate to `coord_t`.

Included: overlap-derived outer and inner rectilinear offsets. Deferred: subsequent path geometry, timing/M73, and exact G-code parity.

## Acceptance

A focused `slice_project` test observes the exact adjacent E word and the rectilinear test observes the source float values. Changed Rust files remain below 400 lines and pass focused nextest before the slice is committed and pushed independently.
