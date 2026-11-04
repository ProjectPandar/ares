# SPEC: BBL global label inventory and per-layer skip envelopes

Date: 2026-10-23. Status: source-cited SPEC.
No implementation, new tests, fixture modifications, or slicing runs in this slice yet.
Frozen print_order/retract branches are not dependencies and remain untouched.

## 1. Witness and precise diagnosis

Witness input SHA256 (from the Type1 tower-disabled public RED):
`e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28`.
Settings: stock Bambu Lab X1 Carbon 0.4 nozzle, two printable build items,
`enable_prime_tower=0`, `print_sequence=by layer`, `print_order=default`,
`timelapse_type=0`, stock nonempty `time_lapse_gcode`, `gcode_label_objects=1`,
`exclude_object=0`, `support_object_skip_flush=0`. Model metadata retains
`identify_id` values 45 and 56.

Stored Native double replay succeeds. API and CLI outputs
are identical to each other; both fail the strict comparator at normalized byte
223, line 6: expected `; model label id: 45,56`, actual `; model label id: 45`.
This is stored evidence, not a fresh reproduction at this branch's base.

| Stored stream | Native | Ares API / CLI |
|---|---:|---:|
| header label inventory | 45,56 | 45 |
| layer start envelopes | 50 | 0 |
| layer end envelopes | 50 | 0 |
| M624 lines | 150 | 100 |
| M625 lines | 150 | 100 |
| individual instance IDs | 45 and 56 | 45 and 56 |

Native SHA256: `fc83ef92f7f1d91f751d16daf109c6ef2a82f1f2817097bcc565aea950fa5358`.
Ares API and CLI SHA256:
`ee4ce1f4691b9391468746c29614fdf91f1e73d6363743bfc9e85a26b3032f10`.

The second ID is **not lost in the parser or individual-object emission**:
Ares already emits ID 56's individual start/stop blocks with mask
`AgAAAAAAAAA=`. Missing are the complete header inventory and layer-set
skip envelopes (`AwAAAAAAAAA=` = bits 0 and 1). Diagnostic isolation replacing
the header list and removing exactly Native's 50 start/end envelopes yields
otherwise byte-identical streams after generator normalization. This is an
ownership-localization experiment only, never an acceptance mask.

## 2. Upstream owner and exact included behaviour

All source paths below are under `OrcaSlicer/src/libslic3r`.

### 2.1 Global inventory: GCode::do_export / m_label_objects_ids

- `GCode.hpp:567`: `std::vector<size_t> m_label_objects_ids`.
- `GCode.cpp:2565-2601`: header suppressed for BTT_TFT thumbnails; inside the
  BBL header and `Calib_None` branch, enable exclude-object output, clear and
  reserve the inventory, iterate **Print.objects × PrintObject.instances**,
  collect `ModelInstance::get_labeled_id()`, sort, and emit the full comma list.
  Calibration clears the inventory and disables exclusion.
- The global inventory is sorted but **not deduplicated**. Collect only the
  active print's instances, not every arbitrary project instance.

This inventory owns bit positions for every consumer; header text must not be
repaired independently with a second, potentially divergent inventory.

### 2.2 Layer membership and envelope: GCode::process_layer

- `GCode.cpp:5106-5135`: construct live `filament_to_print_instances` using
  existing ordering/instance selection. This slice does not change selection.
- `GCode.cpp:5139-5144`: collect `instance.label_object_id` from **all live
  filament instance lists** into `std::set<size_t> layer_object_label_ids`.
  This set is sorted and unique, unlike the global inventory.
- `GCode.cpp:5147-5203`: `insert_timelapse_gcode` lambda renders the existing
  template and adds a skip envelope when BBL, total print instance count >1
  and <= `g_max_label_object` (64, `GCode.cpp:95`), membership nonempty, and
  `Calib_None`. Neither prime-tower enablement, `exclude_object`, nor
  `support_object_skip_flush` gates this envelope.
- Exact envelope spelling is observable, including the missing space in the
  upstream end comment:

  ```gcode
  ; object ids of layer N start: 45,56
  M624 AwAAAAAAAAA=
  <existing timelapse text, if any>
  ; object ids of this layerN end: 45,56
  M625
  ```

- An empty template does **not** suppress this layer envelope: the membership
  gate is outside the nonempty-template branch. Preserve that separation.
- Existing lambda insertion sites `GCode.cpp:5209,5276,5298,5459,5546` determine
  timing. Reuse corresponding Rust call paths; do not add a second unconditional
  layer emission or alter retract/toolchange/timelapse scheduling.
- Writer position tracking for rendered template content (`:5171-5178`) happens
  before envelope construction. Mask/comment-only output must not incorrectly
  invalidate position or become a synthetic template motion.

### 2.3 Shared mask encoding, not invented numeric IDs

- `GCode.cpp:7318-7330` `encodeBase64(uint64_t)`: explicitly serialize bytes
  `(value >> (8*i)) & 0xff`, despite the nearby misleading big-endian comment.
- `GCode.cpp:7331-7347` `_encode_label_ids_to_base64`: locate each ID with
  lower_bound in the global sorted inventory; bit index is that position,
  then encode the eight bytes. Unknown IDs/zero mask are upstream logic errors.
