# Spec: KSR FDM Test V4 task214 source no-op monotonic three-opt

## Observable contract

For the first KSR bottom-surface monotonic fill after `G1 X99.635 Y137.851 E.15048`, Ares emits the source ant order whose next extrusion begins `G1 X111.296 Y141.099 E.0564`. Ares does not reorder ant paths through an optimization absent from OrcaSlicer 2.4.2.

The fixture assertion is exercised through `slice_project`; production code remains fixture-independent. The corrected order must remove the previously reordered travel while preserving earlier exact-E differences. Rust files remain below 400 LOC, and formatting, focused tests, and strict workspace Clippy remain clean.

## Upstream boundary

This slice corrects the earlier reading of OrcaSlicer 2.4.2 `src/libslic3r/Fill/FillRectilinear.cpp:2186-2207,2539-2549`: `monotonic_3_opt` is intentionally empty, so its call has no effect before ant-path measurement. Ares removes its invented middle-link swap rather than implementing behavior not present upstream. Ant generation, pheromone updates, path measurement, fill geometry, exact E, cooling, timing, and later G-code differences are otherwise unchanged and deferred.
