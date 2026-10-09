# TiQ2 print-end filament template dispatch

## Upstream owner and Rust destination

The rewrite boundary is `libslic3r/GCode.cpp:3477-3493`, the print-end
`filament_end_gcode` dispatch followed by `machine_end_gcode`, together with
`placeholder_parser_process` (:3894-3919) for writer-state refresh and template
write-back. The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/finish.rs`, `append` and its
existing template invocation context. Existing `first()` dispatch is a
non-parity scaffold to replace, not a compatibility fallback.

## Source conclusion

Upstream branches on `single_extruder_multi_material`:

- True: render only the writer's active filament slot, selected with
  `filament_end_gcode.get_at(active_id)`, and set `filament_extruder_id` to
  that active ID.
- False: iterate every configured `filament_end_gcode` value in vector order,
  including configured slots not used in extrusion. Set
  `filament_extruder_id` to the slot index and invoke the parser with that
  index. Do not change the writer's active filament merely to render another
  configured template.

Each parser invocation refreshes live writer position/E state and applies
successful assignment output back to the writer. Machine-end follows the
filament templates with the writer's active filament as parser index.
`current_extruder` remains the active writer/parser placeholder, not the
loop index. The third parser argument becomes `ctx->current_extruder_id`
(`PlaceholderParser.cpp:827/915`); it selects implicit vector subscripts and
also participates in filament_map/extrusion_width resolution (:841/:1052).
It must use the loop index even though `current_extruder` stays writer-active.
The RED includes `[nozzle_temperature]` with distinct slot values to separate
these concepts. This distinction matters for the TiQ2 scripts: both slot templates
execute their `current_extruder==0` branch even while the second invocation's
`filament_extruder_id` is1.

Rust currently renders only `.first()` regardless of SEMM. Its writer-state
refresh also seeds `filament_extruder_id` from active ID, so per-template
slot override must be applied after refresh rather than accidentally erased.
Changes must remain local to print-end dispatch/context; do not globally
redefine writer-state placeholder semantics for unrelated callers.

## Observed complete-byte residual

The committed TiQ2 input has SEMM0 and two configured identical filament-end
scripts. After the approved travel correction, all bytes agree through33323
lines. Native renders the first script at33314-33323 and the second at
33324-33333, then machine-end begins33334. Rust emits the matching first
script, then starts machine-end at33324 (`G1 X-19 F3000 ; home X axis`), omitting
the second invocation. These offsets identify runtime evidence, not equality
assertions. The strict runtime comparator remains local; Native output is
not checked in. Complete TiQ2 parity beyond this first residual is unproven.

## Included and deferred behavior

Include the source dispatch distinction, vector order, active-slot SEMM
selection, per-slot `filament_extruder_id`, preservation of active
`current_extruder`, and live writer-state refresh/write-back between invocations.
Preserve existing template error reporting, blank-line/formatting behavior and
machine-end execution. A configured unused slot is not skipped simply because
no extrusion used it. No output deduplication or banner-external normalization
is introduced.

Defer unrelated start/toolchange template dispatch, statistics/footer timing,
other placeholder-language gaps and any later complete-output residual.
No new pipeline abstraction or generalized template framework is needed.

## Runtime verification conclusion

The independent runtime RED (`4f9e63b2f` before rebase) confirms Native
footer records with writer `current_extruder=0` for both invocations,
`filament_extruder_id=0/1`, and implicit `[nozzle_temperature]=211/233`.
Ares initially emits only slot0 and the machine template, omitting slot1.
The local dispatch/context correction makes those same records equal.

Additional runtime Native controls pass for a nonzero active SEMM filament,
an unused configured filament template and writer-position assignment flowing
from slot0 to slot1 to machine-end. The first SEMM attempt selected slot0 in
both implementations; that was a fixture failure, not a production RED.
The corrected input assigns wall/infill/support to slot2 and disables the
tower; it retains the strict Native slot1/temperature233 assertions.

The original committed TiQ2 input now passes the complete-byte runtime
comparator, with generated-by banner normalization only. No footer filtering,
whitespace relaxation or output deduplication is used. This establishes the
observed whole-file result for this input, not parity for arbitrary projects.
Fresh full default and both paired reference gates pass; complete equality
is verified for this input, not arbitrary projects.

## Loaded-filament precondition

The default workspace exposed a missing upstream prerequisite: the public
profile entry with omitted filament G-code keys reached an empty vector.
The overrides case already failed before applying its layer-height override.
The independent public `profile_settings` RED confirms the composed value is
not the required single empty-string slot. This is not a footer fallback case:
`ConfigOptionVector::get_at` asserts nonempty (`Config.hpp:624-628`).

`OrcaSlicer.cpp:588-605`, `load_default_gcodes_to_config(TYPE_FILAMENT)`,
resizes empty filament-start/end vectors to one empty string after option
creation. Rust `profiles/catalog.rs` created the registry defaults but omitted
that resize. The corrected load boundary decodes these two string-vector
values and resizes only empty vectors, preserving nonempty values. Footer
dispatch keeps its nonempty precondition; no empty-string return is invented.
The public composition and slice-override REDs both pass after this correction.

## Final verification boundary

Fresh full paired controls on `d491b01b5` pass for both Native modes:
seeded base863/863 and candidate868/868; official base863/863 and
candidate868/868. The only five name-level additions are the four footer
semantic controls and the strict original TiQ2 whole-file comparator. No
baseline test is lost, and both failure sets are empty in each mode.
The required environment uses the current v2 control script. Official
seeded-only references retain their visible SKIPPED contract; a nextest
PASS for such a wrapper is not a claim of Native execution.

Default-environment workspace on the same production tree passes8239/8239,
with5 nextest skips and visible optional-Native SKIPPED notices. All-targets,
all-features workspace clippy with warnings denied, core/adapter WASM check
and formatting pass. Subsequent main updates through `1b20d0d08` are
approved test/document-only carry-forward; rebasing preserves their active
roadmap records and does not restore deleted history.

The original dispatch RED and the independent public loaded-slot RED remain
in history before their production corrections. Earlier interrupted or failing
gates do not stand in for these fresh complete results. Native streams and
comparison receipts remain runtime artifacts, not checked-in goldens.

