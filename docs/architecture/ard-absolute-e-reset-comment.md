# ARD: preamble reset comment follows GCodeWriter serialization

Decision: the missing absolute-E verbose reset comment is a bounded Rust rewrite
of `GCodeWriter.cpp:82–106,480–503`, with comment enablement from `GCode.cpp:2038`.
The sole Rust destination is machine.rs::append_flavor_preamble. Do not introduce
a generic reset helper or redesign the writer pipeline for this one consumer.

Non-negotiable constraints:

- Preserve exact command order, coordinates, flavor/mode admission and newline.
- Use gcode_comments only for the cited reset suffix. Relative mode gets no new
  reset; quiet absolute mode keeps the existing bare reset.
- Call existing native_stream_case::check / qualify directly; the Native
  validator closure asserts only annotated reset presence for absolute verbose
  and absence for quiet/relative controls. Whole Native/validated API/real CLI
  comparison, unmasked comments and raw API==CLI remain the public oracle;
  both-mode global Native preflight finishes before public runs.
- Do not extend paint_color activity or used_extruder_case: tool-selection
  witnesses are not this slice's seam. Preserve failed cce75f02f and its STOP
  packet/binaries; no new scanner/helper, input or shared-activity changes.
- Keep the older-base provenance and rejected Native
  attempt visible. Source-derived GUI reachability is not a live-save claim.
- SPEC→tests-only actual RED→minimal GREEN; root remains400 through existing
  grouped registration. No production edits before demonstrated RED.
- No extra fallback, private-helper oracle, pressure-equalizer behavior, other
  reset_e call sites, new crate or owner-adjacent cleanup.

Included behavior is only conditional comment serialization. Adjacent reset_e
state/force/other-caller behavior is deferred; existing Ares code there remains
a temporary compatibility shell around upstream writer concepts, not a new
Ares-owned pipeline.
