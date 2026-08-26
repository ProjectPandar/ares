# KSR FDM Test V4 semantic G-code parity

## Status

Accepted and amended. Original decision: 2026-08-16. Amendment: 2026-08-26.

## Problem

The project route now emits all 460 layers with production motion, arcs, wipes,
retraction lifecycle, templates, timing metadata, and statistics. A normalized
byte comparison is nevertheless not a valid OrcaSlicer contract. OrcaSlicer
2.4.2 `TriangleMeshSlicer.cpp:521-529` uses `tbb::parallel_for` and appends
intersections to shared per-layer vectors under mutexes. The lock protects
memory but does not define append order. Two runs of the same 2.4.2 Linux
AppImage (`orca-slicer` SHA-256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`)
on the same 3MF produced 6,338,754 and 6,330,090 bytes. After removing
generator/time/progress/object-ID fields, their first motion-order difference
was still at the Z1.2 spiral lift. Cooling-derived feed rates differed by up to
7 mm/min (0.517%). A single captured byte order therefore cannot be derived
from the 3MF and options without fixture-specific data.

## Decision

Treat `slice_project(project_bytes, metadata)` as the deep module and the CLI
semantic golden comparison as its external seam. Complete it through
source-cited Orca G-code slices rather than fixture-specific substitution.
Values come only from the 3MF's typed effective configuration and generated
geometry.

The implementation boundary follows `GCode.cpp:4539-7110`,
`GCodeWriter.cpp`, `GCode/SeamPlacer.cpp`, the arc-fitting path selected by
`enable_arc_fitting`, and `GCode/GCodeProcessor.cpp:1100-1140`. Motion, loop
handling, arc fitting, lifecycle, and post-processing remain separate normal
Rust modules; no `include!`/`include_bytes!` source splitting.

The golden seam validates exact layer metadata and exact multisets of deposited
segments keyed by feature, width, coordinates, arc geometry, extrusion,
acceleration, and fan state. It also validates exact wipe and retraction
lifecycles, lift shape, configuration, machine templates, control events,
volume/mass/cost statistics, generator version, and object labels. It ignores
independent-island traversal order and derived M73 placement. Cooling feed
rates must remain within both 10 mm/min and 1% of the reference; estimated
times within five seconds; displayed filament length within 0.05 mm. These
bounds contain the measured upstream run-to-run variation while rejecting
option or toolpath changes.

Independent-island order also changes the endpoints of inter-island XY travel
and may rotate a closed loop's selected arrival point. The invariant travel
comparison therefore checks motion count and kind, exact vertical profile,
arc direction, quantization-bounded radius and turns, the feed multiset, and
the acceleration-value set. It does not select one scheduler-dependent XY
route as canonical.

## Consequences

Each option-driven behavior is implemented and committed independently.
Source-stage byte encoders and browser oracle exports that pin internal
structures are removed once their externally observable behavior is covered.
Production code never reads the reference G-code or fixture identity. Ares
keeps deterministic cross-platform ordering rather than reproducing one TBB
schedule. Files remain below 400 LOC and tests live in dedicated modules.