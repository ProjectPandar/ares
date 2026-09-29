# Plate-frame wipe clipping

## Upstream boundary

Port `GCode::Wipe::wipe` (`OrcaSlicer/src/libslic3r/GCode.cpp:443-491`),
`Polyline::clip_end` (`Polyline.cpp:52-72`), and the active origin context
from `generate_skirt` / `generate_object_brim` (`GCode.cpp:4411,4499`) and
object emission (`:5385-5442`) into `ares-core`'s
`project_slice/gcode_emit/motion/{path,state,travel/wipe_path}.rs`.

## Evidence and scope

Anycubic Kobra 3 0.2 nozzle emits E-.70071/E-.09929 where the reference
emits E-.70072/E-.09928 during first-layer brim wiping. The start and
intermediate integer points agree; clipping negative local coordinates
instead of positive plate coordinates changes the truncation result.

Track whether the current emission uses upstream's zero-origin plate
frame. Use it for the clipped endpoint's truncation and final coordinate
conversion; retain local storage for cursors and wipe points. Set this
context from the destination path before its travel/retract, not from
the previous path's role. This is a temporary compatibility shell around
upstream origin semantics, not a new Ares-owned pipeline.

Full `set_origin` state-model replacement, per-object skirt grouping,
other residual printer differences, and unrelated option gaps are deferred.

## Acceptance

- Real-Orca complete-output smoke fails before the change and passes after.
- Cover plate-loop wiping and a transition back to model-space wiping.
- Preserve exact E distribution, local cursor, and nonzero extruder offset.
- No comparison tolerance, debug probes, fallback, or source-pinning test.
- fmt, Clippy, workspace Nextest, platform compilation and independent
  six-aspect review pass before code/docs are committed and pushed together.
