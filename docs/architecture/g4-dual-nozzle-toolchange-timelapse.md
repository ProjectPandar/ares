# G4 dual-nozzle traditional timelapse at logical tool changes

Status: bounded four-map whole-output/consumer GREEN. Scope excludes the other
five G4 keys. The source ownership and deferred behavior below remain binding;
verification and the Native-only ordering observation are in `g4-manual-map-review.md`.
This is a `libslic3r/GCode.cpp` lifecycle rewrite, not a new G-code pipeline.
The original six-key harness specification remains
[the G4 coverage boundary](option-g4-tool-multifilament-coverage.md).

## Owning upstream boundary

OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `GCode/ToolOrdering.cpp:1275-1312`: retain Manual maps and convert their
  one-based physical nozzle IDs to zero-based IDs; no automatic reassignment.
- `ToolOrdering.cpp:955-981`: most-used physical nozzle counts distinct
  appearances per layer. `Print.cpp:3282-3286` and `GCode.cpp:3649-3656` map
  each logical filament to its physical nozzle.
- `GCode.cpp:4645-4650`: traditional insertion is enabled by configured nozzle
  count, not the number of distinct used nozzles or logical filaments.
- `GCode.cpp:5289-5299`: without a wipe tower, before `set_extruder`, a pending
  logical tool change on a two-nozzle printer inserts the traditional block
  when the current filament maps to the most-used physical nozzle. Order is
  ordinary eager retract, object-label flush, template render/position tracking,
  then mark inserted. `:5308` calls `set_extruder` afterwards.
- `GCode.cpp:5145-5202`: `insert_timelapse_gcode` renders the template and marks
  position unclear, adopting its last Z as appropriate.
- `GCode.cpp:5478-5494`: per-instance completion only updates object-stop
  labels. It does not retract. `ObjectLabels::queue_stop` must not preempt the
  later tool-change or layer-tail owner with an eager lift.
- `GCode.cpp:5255-5258,7331-7346`: by-layer exclude-object +
  `support_object_skip_flush` derives a 64-bit per-tool mask from the actual
  instances in `filament_to_print_instances`, locating IDs in the sorted unique
  label list and base64-encoding its little-endian bytes. The protocol's fewer
  than64 label restriction is not a new Ares allocation/pipeline policy.
- `GCode.cpp:7775-7779,8037-8041`: a nonempty old filament-end invocation
  consumes the pending mask as M624; the corresponding nonempty new start
  invocation closes it with M625. Neither command is a per-instance start/stop
  marker; empty templates do not manufacture invocations or consume a mask.
- `GCode.cpp:5527-5546`: an already inserted block skips the entire tail
  retract/label/template sequence; it does not merely skip labels.
- `GCode.cpp:7710-7714`: `need_toolchange(new_filament_id)` uses logical identity.
  Manual `[1,1]` therefore still changes logical0 to1 on physical0, with T1
  and the second filament's temperature. It must not suppress that switch.

## Rust boundary and included behavior

The destination is the existing `gcode_emit/tool_transition.rs` emission seam
and `gcode_emit/emit_helpers.rs` layer tail. The former currently flushes labels
and sets the inserted flag without rendering/tracking the block; the latter
still renders at the tail when that flag is set. Its `map.len()==2` predicate
also confuses logical filament count with configured nozzle count.

Replace that incomplete compatibility shell with the upstream pre-change
retract/labels/render-track lifecycle, using the existing timelapse context and
writer bookkeeping. Skip the complete tail when already inserted. Do not
change speed, time estimation/formatting, logical/physical IDs, retraction
magnitudes, template language, cache behavior or comparison normalization.
The source tower branch `GCode.cpp:5264-5277` is deferred unless independently
admitted; `Print.cpp:3443-3448` tower map wiring is inactive in this reproducer.

