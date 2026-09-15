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

probe4: `g++ -O1 -std=c++17 -I. -I<nix eigen>/include/eigen3 probe4.cpp -o probe4`
(needs `nix-shell -p gcc -p eigen`) — runs the vendored Clipper 6.2.6
open-path intersection (`ctIntersection`, pftNonZero, PolyTreeToPolylines)
on an `ARES_DUMP_PLANESUBJECT` subject + `ARES_DUMP_PLANEBOUND` clip
region and prints the fragment list for tie-order comparison against
ares's classic_clip output. `clipper.cpp`/`clipper.hpp` at the top level
and the `boost/`, `oneapi/`, `libslic3r/` stub dirs exist only to satisfy
the vendored header's include chain; `clipper/` holds the pristine
vendor sources.

## probe9-chain.cpp — vendored upstream chain_points (same-input oracle)

Assembles upstream's `chain_points` verbatim (ShortestPath.cpp:44-1012
template chain + MutablePriorityQueue.hpp + KDTreeIndirect.hpp) with a
mini Vec2d shim; reads `x y` pairs on stdin, prints the chained order.
Decisive use (ksr gcode layer 3, #170): ares' six surface bbox centers
from ARES_DUMP_PRELUDE feed in unchanged, and the upstream binary
prints `0 1 2 3 4 5` — identical to ares' own chain. The chain
machinery is therefore empirically exonerated: the island-order group
swap can only come from the chain INPUTS (the surface set/centers),
i.e. a slicing or surface-classification divergence on this .drc mesh.

Build: see the file header (g++ -O2 -std=c++17 -DNDEBUG with the stub
Utils.hpp providing next_highest_power_of_2).
