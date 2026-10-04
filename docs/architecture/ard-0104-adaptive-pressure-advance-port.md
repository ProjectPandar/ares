# ARD-0104: Port the adaptive PA filter after cooling

## Source and destination

OrcaSlicer v2.4.2 (`8500fcdccaa10b5099ac20d252af3a7c560046f1`)
owns this behavior in `src/libslic3r/GCode/AdaptivePAProcessor.cpp`,
`AdaptivePAInterpolator.cpp`, `PchipInterpolatorHelper.cpp`, and
`GCode.cpp::{_extrude,extrude_loop,process_layers,set_extruder}`.
The destination is private `ares-core::project_slice::gcode_emit::adaptive_pa`
and its existing motion/filter call sites. The project emitter remains a
temporary compatibility shell, not an independently designed slicing pipeline.

## Decisions

Port the WIP branch's arithmetic and scanner selectively, not its old
single-material indexing or its whole emitter. Respect current regional motion
options and logical filament ownership. Adaptive prediction consumes cooled,
fan-processed layer output only; machine start, end and final purge are not
filtered. Preserve PCHIP endpoint clamping, acceleration grouping, three-decimal
rounding, wipe exclusion, lookahead boundaries and duplicate suppression.
Source-defined invalid-model handling retains static PA; no new unsupported
guards or silent input exclusions are permitted.

The public multi-tool tracer also exercises upstream comment forwarding:
`GCodeWriter::{set_junction_deviation,_spiral_travel_to_z,travel_to_xyz}` and
`GCode::{extrude_loop,_extrude}` (arc descriptions). Correct the existing
machine-envelope, inward-move, lift and constant-path emit sites to forward
these source comments under `gcode_comments`; never strip comments in the
comparator or disable this option to hide residuals. These are source-owned
text emission repairs, not changes to geometry, routing or lift policy.
Calibration numeric prefixes follow `std::stod`; PA commands follow
`GCodeWriter::set_pressure_advance`'s four-significant-digit `defaultfloat`,
including notation selection after rounding.

Tests use the user-requested public exported-3MF/full-G-code seam. Keep the
existing generator-only comparator unchanged and retain each paired stream.
Official CPU0 and separately identified seeded references must not share caches.
Historical WIP evidence is provenance, not verification of the port.

This supersedes WIP ARD-0037 for this worktree. Tower planning/reset integration
is a dependent slice, not proven by the no-tower tracer. Calibration modes,
hardware operation and exhaustive printer/option coverage remain unclaimed.
