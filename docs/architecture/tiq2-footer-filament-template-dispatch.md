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
loop index. This distinction matters for the TiQ2 scripts: both slot templates
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

## Test boundary before production

Restore the complete-byte TiQ2 runtime-OrcaRunner RED on merged main. Use the
shared Native environment contract; missing Native must fail when required,
and optional absence must produce visible SKIPPED evidence.

Add a runtime Native E2E with distinguishable filament-end scripts so the
assertions distinguish slot order, inclusion of an unused configured slot,
`current_extruder` versus `filament_extruder_id`, and machine-end ordering.
Include a SEMM control with a nonzero active filament to reject hardcoded
slot0 or unconditional all-slot iteration. Include writer-state assignment
between invocations to reject rendering every template from a stale snapshot.
All expectations come from runtime Native, not an Ares-generated golden.

Only after semantic RED, implement the cited dispatch and invocation context,
then rerun strict complete-byte comparison, affected template/Type2/fan controls
and required paired/control/workspace/clippy/wasm/fmt gates. If another residual
remains, report it without filtering or claiming complete parity.
