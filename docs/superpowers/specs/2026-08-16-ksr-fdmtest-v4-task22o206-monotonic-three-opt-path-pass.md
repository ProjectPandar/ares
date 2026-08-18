# Spec: KSR FDM Test V4 task206 monotonic three-opt path pass

## Observable contract

Every ant-generated monotonic region path receives OrcaSlicer's single forward three-opt pass before path length comparison. For each four-link window, the middle two links swap only when the second does not depend directly on the first and replacing the three affected transitions strictly lowers total transition length. Equality preserves source order.

The source comparison is the red signal: Ares measured ant paths without the mandatory three-opt pass. Fixture comparison confirms the pass does not affect the first divergent stored arc; it may affect later multi-region paths while preserving precedence and deterministic RNG behavior. Files remain below 400 LOC; monotonic chain tests, formatting, and Clippy remain clean.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `Fill/FillRectilinear.cpp:2189-2216,2539-2549`, `monotonic_3_opt` and its call before ant path measurement, into `fill::rectilinear::chain`. Pheromone/RNG behavior, infill counts, outline nudging, timing, and remaining G-code differences are otherwise unchanged and deferred.
