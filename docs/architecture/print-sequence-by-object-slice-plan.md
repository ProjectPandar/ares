# print_sequence=by_object: port slice plan (S1–S4)

Date: 2026-10-08. Worker5. Plan for review — **no production code in
this submission**. Companion to the inventory
([print-sequence-by-object-inventory.md](print-sequence-by-object-inventory.md)).

Ordering principle: upstream dependency order. Every slice is a port of
named upstream functions/lines; sequential printing must reproduce the
upstream structure, never an Ares-designed pipeline. Each slice lands
with its RED committed first and failing on the native comparison, then
the port turns it PASS.

## S1 — per-object layer collection and the by-object emission loop

Upstream (the port boundary):

- `GCode::collect_layers_to_print(const PrintObject&)` —
  GCode.cpp:1721 (per-object layer collection; the print-wide overload
  GCode.cpp:1838-1853 stays for by-layer).
- The by-object branch guard `print_sequence == ByObject &&
  !has_wipe_tower` — GCode.cpp:3284 (the wipe-tower fallback is part of
  the port, not an omission).
- Per-object `ToolOrdering(object, initial_extruder_id)` +
  `sort_and_build_data` rebuild on object change — GCode.cpp:3289-3297;
  the process-side twin Print.cpp:2481, 2516-2518.
- The per-instance emission loop with
  `process_layers(print, tool_ordering, collect_layers_to_print(object),
  instance_index, file, prime_extruder)` — GCode.cpp:3285-3346.
- `; total layer number` header via the
  `Total_Layer_Number_Placeholder` tag — GCode.cpp:2582-2583.

Ares target modules:

- `project_slice/gcode_emit/layers/schedule.rs` — a per-object schedule
  variant alongside the global `tool_ordering::build`.
- `project_slice/gcode_emit/layers.rs` — the outer per-instance loop
  driving the existing per-layer machinery per object.
- `gcode_file_start.rs:57` — `total_layer_count` becomes the sequential
  sum (Σ per-object layer counts).

Include: the loop, the per-object collector, the per-object ToolOrdering
rebuild, the header count. Defer: S2/S3 content inside the loop body
(emit `printing_by_object_gcode` only if it is non-empty; Ares fixtures
leave it empty), the wipe-tower interplay (fixtures run without the
tower; the `!has_wipe_tower` guard is asserted, not exercised).

RED: commit the two-object by-object E2E (Ender-3 0.4 + Kobra 3 0.4, the
probe recipe from the inventory) asserting `; total layer number:` =
2×N and the whole-file native comparison; it fails today at line 3
(100/124 vs 50/62).

## S2 — inter-object transition block

Upstream:

- `printing_by_object_gcode` evaluation + per-object first-layer
  bed/extruder temperatures — GCode.cpp:3334-3339
  (PrintConfig.cpp:2037).
- `current_object_idx` placeholder set from `finished_objects` —
  GCode.cpp:3333; `++finished_objects` — GCode.cpp:3361.
- `m_cooling_buffer->reset(position)` + `set_current_extruder` per
  object — GCode.cpp:3341.
- Between-objects travel: `travel_to_z(first_layer_print_height +
  z_offset, "Move to the first layer height")` — GCode.cpp:3367.

Ares target modules: `project_slice/gcode_emit/layers.rs` (the loop-body
transition), `gcode_placeholders.rs` (the `current_object_idx`
placeholder), the cooling-buffer state reset in the emit state machine.

Include: the four items above. Defer: `m_second_layer_things_done`
per-object reset interactions beyond what the comparison demands.

RED: the same two-object fixture with a non-empty
`printing_by_object_gcode` (e.g. `;OBJECT_START {current_object_idx}`)
and `between-object` travel lines asserted at the object boundaries;
fails today (nothing is emitted between objects).

## S3 — per-object most-used extruder and flush statistics

Upstream:

- `tool_ordering.cal_most_used_extruder(print.config())` per object —
  GCode.cpp:3342 (the count runs over the OBJECT's ordering, replacing
  the global count consumed by the placeholders).
- The flush statistics accumulation — GCode.cpp:3347-3355
  (`stats_by_single_extruder`, `stats_by_multi_extruder_best/curr`,
  `m_sorted_layer_filaments`).

Ares target modules: `tool_ordering.rs` (`most_used_extruder` re-run per
object — the deferred TiQ2 item), the flush-statistics plumbing
(`first_filaments.rs`/the statistics surface), consumed by the existing
placeholder sites.

Include: the per-object most-used consumed by the M73/timelapse
placeholders; the flush statistics recording. Defer: any statistics
presentation that only surfaces outside the compared G-code.

RED: a two-object, two-filament by_object case where the per-object
argmax differs from the global argmax (object A mostly filament 1,
object B mostly filament 2); the M9711/M73 placeholder must follow each
object's own count. Fails today (one global count).

## S4 — Print::validate sequential clearance checks

Upstream:

- `Print::sequential_print_clearance_valid` (horizontal clearance,
  extruder_clearance_radius) — Print.cpp:613-840.
- The vertical clearance check — Print.cpp:841ff.
- `Print::validate` integration and the rejection report (the object
  pair / height) — Print.cpp:1298.

Ares target modules: `print/validate/sequential_clearance.rs` (new
sibling of bed_filament/extrusion_width/identity), wired into the
validated entry orchestration.

Include: both clearance checks and the validate integration with the
native-matching rejection family. Defer: nothing inside the two checks;
any sub-check unreachable by stock presets is recorded, not invented.

RED: two overlapping/too-close objects with `print_sequence="by
object"` → native rejects (`Print::validate` family); Ares must reject
through the typed boundary with the same family, and must not reject a
passing arrangement. Fails today (no detector).

## Sequencing and gates

S1 → S2 → S3 → S4. S2/S3 assume the S1 loop; S4 is independent of the
loop but lands last so its RED uses the already-porting mode. Every
slice: source-cited spec note in this file, RED first (failing on the
native comparison), fresh seeded base/cand before merge, clippy
`-D warnings`, fmt, and the review gates. All fixture machinery already
exists (`export_case_multi`).
