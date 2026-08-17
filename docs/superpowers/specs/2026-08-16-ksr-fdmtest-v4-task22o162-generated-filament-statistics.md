# Spec: Task 22O.162 generated filament statistics

## Observable contract

The KSR project G-code ends with per-filament used length, volume, weight, and cost comments after `EXECUTABLE_BLOCK_END`. Values are computed from generated deposition moves and the loaded `filament_diameter`, `filament_density`, and `filament_cost` options; production code must not inspect fixture names or reference G-code.

## Upstream boundary

This slice ports the relevant behavior from OrcaSlicer 2.4.2 `src/libslic3r/Extruder.cpp:30-49,124-145` and `src/libslic3r/GCode.cpp:2312-2368`.

Included behavior:

- raw generated line and arc extrusion deltas accumulate without retraction/unretraction double counting;
- filament length converts to volume, mass, and cost from typed project filament options;
- configured but unused filament slots emit zero values;
- source two-decimal footer formatting and placement after the executable block.

Deferred behavior: bringing the KSR material totals from the current generated geometry to the reference totals, object identifiers, sub-micron extrusion differences, later path ordering, cooling, and timing.

## Acceptance

A focused `slice_project` test asserts the complete generated KSR filament-statistics footer. The changed core crate passes Clippy and rustfmt, and every changed Rust source remains below 400 lines.
