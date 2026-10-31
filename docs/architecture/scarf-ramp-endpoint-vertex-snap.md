# Scarf ramp endpoint snaps to a nearby path vertex

## Source boundary and destination

This is a source-cited rewrite slice of OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/ExtrusionEntity.cpp:487-505::split_at_slope_end`, called by the
`ExtrusionLoopSloped` constructor when a path exceeds the remaining ramp length.
The old v2.4.2 constructor split unconditionally at that length.

The intended Rust destination is the ramp split in
`project_slice/gcode_emit/motion/scarf.rs::build` and its private `split_path`
helper. That unconditional split is a temporary compatibility shell around the
upstream sloped-loop construction, not an independent scarf geometry policy.
No production change is part of this RED.

The maximum segment-length caller contract is unchanged between
`GCode.cpp:5944-5951` at v2.4.2 and `GCode.cpp:8695-8702` at beta:
`slope_min_length / slope_steps`, converted to scaled coordinates. The existing
input explicitly uses 10mm and four steps; this is not a default/import mismatch
or a proposal to alter the emission caller's speed policy.

## Included behavior and non-negotiable decisions

- Scan the source path in order, accumulating its lengths until the requested
  ramp endpoint falls within an edge.
- The source snap distance is `min(0.5 * slope_max_segment_length, scale_(1.0))`.
  If the remaining distance to that edge's next vertex is strictly less than
  this distance, split at that vertex instead; otherwise split at the requested
  length. Preserve the strict comparison, units and evaluation order.
- Pass the resulting ramp to the existing recursive half-segment detail step.
  An extended ramp can therefore require more subdivisions without changing the
  configured maximum segment length or step count. Do not compensate by changing
  that limit, positions, flow, E precision, speed or the formatter.
- Keep start/end slope ratios and the later clipping and emission contracts
  unchanged. No fixture-specific branch, tolerance tuning, new configuration,
  private diagnostic API or geometry backend replacement is permitted.

Adjacent beta `trim_seam_ends` behavior, preservation of arc-fitting metadata,
other ramp/role clipping changes, seam placement, default speed selection,
cooling/time/progress changes and whole-file configuration/footer differences
are deferred. This witness uses unfitted straight edges. A later slice must cite
those adjacent upstream boundaries rather than silently expanding this one.

## Public RED and its limits

Reuse `tests/parity/scarf-default-threshold/boundary.3mf` byte-for-byte, SHA256
`1646ba992cc58685074abd95e009d17f85f88e190379912201d9d9f88399f4e5`.
The existing whole-file test
`runner::scarf_default_threshold::scarf_default_threshold_matches_complete_output`
remains unchanged. It reproduces the beta ledger's first mismatch at the first
non-first-layer scarf extrusion: Native advances to X106.25/Z0.225, while Ares
advances to X107.5/Z0.25. Its Native feedrate prerequisites still hold.

The new public case is
`option_coverage::scarf_ramp_endpoint_vertex_snap::scarf_ramp_snaps_to_nearby_vertex_without_a_short_flat_stub`.
Its seam is fresh official-beta Native replay versus
`ares_core::slice_project_plate_validated` on that same input. Compare the entire
second-layer object, retaining entry travel, M73, feed commands, all ramp/flat/end
motions, extrusion values and terminal E0. Do not omit, sort, rotate, normalize or
retune any part of that object. Full supplied Native/API streams are retained
externally; no Native golden output is committed.

The saved whole-file reproduction has a 588B Native object with eight ramp
subdivisions, versus a 393B Ares object with four and a short flat stub. Fresh
double-replay stability and the new validated-API comparison must qualify this
RED before implementation approval. There is no claimed future GREEN.

Closing this owner-focused object comparison would not itself close the original
complete-file ledger row. That comparison, its progress/footer evidence and any
remaining cross-owner differences stay visible and require their own disposition.
