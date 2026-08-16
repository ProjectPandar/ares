# KSR FDM Test V4 complete G-code parity

## Status

Accepted. Decision date: 2026-08-16.

## Problem

The project route emits all 460 layers but not production-equivalent motion: the current output contains non-finite extrusion values, no arcs, wipes, or retraction lifecycle, incomplete acceleration/speed selection, no end templates/statistics, and placeholder time metadata. The current normalized golden comparison first diverges at the initial M73 remaining-time value.

## Decision

Treat `slice_project(project_bytes, metadata)` as the deep module and the CLI golden comparison as its external seam. Complete it through source-cited Orca G-code slices rather than fixture-specific substitution. Values come only from the 3MF's typed effective configuration and generated geometry.

The implementation boundary follows `GCode.cpp:4539-7110`, `GCodeWriter.cpp`, `GCode/SeamPlacer.cpp`, the arc-fitting path selected by `enable_arc_fitting`, and `GCode/GCodeProcessor.cpp:1100-1140`. Motion, loop handling, arc fitting, lifecycle, and post-processing are separate normal Rust modules; no `include!`/`include_bytes!` source splitting.

The only golden difference allowed is the validated generator line: slicer name `Ares` and generation timestamp. Printing-time estimates, progress, executable block, end G-code, and filament statistics are observable output and must match.

## Consequences

Each option-driven behavior is implemented and committed independently. Source-stage byte encoders and browser oracle exports that pin internal structures are removed once their externally observable behavior is covered. Files remain below 400 LOC and tests live in dedicated modules.