The destination also includes `object.rs`, `layers/layer_end.rs`, the existing
per-tool bucket iteration and the transition's filament-template seam. Replace
`queue_stop`'s premature `end_layer_for_timelapse`/defer call with label-only
queuing; remove that obsolete parameter from its callers. The actual final
layer owner retracts once before labels/template, without depending on whether
an object label exists. The existing raft final-entry tail remains its own
retraction owner; do not introduce retracts for intermediate tool entries.

Port pending `m_filament_instances_code` in the label state, deriving each
by-layer tool group's mask from actual source-instance membership. Reuse the
existing64-bit little-endian base64 encoder; do not hardcode AQ, expand the mask
from configured filaments, reuse the previous object's mask or disable skip
flush. Add M624 only before an invoked old end template, consume its pending
mask, and add the paired M625 only after an invoked new start template. Startup
without an old filament must not consume the mask. Source `:3312-3313` by-object
mask production remains deferred; the by-layer consumer rewrite is not a claim
that the independent sequential-print path is ported.

Failure modes to exclude before implementation: double eager lifts from both
instance completion and process-layer insertion; missing tail retract when
labels exist; premature labels before retract; rendering a second timelapse at
the tail; wrong logical-to-physical selection; wrong per-tool membership or
copy identity in the mask; mask consumption on empty templates; an unmatched
M625 or hardcoded single-instance mask. Existing legal whole-output RED covers
the active timelapse and mask invocation witnesses; full controls must reveal
any other changed stream. Do not edit `start_travel`/`restate` or retained/O6
owners as part of this slice.

## Legal E2E boundary and retained original

Use the G4 H2D cube10 Manual-map tracer, with both 0.4mm nozzles, explicit
`nozzle_volume_type=[Standard,Standard]`, coherent printer variant/retraction
payloads, two PLA logical filaments, wall1 and sparse/solid2, no tower/support/
raft. The real Native stream uses `; FEATURE:` vocabulary; observe wall/sparse
XY extrusion with that vocabulary. Configuration comments alone are not proof.
The production RED is complete ordered Native/Ares output equality, not a
filtered timelapse fragment. All four coherent Manual maps remain required by
the mapping domain. A mapping metadata RED is separate from production RED.

The first admission had two nozzles but `nozzle_volume_type=[Standard]`.
Its original input SHA256 is
`c5f3aed4318e10409c91092e069f5a4b7af0db6f14741b10938b6c7b69fa06ac`.
This input reaches the existing
[O8 UB_ADJACENT selector boundary](o8-x2d-scalar-flush-context.md):
`OrcaSlicer.cpp:3351` triggers reconstruction from old/new type-vector shape,
then `:3457-3463` reads missing raw selector1. Native's observed reconstructed
377/659 cells versus Ares's retained140 cells are not defined calculator parity.
Preserve this attempt and the prior missing-plate-metadata harness failure;
neither is silently replaced or called fixed. The additional complete-selector
context is a new legal witness, not a repair of the preserved original.
No O8 dependency guard is removed and no Native UB result is emulated.

## Verification and deferred scope

Commit legal-input whole-output RED before production correction. Verify the
pre-change timelapse/label order, actual logical tool/nozzle/temperature use and
complete output after the rewrite. If another first difference remains, report
it rather than widening the production scope or changing expectations.

Production changes require new-tree default-environment workspace, strict
clippy, formatting and core/adapter WASM checks. Fresh complete seeded and
official base/candidate controls require all seeded cases PASS and exact-name
official failures a subset of base failures. Inventory every common input's
Ares raw SHA-set change, with distinct captures/counts separated; no full
matrix or arbitrary project parity claim follows. Whole-byte comparison permits
only generated-by-banner normalization. Native input/streams/cache evidence
remain runtime artifacts, not committed goldens.

Deferred: tower insertion path; every automatic grouping strategy; dynamic
mapping; other G4 key contexts not yet independently executed; ramming,
calculator UB-adjacent shapes, speed/Flow changes and unrelated Native crashes.
