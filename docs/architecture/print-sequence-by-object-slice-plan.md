# print_sequence=by_object: port slice plan (S1–S3)

Date: 2026-10-08. Worker5. Plan for review — **no production code in
this submission**. Companion to the inventory
([print-sequence-by-object-inventory.md](print-sequence-by-object-inventory.md)).

Scope note: the sequential clearance validations (inventory items 16-18,
`Print::sequential_print_clearance_valid` Print.cpp:613-840, the vertical
check Print.cpp:841ff, the validate integration Print.cpp:1298) are
**assigned to worker4** and are not part of this plan.

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

## Not in this plan

S4 (the `Print::validate` sequential clearance checks, inventory items
16-18) — **owner: worker4**; this plan neither scopes nor schedules it.

## Sequencing and gates

S1 → S2 → S3 (S4 runs under worker4 independently). S2/S3 assume the S1
loop. Every slice: source-cited spec note in this file, RED first
(failing on the native comparison), fresh seeded base/cand before merge,
clippy `-D warnings`, fmt, and the review gates. All fixture machinery
already exists (`export_case_multi`).

## S1 implementation map (staged, not yet landed)

The Ares emit is one global Z-group loop (`layers.rs:144` over
`schedule.merged`); every chunk already carries `object_index`
(`layers/schedule.rs:14`) and `schedule.object_layer_counts` exists. The
S1 restructure, in dependency order:

1. Scope the schedule per object: a filtered `merged` (the chunks whose
   entries' `object_index` match, Z-ordered), a per-object
   `ToolOrdering` — `tool_ordering::build` restricted to that object's
   layers with `final_extruder_id` seeded (the upstream
   `ToolOrdering(object, final_extruder_id)` port, NOT a global-ordering
   filter), and `layer_count` = the object's count.
2. Parameterize the loop body (the `while group_start < merged.len()`
   section) over the scoped schedule; the by-object driver calls it once
   per instance in `sort_object_instances_by_model_order` order and
   concatenates the output; the by-layer path keeps today's single call.
3. Cross-object state that carries over the concatenation:
   `max_layer_z` (running max), `second_layer_done`, the fan mover span
   (constructed once outside the loop), the writer state; per-object
   resets mirror GCode.cpp:3341 (cooling reset) — recorded, S2 lands the
   templates.
4. The `-1` skip: an object whose ordering yields no first extruder is
   skipped entirely (GCode.cpp:3293-3296).
5. `total_layer_count` (`gcode_file_start.rs:57`) sums the per-object
   counts in by-object mode (the RED's line-3 assertion).
6. The `!has_wipe_tower` guard (GCode.cpp:3284): with a tower, keep
   today's by-layer path (assert the guard, do not exercise it — the
   fixtures run towerless).

Wiring notes (observed, for the driver author):

- `schedule.merged` is a flat `Vec<(f64, LayerToPrint)>`; the loop groups
  equal-z runs (`layers.rs:144-150`). The per-object view is
  `merged.iter().filter(|(_, ltp)| ltp.object_index == object)` (the
  chunk order already carries the z order).
- The by-object gate:
  `traversal.resolved.views.full.process.print.print_sequence ==
  ProcessPrintSequence::ByObject && schedule.tower.is_none()`
  (GCode.cpp:3284 `!has_wipe_tower`).
- Loop locals that must survive the parameterization (either as params
  or by keeping one loop with a drive plan):
  `cooling`, `spiral`, `entry_builder`, `raft_layers`, `max_layer_z`,
  `layer_change_template`, `timelapse_gate`, `second_layer_done`,
  `chunk_index`, `final_chunk`, `last_object_copy`, `previous_layer_z`,
  `first_group`, `state.most_used_extruder` (per-object:
  `tool_ordering.most_used_extruder(traversal)` on the scoped build).
- How the loop consumes the ordering: the tool picks go through the
  `tool_ordering` binding (`layers.rs:118` destructure); the driver must
  swap it for the scoped build per object (the z-keyed lookups then hit
  the object's own layers).
- The last extruder for the seed chain (`ToolOrdering.cpp:932-935`):
  walk the scoped layers backwards to the first non-empty `extruders`
  and take `.back()`; add `ToolOrdering::last_extruder()` in
  `statistics` alongside `collect` if absent.

Known interactions to verify during implementation: the avoid-crossing
boundary cache (built per slice_z across every object today — in
by-object mode upstream uses the object's own slices), the raft chunk
interleave (first-layer only), the skirt/brim emission point (upstream
keeps them at the true first layer), and `state.most_used_extruder`
(S3 replaces the global count; until then the global value stays).

## S1 landed state (RED advanced to the S2 boundary)

The by-object pass is wired: `OrderingScope::Object` builds the scoped
ordering (own z heights, seeded dontcare, no custom ranges —
ToolOrdering.cpp:417), `layers.rs` drives the same per-layer machinery
once per object over the object's chunk filter, and the header sums the
per-object layer counts. Guarded suites pass unchanged; both REDs moved
from the header (line 3) into the inter-object transition:

- ender3: first difference at byte 76612 (line 3603), ares −68 bytes;
- kobra3: first difference at byte 113703 (line 4261), ares −353 bytes.

The missing block at each object boundary (after `stop printing object`):
the end-of-object wipe, the safety Z lift (10.4→10.8), the cross-object
download travel (`G1 X… Y… Z10.8`), and the `M104` temperature switch —
exactly the S2 slice (GCode.cpp:3334-3367: the temperature, the current
object index, the cooling reset, `travel_to_z`). The ares side instead
emits the next object's layer-change template with a bogus negative
HEIGHT (previous_layer_z carried over the boundary). S2 owns this; the
S1 REDs stay red at the recorded boundary, assertions unchanged.

One pre-existing quirk surfaced: the first layer's region options carry
raw one-based filament ids that the global pass silently rewrites via
`handle_dontcare`; the scoped seed path now applies the same rewrite
before leading with the seed (recorded above, in build_inner).
