# print_sequence=by_object: port slice plan (S1–S3)

Companion to the inventory
([print-sequence-by-object-inventory.md](print-sequence-by-object-inventory.md)).

Scope note: the sequential clearance validations (inventory items 16-18,
`Print::sequential_print_clearance_valid` Print.cpp:613-840, the vertical
check Print.cpp:841ff, the validate integration Print.cpp:1298) are
not part of this plan.

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
16-18); this plan neither scopes nor schedules it.

## Sequencing and gates

S1 → S2 → S3 (S4 is independent). S2/S3 assume the S1
loop. Every slice: source-cited spec note in this file, RED first
(failing on the native comparison), fresh seeded base/cand before merge,
clippy `-D warnings` and fmt. All fixture machinery
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

## S2 spec

Upstream boundary: the per-instance transition inside the by-object
loop, `GCode.cpp:3300-3372`, between the ordering rebuild (:3289-3297)
and `process_layers` (:3349-3350). For every instance after the first
(`finished_objects > 0`, :3309), in order:

1. `set_origin(unscale(instance.shift))` (:3303) — already emitted per
   chunk in Ares (the build-item placement); verify once, no change.
2. Cooling markers off (:3308); `avoid_crossing_perimeters
   .use_external_mp_once()` (:3309).
3. The toolchange branch (:3310-3325): if
   `need_toolchange(initial_extruder_id)` → `set_extruder(..., true)`
   (prime) and `prime_extruder = true` for the instance's
   `process_layers`; otherwise `retract()`. The exclude-object label
   base64 rides on the toolchange when `support_object_skip_flush`
   (:3313-3317).
4. `travel_to_z(m_max_layer_z + z_hop[initial_extruder_id])` (:3326) —
   the safety lift (the native's `G1 Z10.4`: max_layer_z 10.0 + z_hop
   0.4). Ares has `max_layer_z` in the append locals.
5. `travel_to(Point(0,0), erNone, "move to origin position for next
   object")` (:3327); then `avoid_crossing_perimeters.disable_once()`
   (:3329-3330).
