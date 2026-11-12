# TimelapsePosPicker optional farthest target

## Rewrite boundary

Pinned upstream: Orca `345efec64c7b95a05ce5cf96df0dde66a158835c`.

Included source:

- `src/libslic3r/GCode.cpp::compute_farthest_point:6307–6310,6438–6457` resets the global point to zero, then retains the external-perimeter/infill/support producer result and photo-head ownership.
- The enable/inline consumers at `GCode.cpp:6683–6687,9831–9853` own farthest production and inline-photo state.
- `GCode.cpp::timelapse_gcode_config:6526–6531` engages PosPickCtx::farthest_point only when farthest is enabled. It subtracts the separately scaled/rounded plate origin from the global scaled point.
- `GCode/TimelapsePosPicker.cpp:552` forwards the optional target to pick_pos_internal.
- `pick_pos_internal:406–414` uses candidate-to-target L1 distance when the target is engaged. Without a target it uses current-to-candidate L1 minus one third of candidate-to-camera L1. Both branches are upstream behavior.

Rust destination: `gcode_emit/layers/farthest.rs::Context` and `begin_layer`, with forwarding through `timelapse.rs`, `timelapse_pos.rs::position` and `pick_pos_internal`. The existing `layers/farthest/target.rs::compute` remains the producer. The traversal-based picker is a compatibility shell for the upstream context, not a separate planner or parallel per-layer state machine.

## Non-negotiable decisions

1. Retain an optional plate-relative, native fixed-scale Point in the existing Context. Disabled layers have no target. Enabled layers start from the upstream zero global point minus plate origin when the search finds no candidate. A candidate replaces that value.
2. Reuse the global farthest_gcode_pos/producer result. Convert its geometry scale to the picker's native fixed scale and subtract a separately rounded plate origin. Do not round a global-minus-origin floating difference.
3. Forward the optional point without recomputing a farthest point or applying physical_extruder_map to geometry/head indices.
4. Change only the engaged-target objective. Preserve the current-position early return, absent-target camera score, candidate enumeration, clearance truncation, height arithmetic, smooth/by-object handling and inline ownership. No forced-safe flags, hard-coded safe positions, epsilon, cooling/retract/clock compensation or comparator normalization changes.

## Tests

The no-target control derives from repository P0 and changes only farthest_point_timelapse to 0. Its metadata records synthetic repository provenance, source/parent hashes and the changed archive field. Native publishes 50 safe snapshots in both official and seeded modes. Complete Native/API/CLI output comparison with the original timelapse script protects the absent-target camera branch.

The five original printer_physical_identity cases exercise the engaged target without changing their inputs or assertions. Before target forwarding, their whole-output comparison fails at later M9711 coordinates with X248 Y163 expected and X177 Y233 produced. The clock headers and height-decision blocks match. Both the height-marker control and the no-target control remain part of the complete-output comparisons.

An upstream P0 trace has an engaged target (179790001,164790001), zero plate origin and matching measured safe polygons. Replaying only the objective selects (248,163) for the target score and (177,233) for the camera score. This isolates the objective but does not establish complete output or general producer parity. Direct stored/forwarded Rust point observations can be compared with Native global/context points. Optimized-out locals remain distinct from directly measured values.

Failure modes:

- The optional target is dropped or the camera score is used while a target is engaged.
- The camera score is removed when the target is absent.
- A disabled or newly initialized layer retains a stale target.
- Native scale conversion or plate-origin subtraction changes the point.
- Producer ownership, inline photo timing or height decisions change.
- Coordinates match while other complete-output bytes differ.

## Deferred behavior

Heap/tie/sampling/path-collision machinery, object/bbox enumeration, writer-current-position context, multi-plate inline endpoint matching, rods/rafts and tower geometry remain outside this target-forwarding boundary. The plate-origin formula follows the cited source. The five zero-origin cases do not cover nonzero-origin or multi-plate producer behavior.
