# Task 22O.86 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:1590-1709`, into
`fill::rectilinear::path_matrix`.

Represent each region/orientation pair in a dense `2N × 2N` matrix. Lazily
compute source f32 Euclidean endpoint length and visibility using O80 boundary
selection and retained coordinate scale, cache exact bits, retain pheromone per
edge, and update only pheromone during initial-deposit reset. Preserve the
pinned source's adjacent-contour guard literally; its compared boundaries do not
fire for valid left-to-right regions.

Deferred: ant simulation, RNG, pheromone evaporation/reinforcement, path
selection, polyline emission, entities, lifecycle, and G-code. No fallback,
fixture branch, or public API.

Compile RED proved the missing matrix module. Two focused tests pass for exact
four-orientation f32 bits, lazy cache identity, both scales, and pheromone-only
reset; both O85 regressions remain green. Strict core Clippy, rustfmt, diff, and
sub-400-LOC gates pass. Matrix and focused test shards are 77 and 49 LOC.