6. `current_object_idx = finished_objects` (:3333); the
   `printing_by_object_gcode` custom template processed with
   `initial_extruder_id` (:3334-3335); the first-layer bed and extruder
   temperatures (no wait) (:3337-3338 — the native's `M104 S220`); the
   template written (:3339).
7. The cooling buffer reset at the current position + the current
   extruder (:3341-3342).
8. `process_layers(..., prime_extruder)` (:3349-3350) — in Ares, the
   phase loop iteration.
9. After the instance: the flush statistics (:3352-3359), the PLR
   disable when `m_second_layer_things_done` (:3361-3367),
   `++finished_objects`, `m_second_layer_things_done = false`
   (:3369-3372).

Deferred (not S2): `cal_most_used_extruder` per object (:3346 — S3),
`m_printed_objects` (:3347), `m_sorted_layer_filaments` (:3356-3359),
the statistics accumulators (:3352-3355).

Ares mapping: the transition writes live in `layers.rs` between the
phase loop iterations (after the previous phase's last chunk, before
the next phase's first `boundary::append`). The state resets the
boundary needs:

- `previous_layer_z` resets per phase — the S1-introduced negative
  `;HEIGHT:` (-9.8) comes from carrying object 1's top z (10.0) into
  object 2's layer 1; the upstream recomputes the height inside each
  object's `process_layers` from the object's own layers, so the per-
  phase reset IS the upstream behavior, not a hack.
- `second_layer_done` resets per phase (:3372).
- The fan mover and the cooling state carry per the cooling-buffer
  reset semantics (:3341-3342).
- The transition block writes with the cooling markers off
  (:3308) — in Ares the cooling rewrite windows must not swallow the
  transition bytes (write them outside the `cooling.begin_layer`
  window).

Acceptance: both REDs reach PASS in full (the by-layer fixtures stay
byte-identical, proven by a fresh seeded base/cand run over the by-
layer suite); no assertion changes.

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

## Deferred: M73 duplicate-z dedup under sequential printing

Upstream `m_layer_count` computation deduplicates numerically adjacent
print-z values within an object before accumulating
(`GCode.cpp:2502-2504`: `if (abs(*it - *(it + 1)) < EPSILON)
temp_layer_count--`). The Ares tail sums raw per-object schedule entry
counts (`Σ object_layer_counts`), which assumes unique layer z per
object. Standard slicing never produces adjacent duplicate z inside one
object, so no current fixture exercises this branch; if a future fixture
does (stepped z, modifier stacks), add a RED case asserting the
deduplicated total before touching the count.

## Spec: by-object chunks per print-object instance

Under `print_sequence = by object`, upstream iterates print-object
INSTANCES, not objects (`GCode.cpp:3279-3302`:
`print_object_instance_sequential_active` walks
`print_object_instances_ordering`; per iteration it rebuilds
ToolOrdering on object change :3283-3297 and applies the instance shift
via `set_origin` :3302). Every instance after the first runs the full
transition: `if (finished_objects > 0)` gate (`GCode.cpp:3304`),
toolchange-or-retract (:3308-3320), hop + origin travel
(:3326-3327), placeholder/first-layer temps (:3333-3339);
`finished_objects` increments after the per-instance preamble. The M73
count follows the same granularity: `instances().size() *
temp_layer_count` per object (`GCode.cpp:2505`), surfaced both in the
`total_layer_count` placeholder (`GCode.cpp:2890`) and the compatible
tail (`GCode.cpp:3542`).

Ares gap: the schedule merges same-object instances into one chunk, so
the second copy prints without the inter-instance transition and the
header/tail count says 62 instead of 124.

Planned change:
1. Schedule: under by-object, keep one chunk per instance (per-instance
   layer list + per-instance transition boundary); by-layer grouping
   unchanged.
2. Counts: header placeholder and tail use Σ(instances × layers) per
   object; keep the by-layer merged-z count.
3. Acceptance: single_object_two_instances fixture GREEN
   byte-for-byte; ender3/kobra3 by-object stay GREEN; by-layer guards
   (wire_modes/cross_map/semm_ramming) unchanged.

### Implementation anchors for the per-instance split (located)

- `schedule.rs:223-229`: the group loop merges every entry within
  `minimum + EPSILON` — same-z entries of both instances collapse into
  one chunk.
- `schedule.rs:183`: global `sort_by(z)` interleaves the two instances'
  layer stacks; upstream prints instance-major (full z stack per
  instance, z restarts at the first layer after each transition).
- `schedule.rs:58 build` iterates `objects` per print object: for a
  one-object-two-instance project `merged` holds only 62 entries — the
  second instance's layers never enter the schedule (root cause of
  ares 62 vs native 124).
- Change shape: under by-object, expand `objects` to instances in
  `build` (instance order = print order; labels stay indexed by object;
  per-instance transition boundary at the chunk level mirrors
  `GCode.cpp:3304`), skip the global z-sort in favor of per-instance
  contiguous runs, and let the EPSILON group loop run within each run.
- M73: header placeholder + tail take Σ(instances × layers); the
  `.max(sliced_layer_count)` guard in `gcode_emit.rs` must be revisited
  so multi-object by-layer fixtures keep the merged-z count.

### Phases driver is the split point (located, implementation pending)

`layers.rs:171-200`: the by-object branch builds one `DrivePhase` per
print OBJECT (`for object in 0..objects.len()`, filtering `merged` by
`entry.object_index`), so a one-object-two-instance project yields one
phase and the copies collapse. The inter-phase transition machinery
already exists (`layers.rs:233+`, the `GCode.cpp:3309-3342` port), so
the fix is: iterate `(object, copy)` pairs instead of objects — reusing
the SAME ToolOrdering for the copies of one object (no rebuild,
`GCode.cpp:3286`). Open risk to resolve first: per-copy geometry
placement — `instance_origin` reads `instances.first()` only and
`last_object_copy` tracking must drive both the origin travel and the
extrusion coordinate offset per copy; the 62-entry `merged` slice is
shared by both copies and stays untouched.

### Refinement: the split needs per-copy path partitioning

The RED body diff is only -5164 bytes (the transition/wipe blocks), not
a missing half: Ares already prints BOTH copies' geometry inside every
layer (by-layer style interleaving), so per-instance chunks require
partitioning each layer entry's paths by copy (the two builds sit at
distinct plate positions), not merely regrouping chunks. Upstream keeps
the object's layers in local coordinates and re-homes them per instance
via `set_origin` (`GCode.cpp:3303`), printing copy 1's full stack before
copy 2's. The Ares traversal bakes plate coordinates into paths, so the
emission layer must either (a) partition traversal regions per copy, or
(b) emit local coordinates with a per-chunk origin offset. Decide (a)
vs (b) when implementing; the transition, M73 count, and phase machinery
findings above are unchanged.

