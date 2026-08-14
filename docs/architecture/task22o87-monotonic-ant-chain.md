# Task 22O.87 architecture decision record

## Status

Accepted for implementation. Decision date: 2026-08-14.

## Decision

Port pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Fill/FillRectilinear.cpp:2190-2582`, into
`fill::rectilinear::chain` and `fill::rectilinear::rng`.

Use the standard MT19937-64 default state, source dependency queue, greedy
initial pheromone, 25 rounds, at most 10 ants, eight no-change exit, 0.9
best-edge choice, probability fallback, local diversification, global
reinforcement/evaporation, strict shortest-path replacement, and both region
orientations over O86. Preserve the pinned empty `monotonic_3_opt` behavior.

Output owned `(region_index, flipped)` links; no pointers or public API cross the
Rust seam.

Deferred: path-to-polyline emission, filler entities, lifecycle, and G-code. No
fallback or fixture branch.

Compile RED proved both chain and RNG modules missing. Three focused tests pass
for standard MT19937-64 output, empty/single behavior, exact seeded branching
order/orientations, precedence, repeatability, and immutable input; both O86
regressions remain green. Strict core Clippy, rustfmt, diff, and sub-400-LOC
gates pass. Chain, RNG, and focused test shards are 300, 67, and 54 LOC.
