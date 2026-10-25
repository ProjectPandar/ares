# ARD — spiral vase object-start Z-only line

Non-negotiable decisions (spec: `spiral-vase-object-start-z-spec.md`).

1. **Literal rewrite-all port.** `SpiralVase.cpp:136-141`: every Z-only
   G1 of a spiral-enabled layer is kept and rewritten to the base z with
   the 3-decimal `GCodeLine::set` splice. The
   `gcode_spiral_vase/project.rs::append_layer_z` `already_emitted`
   dedup is a deviation and is removed — not adjusted, not widened.
2. **Writer mechanism, not an insert.** The object-start Z-only line is
   the generic `m_need_change_layer_lift_z` forced Z-sync
   (`GCode.cpp:5708` + `:6367-6385`, `force=true` bypassing
   `will_move_z`). Any "if spiral, insert `G1 Z`" special-case outside
   the two mechanism sites (`append_layer_z`,
   `start_travel.rs:148-157`) is rejected.
3. **Conditional second site.** `start_travel.rs` is touched only if the
   RED bytes prove the line still missing after the filter fix, and only
   to align with `GCode.cpp:5708`/`:6367-6385` semantics (flag cleared
   only at the `_extrude` guard / `travel_to`'s 2-point path; line after
   the `M624` flush).
4. **Masked-producer clause.** The dedup may hide over-emission
   elsewhere. GREEN runs every `spiral_*` parity case in both Native
   modes; a regression is a STOP until root-caused to the real producer.
   Restoring the dedup (or any equivalent suppression) is forbidden.
5. **Native-first RED, unweakened.** Runtime Native official + seeded,
   double-replay determinism, complete comparator (`check`: API and CLI
   byte equality against Native), frozen input SHAs; no Native artifacts
   committed.
6. **E2E-only verification.** No unit tests; the RED cases are the sole
   verification.
7. **Boundary hygiene.** Only the two mechanism sites may change in
   GREEN; queued and W1-touched files are untouched.
