# ARD0092 — Clipped-empty loop state

Status: finite slice verified; broader output parity remains open.

`GCode.cpp:5796-5805::extrude_loop` owns the empty clipped-loop return in
`gcode_emit/motion/loop_paths.rs`. Decide whether any ordinary path remains
before wipe-before-external or modifying the wipe path/inward position.
Keep scarf handling and active spiral-vase clipping semantics separate, as
upstream does. No geometry rebuild, defensive cloning, new fallback or
support feature guard is needed.

`GCode.cpp:320-358,435-497,7659-7670::calculateWipeRetractionLengths/wipe/retract`
owns stored-path eligibility in `motion/travel/wipe_sequence.rs`. Eligibility
is independent of whether the substituted current path has moving segments.
Preserve the pre-wipe fraction, speed/tags, path reset and final top-up even
for a zero-length wipe. Do not discard this native lifecycle or manufacture
movement. Reuse the existing eligibility predicate, without an added state
field, geometry clone or new public option.

The real project-input/complete-output boundary is authoritative. A public
seam gap that consumes all ordinary cube wall loops must leave actual infill
and its native movement stream intact. No new private/source-pin assertion
may substitute for that result. Existing one-region motion scaffolding is a
temporary upstream compatibility shell, not a new Ares emission architecture.

This slice is independent of the held Normal/Tree batch and may be submitted
only after its own real-output/regression/quality review. It does not close
Tree, modifiers, printer/option matrix or cross-platform runtime parity.
