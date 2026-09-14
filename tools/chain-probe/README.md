# chain-probe — upstream `ShortestPath.cpp` oracle for chain order parity

Standalone C++ probes that run OrcaSlicer 2.4.2's exact `chain_polylines`
(greedy2 + two-exchange) and `chain_segments_greedy_constrained_reversals`
(v1, emission reorder) on dump files produced by ares diagnostics.

The `.inc` files are verbatim extractions from
`OrcaSlicer/src/libslic3r/ShortestPath.cpp` (see the line anchors in each
file's origin commit); `KDTreeIndirect.hpp` / `MutablePriorityQueue.hpp`
are upstream headers with two local patches: a `next_highest_power_of_2`
stub for `Utils.hpp`, and a 4x node-array pad because the upstream
heap-indexed build writes past its `next_highest_power_of_2(n+1)`
allocation for deep right-most paths (upstream is saved by heap layout
luck; the pad changes nothing for valid nodes).

Build:
    g++ -O1 -DNDEBUG -std=c++17 -I. probe.cpp  -o probe
    g++ -O1 -DNDEBUG -std=c++17 -I. probe2.cpp -o probe2

probe: `./probe <ARES_DUMP_PLANECLIP dump>` — runs upstream
`chain_polylines` on the clipped plane-path fragments (the `R` raw-scaled
tails emitted by `ARES_DUMP_PLANECLIP` in `fill/plane_path.rs`) and prints
`idx=`/`R` per position for comparison against ares's own chain order.

probe2: `./probe2 <ARES_DUMP_CHAIN dump> [block]` — runs upstream
`chain_segments_greedy_constrained_reversals` on one `C/E/O` block and
marks positions that differ from ares's recorded `O` order with `DIFF`.

Findings so far (2026-09-14, octagramspiral top-surface case):
- ares's greedy2+two-exchange chain output is identical to upstream's on
  the same fragment list (order and flips).
- ares's v1 emission reorder is identical to upstream's on the same
  entity list + seed (0 DIFFs over 73 entities).
- the remaining octagramspiral divergence is therefore in the fragment
  list entering those stages — the `classic_clip` scanbeam emulation's
  output order/orientation versus orca's `intersection_pl` output.
