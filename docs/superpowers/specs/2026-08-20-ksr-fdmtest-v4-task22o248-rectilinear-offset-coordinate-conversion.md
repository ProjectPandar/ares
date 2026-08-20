# Spec: Task 22O248 rectilinear offset coordinate conversion parity

## Observable contract

`fill_monotonic_surface` converts the two monotonic fill offset distances to scaled integer coordinates before invoking polygon offsetting. Positive and negative scaled values both truncate toward zero, matching OrcaSlicer's `coord_t` constructor parameters. The KSR fixture's first-layer executable G-code must advance beyond the current first arc-offset divergence after progress and dynamic object metadata are excluded.

## Upstream boundary

Port the argument conversion at `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:391-397, 2771-2775`: `float(scale_(...))` is passed into `ExPolygonWithOffset` parameters typed as `coord_t`. Keep Ares' existing rectilinear module seam; do not add fixture-specific geometry or output constants.

## Included behavior

- Scale the configured overlap/spacing formulas using their retained source precision.
- Convert each finite scaled result to the integer coordinate grid by truncating toward zero.
- Pass the resulting exact integral values to both outer and inner polygon offsets.

## Deferred behavior

Later rectilinear chaining, arc fitting, G-code progress placement, lifecycle metadata, and unrelated fill patterns remain unchanged.
