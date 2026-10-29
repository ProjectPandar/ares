# BBL forced eager-layer retract: parent can_lift rewrite slice

## Stage and source boundary

COORD authorized SPEC → RED on exact main
`0266b34949a6f9503002a3420e1c526cb745462b`. This document is the SPEC commit;
tests/inputs follow separately. GREEN requires independent approval after RED.
No production change, FULL, rebase or integration is authorized by this stage.

Pinned OrcaSlicer source: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The owner is **libslic3r GCode::retract's parent can_lift predicate**, not an
Ares-owned lift scheduler or GCodeWriter policy redesign:

- `src/libslic3r/GCode.cpp:4657–4660`: BBL layer-start Auto/Spiral/Slope
  select SpiralLift for the eager call.
- `GCode.cpp:5205–5210::process_layer`: forced BBL layer-start calls
  `retract(false, false, auto_lift_type, true)` before timelapse insertion.
- `GCode.cpp:7676–7704::retract`: needs_lift plus can_lift gates the writer
  call; enforce is checked even when the nozzle was already retracted.
- `src/libslic3r/GCodeWriter.cpp:652–681::eager_lift`: live-height eligibility,
  Spiral/Normal emission and writer lifted/pending bookkeeping belong here,
  **not the parent's surface-enforcement policy**.
- `PrintConfig.cpp:5320–5335`, `GUI/Tab.cpp:5150`: existing On surfaces GUI
  option `retract_lift_enforce`, no hidden/private input activation.

## Exact Rust destination and non-negotiable semantics

Existing `crates/ares-core/src/project_slice/gcode_emit/layers/boundary.rs`
forwards the forced BBL layer-start operation to
`motion/travel.rs::flush_pending_retract_eager_forced` →
`flush_pending_retract_eager_with` → `motion/travel/lift.rs::append_eager`.
The forced caller's lift tail omits the parent can_lift check. The current
writer shell intentionally owns only writer-level conditions.

Port only that missing parent predicate at the forced caller's **eager-lift
invocation**. Reuse the existing `lift::enforce_allows_lift` semantics:

| Existing enum | Parent can_lift |
|---|---|
|AllSurfaces|true|
|BottomOnly|actual physical layer counter == 0|
|TopOnly|last non-gapfill role is TopSolidInfill or Ironing|
|TopAndBottom|physical first layer or that top-role condition|

The existing Rust physical counter/last-non-gapfill feature handoff is reused;
this slice does not invent a new counter, classify new roles or port ironing.
Actual layer arguments/state, not observer breakpoint counts, establish layer
identity. No epsilon/height proxy substitutes for the physical first-layer test.

**Preserve all existing unconditional retract/wipe/reset-E/pending semantics**:

- Do not early-return from the forced retract when can_lift is false.
- Keep restart-extra assignment, eligible wipe, remaining retraction and
  unconditional reset-E at their current boundaries.
- Keep the existing pending-layer and pending-lift handling/reset semantics;
  do not move those assignments behind the new surface predicate.
- Gate the eager invocation only; preserve the existing needs_lift and writer
  height/position/lift-type guards and their internal bookkeeping.
- Do not add a global surface guard inside append_eager/GCodeWriter's shell.
  Other already-guarded callers must retain their own upstream ownership.

The existing emission scaffold remains a temporary compatibility shell around
these cited libslic3r concepts. This slice replaces its missing parent contract;
it does not grow a separate Ares pipeline/state machine. Keep core logic portable
across browser WASM, Windows, macOS and Linux, with no UI/filesystem/terminal or
native-oracle behavior added to ares-core.

## Frozen public E2E witnesses

Origin: existing input-only
`tests/parity/modifier-bottom-shell/classic-four.3mf`, SHA256
`438bf4c61895a7a8d51e3c61a70fa8df120a9f3a607d63a4291d359984154a7c`.
Its stored Bambu X1C0.4 Standard-Mine process is customized, not pristine factory.
Preserve mesh, placement, modifier bottom-shell4, layer data and every other
setting. Each target changes only element0 of retract_lift_enforce:

| Case | Value | Input SHA256 |
|---|---|---|
|all-surfaces control|All Surfaces (original bytes)|438bf4c61895a7a8d51e3c61a70fa8df120a9f3a607d63a4291d359984154a7c|
|bottom-only primary|Bottom Only|f735864a4dc6d992440a7e9d9a8b5f620775cdda7ea09d53dc09e65849664bcf|
|top-only related target|Top Only|bd42ee78a736e7916ae8b4c549c8c52f45c6522b89214d1029d2701b9039bc77|
|top-and-bottom related target|Top and Bottom|792645d56b67be089d137bc95ce7b76827fbef882782774c235ddc3974ec63c3|

RED seam, explicitly approved: existing
`runner::native_stream_case::check`, which performs Native double replay before
public `slice_project_plate_validated` and actual ares CLI, then complete ordered
bytes comparison. Four direct calls, no duplicated harness/normalizer/helper.
Commit input-only3MFs and their SHA records; **no Native G-code outputs/goldens**.
No private call-count assertion, inverted equality, ignored test, rejection-as-
success, fallback, tolerance, M73/time/E/F/wipe/footer mask or comparator change.

On clean unmodified production, expect control PASS and the three targets to
fail actual Native/public whole-output equality in both official and seeded
modes. Native must succeed and replay stably; API/CLI must succeed and their raw
outputs must agree. A rejection, launch error or unstable oracle is not this RED.
Shared comparison panics may precede wrapper-tail metadata; audit raw artifacts
independently and never claim an unreachable assertion executed.

## Discovery evidence, limits and deferred behavior

Accepted discovery:
`/home/indexyz/ares-evidence/w3-zhop-lift-readonly-3fbad615e/README.md`, manifest
`8851089c627c59fd2b958fa66898c86b84112636f85b9e7366ee72998a3ca8c6`.
It observed Bottom's actual counters0..49: Native50 parent eager requests but
one writer entry; Rust50 eager entries with BottomOnly still selected. Four
successful observers retained whole canonical streams. Failed Native launcher,
unreadable Native writer-Z and unreliable Rust pending enum display were
excluded. These are diagnostic facts, not expected G-code constants in tests.

The complete first failure was time/M73-related, with later extra lift commands;
it is not permission to repair an estimator or mask progress. W4's class.3mf
clue was full-stream negative for the explicitly tried0%/50% overlap and
precise_z_height1 inputs; do not conflate it with this eager-parent witness.

Deferred: global writer guards, curve/angle/height mathematics, lazy-lift rules,
new feature/counter classification, generic toolchange/wrapping behavior,
spiral-vase, geometry/perimeters/Arachne/avoid/ironing, estimator changes and
other workers' scopes. No empirical numerical correction or adjacent cleanup.

## Stage exit / next approval barrier

1. Commit this SPEC alone, then commit test/input-only RED separately.
2. Run both modes on the exact clean RED tip, CPU12/jobs1/threads1, fresh
   sequential mode artifact/cache roots; preserve actual Native command/status/
   input/output identities and all public raw artifacts.
3. Run rustfmt and locked workspace/all-target Clippy; compile core+wasm for
   wasm32-unknown-unknown. Report actual gate results, never pretend browser run.
4. Submit RED RR with one passing control and three intended strict failures
   per mode, production/helper hashes unchanged. **Stop before GREEN**.

After independent RED approval, GREEN must close every frozen full stream and
preserve controls under the existing shared checker, plus the approved neighbor
scope and platform gates. Any residual means STOP/report, not adjacent repair.
No early FULL/rebase/merge; queued aligned/simple chains remain frozen.