- Individual queued-object blocks at `:5354-5359` / `:5477-5485`, and existing
  per-filament flush masks, consume the same vocabulary. Preserve their timing
  and bytes; do not synthesize a different ID for a source object/copy.
- Upstream layer admission allows <=64 instances while encoder asserts <64
  inventory entries. Do not silently invent a resolution: the exact-64 edge
  must be investigated against Native before claiming support. Initial RED
  does not exercise that edge.

## 3. Rust destination boundary and scaffold replacement

Under `crates/ares-core/src/project_slice/gcode_emit/`:

1. `header.rs::append_header` currently reads
   `project.objects().first().instances().first()` and prints one `label_id`.
   Replace this single-instance scaffold with the upstream active-print
   inventory; preserve header/BTT/BBL gates.
2. `layers/instances.rs::collect` already maps source/transform groups to
   printable copies and retains each loaded label ID. `layers/schedule.rs`
   owns these instances before header emission. Use that active-print domain
   for inventory membership (including transform splits), not first-object
   assumptions or unrelated project instances.
3. `object/filament_instances.rs::FilamentInstances` already sorts an inventory
   and encodes masks using the existing base64 function. Deepen or rename this
   module to reflect the upstream global label-inventory owner; do not build
   separate header, per-layer and per-tool bit-index maps. Existing per-tool
   behaviour must remain unchanged. Its current construction is gated by
   `support_object_skip_flush` in `layers.rs`; the new global inventory must
   not inherit that unrelated per-filament optimization gate.
4. `object.rs::ObjectLabels::from_traversal` currently rebuilds a sorted,
   deduplicated project-instance inventory per object. Route individual masks
   through the same active-print inventory, avoiding an accidental ordinal
   mismatch. Deduplication policy differs upstream and must not leak into the
   shared inventory.
5. `layers/drive.rs` / `tool_buckets.rs`: collect sorted unique layer IDs from
   the **actual built per-tool instance entries**, mirroring `:5139-5144`.
   Do not approximate membership using all project objects, the first object,
   active tool alone, or geometry bounds. A label can be in several tool lists
   but appears once in the layer envelope.
6. `timelapse.rs` with existing `layers/timelapse.rs`, `layers/boundary.rs`,
   `layers/object_entry.rs` insertion paths: add envelope composition to the
   tracked insert path, not non-BBL untracked template rendering.
   `timelapse.rs::append` currently early-returns for empty template and has
   no membership context: separate optional template rendering/tracking from
   envelope output without changing template state semantics.

Keep all touched Rust files <400 LOC (split responsibility locally if necessary),
platform-neutral and free of filesystem/oracle/terminal code. The earlier
first-object header and per-object reconstructed maps are compatibility
scaffolds around the cited GCode ownership, not an Ares-owned skip protocol.

## 4. Included and deferred boundary

Included: normal slicing on BBL printers; active-print sorted global inventory;
header comma list; unique live layer membership; layer-level M624/M625 envelope
at existing insert sites; eight-byte global-index encoding; empty-template
separation; preservation of existing per-object and per-filament mask behaviour.
Prime-tower-disabled stock X1C is the first witness, not a new tower pipeline.

Deferred: Type1 wipe-tower generation/export, toolchange/flush scheduling,
print_order/retract work, new geometry/label allocation, GUI object management,
non-BBL exclusion redesign, browser/UI, and independent calibration workflow
support. Inspect existing calibration representation before implementation;
if the public API exposes calibration, preserve upstream Calib_None gate rather
than pretending ordinary project settings represent a calibration job. Exact-64
encoder discrepancy needs separate Native characterization before coverage claim.
No new fallback, feature flag, marker mask, or stored Native G-code golden.

## 5. RED-first plan

1. Reproduce the supplied same input at exact base with canonical official and
   seeded environment; confirm Native double replay and public API/CLI difference.
   Retain full logs/exit status, input SHA, normalized first difference.
2. Freeze witness 3MF+SHA and add `native_stream_case::check` coverage using the
   current `(bool, callback)` adapter. Strict comparator remains unchanged.
3. Controls: GUI-reachable one-object BBL (no layer envelope but correct header),
   compatible/non-BBL, and multi-object BBL with empty timelapse template (envelope
   still required). Add a differing-height/layer-membership case to prove the
   mask tracks live layer entries rather than a static all-object mask. Distinct
   nonconsecutive IDs and transform/copy membership must remain observable.
4. Before GREEN, document failure cases: omitted second ID, incorrect ordinal bit,
   stale IDs after an object ends, duplicate layer IDs across filaments, missing
   empty-template envelopes, extra single-object/non-BBL envelopes, misplaced
   blocks or writer-position changes. Use source-cited public stream tests first;
   do not write isolated unit tests after implementation.
5. After RED, port the shared inventory and membership/envelope slice;
   run directed cases in both modes and neighbour/FULL gates. FULL only through
   the canonical parity control-set runner. No ad-hoc oracle environment or disabled requirements.

SPEC completion does not claim a fresh baseline reproduction, RED submission,
GREEN implementation, or parity beyond the supplied stored witness.
