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

## procgcode.patch — standalone GCodeProcessor oracle (--process-gcode)

Patches the CLI main with a `--process-gcode <file>` mode: applies a
default PrintConfig (marlin legacy, 1.75 PLA, 0.4 nozzle) before
process_file — without it the processor's per-extruder arrays are
empty and E-extrusion lines SEGFAULT (deterministic 5/5 on minimal
files; the full ksr reference never crashed because its comment
self-detection populates enough state). Build: same nix pattern as the
other patches (see `tools/chain-probe/procgcode.nix`). Verified outputs:
toolchange microbench 5.630681s, spiraltest2 3.015093s (the pre-config
binary gave 2.988959 — the spiral pin in processor/tests.rs needs
re-baselining), full ksr reference 6538.005371s (UNCHANGED by the
config — the same-stream ground truth stands).

## procgcode patch v2: ksr X2D machine caps

The --process-gcode config sets the ksr's real caps (extruding
20000 / retract 30000 / travel 9000) so the default-cap artifact does
not bind. Measured: full ksr reference 6538.005371; spiraltest2
2.988959; toolchange 5.621093. CORRECTIONS (supersede the earlier
-12ms/-26ms figures recorded here): the -12ms residual was a harness
newline bug (a fused G1 line in the GT file); with the correctly-
newlined file the toolchange-context spiral matches ares EXACTLY
(34.409561 both), and the "E-only retract block" difference is a
dump-scope artifact — upstream skips the cache push but counts the
time, same as ares. See the convergence spec #178-#179 for the full
correction chain.

## Diagnostic JSON and stable-marker variant

`--dump-color-change <json>` exports the processor's raw custom items,
normal custom times (seconds), color volumes (mm³), and these unconverted
`std::map<size_t, double>` statistics as `[filament_id, f64_mm3]` pairs:
`flush_per_filament`, `total_volumes_per_extruder`,
`model_volumes_per_extruder`, `support_volumes_per_extruder`, and
`wipe_tower_volumes_per_extruder`. Empty maps remain empty arrays. There is no
float narrowing, dense padding, allocation to events, or processor change.

The diagnostic reads G-code, not 3MF. Its bootstrap config above is replaced
from embedded G-code settings for recognized OrcaSlicer/Slic3r producers,
with defaults for absent settings (`GCodeProcessor.cpp:2620-2642`). Truncating
away the config footer changes that experiment. Load/unload/tool-change time
bootstrap defaults are zero; recognized embedded settings can override them.

The stable-marker diagnostic variant is retired: beta upstreamed the
pending-marker remap as `OptionsZCorrector::on_moves_inserted`
(`GCodeProcessor.hpp:1105-1116`, called at `GCodeProcessor.cpp:8132`),
so `ARES_PROCGCODE_BIN` (raw Native beta) is the complete authority for
ColorChange checks.

## Print statistics (Summary legend)

`print-statistics.nix` builds the seeded replay producer from the
`g3wj93ly…-orca-slicer-2.4.2-seed-replay-source` source tree with
`print-statistics.patch` applied. With `--outputdir <dir>` it also writes
`<dir>/plate_<n>.print_statistics.json` after `export_gcode`. The file
contains:

- the `PrintStatistics` totals set by `update_print_estimated_stats`
  (`GCode.cpp:1904-1940`), printed with `%.17g`;
- `modes[Normal|Stealth].time`, printed with `%.9g`;
- the `total_volumes_per_extruder` map.

Tests run it as `ARES_PRINT_STATISTICS_BIN` through `OrcaRunner`, pinned to
one CPU; see `tests/parity/summary/README.md`.
