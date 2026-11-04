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
Official and separately identified seeded references must not share caches.
Historical WIP evidence is provenance, not verification of the port.

This supersedes WIP ARD-0037 for this worktree. Tower planning/reset integration
is a dependent slice, not proven by the no-tower tracer. Calibration modes,
hardware operation and exhaustive printer/option coverage remain unclaimed.

## Deferred upstream boundaries (explicitly verified present, not yet ported)

The [APA-R2 continuation SPEC](adaptive-pa-deferred-r2-spec.md) covers
the remaining source-cited witness groups. R1 already verified a
bounded uniform-width same-role sloped loop; the historical inventory below is
not a claim that every listed mechanism is still wholly absent. Calibration
mode admission, source-specific resets and unexercised residuals remain OPEN.

The following upstream sites exist in the pinned 2.4.2 source but are deferred
for later slices; none is silently omitted from the port's accounting:

- `GCode.cpp:5895-5960` and `GCode.cpp:6040-6070`: the multipath/loop average
  `m_multi_flow_segment_path_average_mm3_per_mm` and `m_multi_flow_segment_path_pa_set`
  suppression after the first multipath extrusion, including the sloped-seam
  (`seam_slope_*`) variant. The Rust `loop_paths/adaptive_pa.rs` mean and state
  fields exist; the seam-slope suppression interaction and the multi-path
  suppression inside `GCode::_extrude`/`extrude_loop` emission order are not
  yet exercised end-to-end.
- `GCode.cpp:6720-6745` and `GCode.cpp:6925-6975`, `7155-7205`: adaptive PA
  evaluation gating inside `_extrude`/`extrude_loop`, including the
  `is_pa_calib` exemption and the overhang override activation point.
- `GCode.cpp:1085-1095`, `1369-1376`, `3155-3162`: `resetPreviousPA` calls at
  tool-change, second toolchange site and initial-filament PA restore; the Rust
  side carries `state.adaptive_pa_resets`, but the upstream sequencing of each
  reset site is deferred to the tower/toolchange-dependent slice.
- `AdaptivePAProcessor.cpp:106-140`: the lookahead window that continues
  consuming feedrate changes until a feature change or wipe boundary; retained
  semantically in `adaptive_pa/scan.rs` but the upstream stream-position
  bookkeeping is not a line-for-line port.

Calibration modes (`CalibMode::Calib_PA_*` at `GCode.cpp:2915`, `3072`, `3246`,
`4692`), hardware-operation M900 diagnostics and exhaustive printer/option
coverage remain unclaimed.