### Emission-side anchor: per-instance set_origin already exists

`layers.rs:110-116` documents that each chunk already emits around its
own build-item placement (`set_origin(unscale(instance.shift))`,
`GCode.cpp:5380/5403/5437` port; copies share the layer layout, records
matched by slice_z), and `tool_entry.rs:92` applies
`bucket.instance.shift` per instance bucket. The per-instance split is
therefore: carry `copy` on `DrivePhase`, thread it into the chunk
emission, and filter the instance buckets to the phase's copy (the
62-entry merged slice stays shared). The inter-phase transition and the
M73 count findings above are unchanged.

### SPEC: 3MF loader multi-instance split (bbs_3mf.cpp:1996-2032, revised)

Upstream gates the load-time split on the producer tag:
`m_is_bbl_3mf` is set when the 3MF's Application metadata starts with
`BambuStudio-` or `OrcaSlicer-` (bbs_3mf.cpp:3947-3961); the split
("if the 3mf was not produced by OrcaSlicer and there is more than one
instance, split the object in as many objects as instances") runs only
when `!m_is_bbl_3mf` (bbs_3mf.cpp:1996-2032). BBL/Orca-produced 3MFs
keep 1 ModelObject x N instances and take the per-instance sequential
path (GCode.cpp:3279-3339); third-party 3MFs are split at load into
N ModelObjects x 1 instance.

Consequences:
- The hand-built two_instances fixture MUST carry the OrcaSlicer-
  Application metadata, otherwise the native takes the third-party
  split (2 ModelObjects x 1 instance) and the fixture never exercises
  the per-instance path it targets. The fixture helper now inserts
  `<metadata name="Application">OrcaSlicer-2.4.2</metadata>`.
- The ares loader split is a SEPARATE slice: port the third-party
  load-time split behind the same producer gate, verified with a
  no-metadata fixture (its acceptance = the native's split behavior:
  N ModelObjects x 1 instance, num_print_regions = N).
- The per-instance by-object phases work in this repo targets the
  Orca-produced shape (1 ModelObject x N instances) and stays as
  implemented in the WIP phases/bucket filter.

### REVISED root cause: the width block is per-instance under by-object

Evidence (two_instances fixture):
- The native output is byte-identical with and without the Application
  metadata (sha 325fb2c3..., 173177B) - the loader split premise is
  REFUTED; the native never took the third-party path.
- The two build items differ ONLY in XY (identity rotation/scale, both
  Z=5) -> per PrintApply.cpp:146-152 one PrintObject with two
  PrintInstances; the native markers are all "id:0 copy 0" (200 = 100
  layers x start/stop).
- The native top carries TWO width sets (per instance first layer);
  ares emits ONE (stream start). -237 bytes = exactly one width set.

Root cause: under by-object, upstream re-runs the slicing-parameter
block (GCode.cpp:2676-2693) at every instance's first layer; ares emits
it once per stream (the by-layer behavior). Fix direction: move/
duplicate the header width-block emission to each by-object phase's
first chunk. The earlier EPSILON and loader-split notes in this file
are superseded by this section; the bbs_3mf loader split remains a
separate gated slice for third-party 3MFs only.

### Final region-count analysis (evidence-backed)

- The two width sets carry IDENTICAL values (diff clean): two
  PrintRegions with the same config, not two different regions.
- The native markers are ALL "id:0 copy 0" (200 = 100 layers x
  start/stop): ONE PrintObject, two PrintInstances, 51 layers each
  (102 LAYER_CHANGE).
- The fixture 3MF defines TWO objects (id 2 unreferenced after the
  retarget, id 4 with both build items); model_settings.config shows
  4 name entries (both defined objects).

Conclusion: the shared PrintObjectRegions (ref-counted per
ModelObject, PrintApply.cpp:425/:434-436) accumulates all_regions per
INSTANCE apply (2 instances -> 2 identical region entries ->
num_print_regions() == 2), while the ares region table dedupes per
object (1 entry). Fix direction: the ares all_regions accumulation
must insert per instance apply (or m_print_regions must not dedupe
identical per-instance configs) - implemented in the ares
load/region-set path, verified by the two_instances width blocks
1 -> 2 and the full-file parity. The by-object phases and the skirt
per-group accounting stay as landed.

## S2 completion: the kobra3 by-object object-boundary blocks

The last failing fixture (kobra3) missed four byte groups at every
object boundary after the transition; all three are
`m_second_layer_things_done` / writer-cache ports, landed as one
ecommit each:

1. Per-object first/second-layer gating
   (`588ccf1f1`). `first_layer` is per-object
   (`layer.id()==0 && |bottom_z|<EPSILON`, GCode.cpp:4589, writer
   `m_is_first_layer` :4590), so after the per-object flag reset
   (GCode.cpp:3367) EVERY object re-runs the layer-0 block
   (initial acceleration/jerk + initial travel speed, :4762-4771)
   and the `!first_layer && !m_second_layer_things_done` block
   (:4777-4797: default acceleration/jerk, nozzle temps). Ares gated
   both on the global chunk index. `begin_layer` takes a `LayerBegin`
   struct; the layer-index advance stays with `advance_for_layer`
   (GCode.cpp:5690-5696).
2. Live writer bed-temperature cache (`cb84e3a07`, plus the
   zero-guard removal).
   `GCodeWriter::set_bed_temperature` refreshes
   `m_last_bed_temperature` on every changed value
   (GCodeWriter.cpp:168-175), so the object-2 transition
   (_print_first_layer_bed_temperature, GCode.cpp:3332) must compare
   against the value the object-1 second layer wrote (:4820). Ares
   passed the startup snapshot; Klipper startup leaves it 0, so ares
   re-emitted M140 S35 twice. The cache now threads as `&mut i32`
   through `LayerBoundary`/`Transition`. The follow-up (`2f7d9a990`)
   drops a non-upstream `bed_temperature > 0` guard on the transition
   write: `_print_first_layer_bed_temperature` always calls
   `set_bed_temperature` (GCode.cpp:4023-4050) and the writer emits
   M140 S0 against a nonzero cache — only the nozzle path keeps a
   `temp > 0` gate (GCode.cpp:4058-4064). Covered by the public-API
   RED/GREEN (`b5a485a84`): a two-instance ksr project with
   first-layer bed 0 / other layers 55 writes M140 S0 at the
   transition.
3. Transition travel acceleration (`e14d9d0ee`). The origin move
   runs through `GCode::travel_to` (GCode.cpp:3327 -> :7350), which
   selects the travel acceleration/jerk BEFORE the writer hop
   (erNone + the previous object's layer -> plain travel values). The
   ares transition wrote the second hop directly: the kobra3 output
   missed `SET_VELOCITY_LIMIT ACCEL=15000 ACCEL_TO_DECEL=7500
   SQUARE_CORNER_VELOCITY=12` between `G1 Z10.36` and `G1 Z10.76`,
   and the stale jerk cache suppressed the object-2 layer-0
   `SQUARE_CORNER_VELOCITY=9` line. Fixed by running
   `begin_path_travel` between the two `writer_z` hops.

Acceptance: ender3 / kobra3 / single_object_two_instances all PASS
(seeded, byte-exact); by-layer guards re-verified by the full seeded
control set.

Known deferred nuance: `begin_path_travel`'s first-layer branch keys
on `state.layer_index == 0` (global) while upstream uses
`on_first_layer()`; kobra3/ender3 set `initial_layer_travel_*` to 0
so no current fixture distinguishes them.
