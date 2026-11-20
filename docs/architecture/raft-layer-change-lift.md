# Lift at the raft-to-object layer change

Upstream: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `src/libslic3r`.

- `GCode.cpp:6731–6746` runs `change_layer` before the new layer's custom G-code and path emission. The ordinary instance loop dispatches support (including interface) and restores the object layer and its over-raft marker at `GCode.cpp:7625–7630,7671–7706`.
- `GCode.cpp:8458–8484` tests `retract_when_changing_layer` and the live writer's `will_move_z` before retraction, then updates nominal and writer Z and marks the next travel as forced Z. Only spiral vase immediately calls `travel_to_z` here. `GCodeWriter.cpp:1156–1172,1279–1294` defines the Z dead band and lift cancellation for the standalone move.
- `GCode.cpp:10865–10895` and `GCodeWriter.cpp:901–930` determine eligibility and reserve a lazy lift. `GCode.cpp:9357–9375,9402–9407` moves to the first extrusion point and then unretracts. `GCode.cpp:10449–10451,10593–10623,10637–10651,10714–10719` separates travel retraction from the forced Z move. `GCodeWriter.cpp:1001–1011,1096–1140,1541–1549` consumes or cancels lift before and after that travel.

## Rewrite boundary

Keep the Ares layer boundary and first-path travel scaffold. Port the known-position predicate in `GCodeWriter.cpp:1006–1011` to `project_slice::gcode_emit::motion::travel::lift::emit_pending`: if live XYZ exactly equals the destination, consume the pending lift without raising the target. The layer change has already set the writer's logical Z, so the first path can have identical writer XYZ even though a new physical Z move must be emitted. Leave the forced XYZ travel intact. Do not suppress lifts for unknown position, equal XY at a different Z, or merely equal formatted coordinates.

This is separate from the existing Auto-to-Spiral layer-change reservation in `change-layer-spiral-lift-reservation.md`. Leave support geometry, interface extrusion, lift type selection, tower motion and progress command placement unchanged.

## Verification

Use a synthetic MK4 tree-slim three-layer raft project with normal object support and arcs disabled. Keep the configured 0.4 mm Z-hop; change only `disable_m73` to isolate motion from progress ordering. Require the Native reference to travel directly in XYZ to the first object layer after the raft interface, with no restore Z before extrusion. Compare the complete emitted G-code through the existing ordered comparator in both Native modes. Run the existing support, raft, travel, lift, Z-hop and layer parity cases on both revisions to detect regressions.
