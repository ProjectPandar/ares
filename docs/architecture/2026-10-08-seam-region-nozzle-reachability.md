# Seam B: region nozzle angle-arm reachability search

Status: NO RED FOUND in the bounded input search below; no production
change. This does not prove universal unreachability. The rewrite
boundary remains section B of
[gcode-seam-parameter-scope-and-visibility-order.md](gcode-seam-parameter-scope-and-visibility-order.md).
Baseline: `325ba112d1856d0178a4672068d3f8784afdf72e`.

## Source boundary

Pinned upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`GCode/SeamPlacer.cpp:456–471` samples polygon angles with
`region->flow(frExternalPerimeter).nozzle_diameter()`; a null region uses0.5.
`LayerRegion.cpp:21–28` and `PrintRegion.cpp:6–22,25–53` resolve that flow
using the region's outer-wall filament. Ares `seam_placement.rs` currently
threads the first global nozzle through `alignment/preparation.rs` and
`seam_candidates.rs`. That source discrepancy is real, but does not itself
supply an observable whole-file RED.

The intended port would carry the existing external-flow nozzle alongside
flow width and source polygon ordering. It remains unimplemented. Flow
construction, tool ordering, temperature scheduling, cooling, and candidate
angle algorithms remain outside B.

## Native experiments

Every result below used a fresh seeded Native cache, the public Ares load/
slice boundary, and the existing complete-byte comparator. Only generated-by
was normalized. A temporary CLI discovery test was archived outside the
repository, then removed; no private production probes were introduced.

Snapmaker J1, Manual filament mapping `[1,2]`, distinct filament colours,
Nearest seam, fixed0.65 wall widths, two walls, no support/prime tower.
Two disjoint Dumbbell model parts belong to one object: left part uses tool1,
right part uses tool2. Global roles use tool1. Public loading checks one
object/two volumes and the nozzle vector. Same-diameter `.4/.4` inputs are
paired with otherwise equivalent `.4/.6` inputs. The target region's
external flow selects nozzle2. Absolute bridge width0.4 avoids an unrelated
Native validation boundary at a percentage-derived0.6 width.

| Round | Paired configurations | Complete-byte result |
| --- | --- | --- |
| regions-v2 | Three layers, simple startup/end | Both DIVERGENT at line32: Native T1 preheat220 vs Ares150 |
| regions-v3 | Zero standby delta, layer cooling slowdown disabled | Both DIVERGENT; same-diameter first difference line275 M73; later both share a third-layer seam mismatch |
| regions-v4 | Single0.2mm layer, otherwise v3 | Both PASS |
| regions-v5 | XY scale/rotation `(0.5,0)`, `(0.75,0)`, `(1,17)`, `(1,43)`, `(0.5,17)`, `(0.75,43)`, `(1.25,17)`, `(1.5,43)` | Four pairs PASS; four pairs both DIVERGENT at line41 during initial tool change |
| regions-v6 | Single-layer shoulders0.65/0.8/1.0/1.2mm | All four pairs PASS |

Thus nine paired configurations (18 slices) passed whole-byte comparison.
The remaining six pairs (12 slices) do not isolate B: their same-diameter
controls also fail. No failure has been relabelled PASS. Diagnostic movement
comparisons used to locate the later third-layer seam mismatch do not replace
the complete-file verdict or remove earlier M73 differences.

## Frozen independent initial-tool finding

The earlier v2 single-region stock-startup input independently exposes
initial-tool selection with equal nozzle diameters; it is preserved as a
separate finding. Input SHA256:
`7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4`.
True first difference: line18, Native `M73 P0 R2`, Ares `M73 P0 R3`.
Startup diagnosis: Native T1 purge targetX324; Ares selects T0 before T1
and purges towardX0. Upstream ownership is `GCode.cpp:2765–2780,2845–2846`
and `ToolOrdering::first_extruder`, not SeamPlacer. Simplified startup inputs
above do not replace or overwrite this finding.

## Disposition

Per the no-RED rule, defer B production changes and report the
bounded search rather than manufacture a RED or fix unrelated behavior.
No GREEN or full-workspace/official gate is claimed: this slice contains
only the investigation report. A future B implementation still requires a
real complete-byte Native RED and fresh paired seeded/official gates.
