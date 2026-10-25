# ARD: `disable_once` arm/consumption ordering (avoid-crossing entry gate)

Upstream pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Status: SPEC (approved
boundary for tests-only RED; GREEN separately authorized).

## Boundary being ported

Upstream owns the one-shot crossing-avoidance disable in three exact places:

1. **Arm, skirt end** — `GCode.cpp:4387-4454` `GCode::generate_skirt`: after all
   assigned skirt loops are extruded, `use_external_mp(false)` (`:4448`) then
   `disable_once()` (`:4449-4450`) gated on `first_layer && loops.first == 0`
   (the extruder that drew skirt loop 0). During skirt emission the flag is
   CLEAR: `use_external_mp()` was set at `:4413`, and every skirt travel runs
   the feature-less gate below.
2. **Arm, brim end** — `GCode.cpp:4486-4527` `GCode::generate_object_brim`:
   `use_external_mp()` (`:4496`), extrude brim entities, then
   `use_external_mp(false); disable_once();` (`:4498-4500`). Same shape as the
   skirt arm.
3. **Consume, before the retraction branch** — `GCode.cpp:travel_to:7415-7452`:
   the feature-less gate `reduce_crossing_wall && !disabled_once() &&
   is_current_position_clear()` (`:7417-7420`) runs the first planning slot;
   `reset_once_modifiers()` (`:7429-7431`, clears `m_disabled_once` AND
   `m_use_external_mp_once`) executes immediately after that slot and BEFORE
   `needs_retraction`/`retract()` (`:7424-7443`); only then may the post-wipe
   re-plan (`:7444-7452`, re-arming `use_external_mp_once` at `:7446-7447`
   when it was used) call `travel_to` again — with the disable already clear.

The sequential by-object arm at `GCode.cpp:3325` is already ported
(`layers/transition.rs:119`) and stays unchanged.

Rust destination (existing code, no new pipeline):
`project_slice/gcode_emit/motion/path/start_travel/{route.rs,approach.rs}`,
`motion/path.rs:269-271`, and the skirt/brim emission sites in
`project_slice/gcode_emit/layers/` + `gcode_emit/brim*`. Current state is a
temporary compatibility shell: the flag is armed at the Skirt feature tag
(skirt START, `path.rs:269-271`), never armed for brim, and consumed only
AFTER the wipe re-plan (`approach.rs:181-186`).

## Itemized audit of the existing `!matches!(feature, "Skirt" | "Brim")` exceptions

| Ares code | Behavior | Upstream counterpart | Verdict |
|---|---|---|---|
| `route.rs:35` gate excludes Skirt/Brim features | skirt/brim travels never route | `GCode.cpp:7417-7420` gate consults only `reduce_crossing_wall`, `disabled_once()`, `is_current_position_clear()` — no feature term | **No upstream basis.** Compensates for arming at skirt START instead of END |
| `approach.rs:182` consumption excludes Skirt/Brim | skirt/brim travels do not consume the armed flag | `reset_once_modifiers` at `:7429-7431` is unconditional per `travel_to` call | **No upstream basis.** Same compensation |
| `path.rs:269-271` arms at Skirt feature tag (layer 0) | flag armed when `;TYPE:Skirt` is emitted | arm at skirt emission END (`:4449-4450`), only for the loop-0 extruder | **Wrong position**; upstream keeps the whole skirt emission routable |
| no brim arm anywhere | brim-only prints never arm | `generate_object_brim:4498-4500` arms at brim end | **Missing** upstream behavior |
| `use_external_mp` (non-once) windows absent | nothing sets the external boundary window | `:4413/:4448` (skirt), `:4496/:4498` (brim) wrap emission | **Missing**; becomes load-bearing once the gate exception is removed |

Latent consequences of the shell (not exercised by current tests, all of which
pass): skirt/brim travels whose straight path crosses walls would detour
upstream but not in Ares; brim-only prints arm upstream but not in Ares.

## RED (this chain)

`ankermake5_first_object_travel_detour` (registered under the existing grouped
`object_emission_cases.rs`; root registry files untouched) evaluates the
Native-runtime stock fixture `tests/parity/avoid-once-ankermake5/ankermake-m5.3mf`
through the unchanged shared `native_stream_case::check` comparator (complete
ordered generator-only comparison, M73 unmasked, Native double replay before
public comparisons, raw API==actual CLI). Expected strict failure: first
difference at byte 2426 (line 106) after `;WIPE_END` — upstream emits the
post-wipe re-plan detour (6 intermediate vertices on the fixture hole contour
plus the destination) while Ares emits one direct segment. Bilateral gdb
pinpoint: `/home/indexyz/ares-evidence/w2-matrix5-20261025T223253-pinpoint/PINPOINT.md`.

## GREEN boundary (authorized separately; upstream literal order only)

1. Consume `avoid_crossing_disabled_once`/`use_external_mp_once` immediately
   after the first planning slot, before `retract::decide` — the `:7429-7431`
   position — unconditionally (no feature term).
2. Remove the Skirt/Brim exceptions from the gate (`route.rs:35`) and the
   consumption site; correct the `route.rs:24-27` comment that currently
   asserts the flag survives the wipe re-plan.
3. Re-time the skirt arm from the Skirt feature tag to skirt emission END on
   the first layer (loop-0 extruder condition), and add the missing brim-end
   arm, both exactly as `:4449-4450` / `:4498-4500`.
4. Add the `use_external_mp` true/false windows around skirt and brim emission
   (`:4413/:4448`, `:4496/:4498`) so the now-routable skirt/brim travels plan
   against the external boundary as upstream does.

No assertion weakening, no fixture/golden G-code, no new guards or fallbacks.

## Deferred

Detour polyline shape internals (W4 lane: router/path_shape family), external
boundary construction internals (`AvoidCrossingPerimeters.cpp` beyond the entry
contract), multi-extruder skirt loop-range assignment beyond the loop-0 arm
condition, wipe-tower and cooling-buffer interactions, other gcode flavors'
emission differences, and any residual beyond the first stable difference.

## Exit criteria

Dual-mode (official + seeded) strict comparator PASS on the RED target at the
GREEN tip under CPU13/threads1, with existing suites unchanged (scope of any
re-run per separate authorization).
