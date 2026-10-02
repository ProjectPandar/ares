# Single-region ironing emission rewrite

## Boundary

Port `libslic3r/GCode.cpp:5445-5468` and
`GCode::extrude_infill` (6150-6178) into
`ares-core/src/project_slice/gcode_emit/motion.rs` as defined in ARD-0090.
The associated eager retraction follows `GCode.cpp::GCode::retract`
(7654-7704) in `gcode_emit/motion/travel.rs::retract_for_timelapse`.
Existing island ordering remains a temporary single-region source shell.

## Included behavior

Emit ordinary infill before infill-first walls and ironing after them.
Chain each infill phase independently from its actual current cursor.
Retain first-layer wall-first behavior. The timelapse insertion gate sees
original infills, including ironing, and applies before ordinary infill in
both wall-order branches. Empty islands do not request an insertion.
Already-completed layer-change retraction still performs an allowed eager
lift before the timelapse template, as upstream does; reset E precedes lift.

## Deferred behavior

Regional ownership/configuration switching and compatible perimeter generation,
multiple tools/wiping and support remain outside this slice. Normal cannot
merge before Tree parity. No fixture, comparator relaxation or support guards.

## Exit criteria

Public cube 3MF with `is_infill_first=1` and `ironing_type=top` must activate
native wall/ironing output and preserve the complete ordered stream. Add a
traditional-timelapse variant with an explicit native marker. Retain all
comparison artifacts before checking activation. Verify sensitive RED/GREEN,
existing controls, workspace Nextest, fmt/Clippy, Tier1 compile checks and
independent six-axis review. Compile checks are not native/browser runtime
acceptance. Submit code/docs together only after independence on clean HEAD.
