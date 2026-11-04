# WipeTower2 tower-approach lift type

Rewrite slice of the lift type used by the Type2 tower approach retract at OrcaSlicer v2.5.0-beta `345efec64`. It follows the gap travel (`wipe-tower2-gap-travel.md`), whose waypoints are the first moves to consume this lift.

## Problem
Before the travel to a Type2 tower, Native retracts with the lift type configured in `z_hop_types`. Ares always schedules a normal lift there. A normal lift is a standing `G1 Z` move. A slope lift rises along the first travel segment instead.

`mk4-layer-change/mk4base.3mf` plate 1 (Prusa MK4, Type2 rib tower, SEMM, `prime_tower_skip_points` 1, `z_hop_types` Slope Lift, `z_hop` 0.4, `travel_slope` 3) first differs in motion at the layer 0 tower approach:
- Native: `G1 X96.264 Y139.08 Z.6 F18000`, then `G1 X46.752 Y203.803 Z.6`. The first move is the slope end point, on the segment toward the first gap waypoint.
- Ares: `G1 Z.6 F18000`, then `G1 X46.752 Y203.803 Z.6`.

Every tower approach in the file shows this pattern (33 hunks). The parity test itself first fails earlier, on processor `M73`/`M74` lines, and also carries layer-change `M204` and final-purge differences that this slice does not touch.

## Upstream boundary
- **`WipeTowerIntegration::append_tcr2`** (`GCode.cpp:1711-1729`), when `travel_to_tower_now` holds:
  - `lift_type` defaults to `LiftType::NormalLift` (`:1720`).
  - It takes the configured type only when `writer().filament() != nullptr && writer().is_current_position_clear()` (`:1721`).
  - The configured type is `z_hop_types` at the current filament's config index (`:1722-1723`).
  - `zhtAuto` maps to `zhtSpiral` (`:1724-1725`), then `GCode::to_lift_type` (`:1726`, `GCode.cpp:10552-10565`): Normal → `NormalLift`, Slope → `SlopeLift`, Spiral → `SpiralLift`.
  - `retract(false, false, lift_type)` (`:1728`) passes it to `GCodeWriter::lazy_lift` (`GCodeWriter.cpp:901-932`), which stores it as `m_to_lift_type` only when nothing is lifted or pending.
- **The consumer** is the next travel (the first gap waypoint, else `travel_to(start_wipe_pos)`, `:1731-1732`). It goes through the existing `GCodeWriter::travel_to_xyz` lift handling, which Ares already ports for slope and spiral.

## Ares destination
- **`motion/tower.rs::retract`** (the `append_tcr2` retract) stops calling `travel::schedule_normal`. It schedules the `:1720-1726` lift type:
  - `LiftMode::Normal` unless `state.extruder_selected && state.positioned` (the `filament()` and `is_current_position_clear()` gates);
  - otherwise `state.options.z_hop_type`, with `Auto` → `Spiral`.
- **`motion/travel/lift.rs`:** the existing `schedule_normal_with` gate (retraction configured, `z_hop` > 0, the lift band, nothing lifted or pending) gets a lift-mode parameter. It does not reuse `mode_for`, because `mode_for` maps `Auto` to `Slope` for the generic retract.
- **Other callers stay as they are.** The Type1 `append_tcr` retract (`GCode.cpp:1210-1215`, `retract_for_type1`) and the other `schedule_normal` callers keep their own lift types.

## Included and deferred
- **Included:** `GCode.cpp:1717-1728` for the Type2 `append_tcr2` approach retract.
- **Deferred:**
  - `m_force_normal_lift` (`GCodeWriter.cpp:903-904`), which only `BeltGCode.cpp:18` sets. Ares has no belt printer path.
  - The `:1762-1790` in-place branch. It stays deferred with the `:1785` gap-travel site.
- **Not tower-owned, so not in this slice:**
  - the final purge skip when the tower does not reach the last print layer (`Print.cpp:5574-5593`), which is a separate candidate slice (see below);
  - layer-change `M204` and processor `M73`/`M74`.

## Oracle and RED
A section oracle with no whole-output parity claim. It goes in a new `wipe_tower2_tower_lift.rs`, a `#[path]` child of `plater_tower_positions.rs`, reusing `wipe_tower2_rib::slice_both`.

**Section definition.** This is the gap-travel approach section with Z added. For every `; CP TOOLCHANGE START`, the section is the moves before it, back to the last move that carries an `E` word. Each move is recorded as its X, Y and Z end point, written as `G1` words with `{:.3}`.

**Cases.** All use `class-default.3mf` (Normal lift, `z_hop` 0.4, `travel_slope` 3, gap wall on) with `z_hop_types` edited:
1. `SlopeLift`: RED. Native rises along the first approach segment.
2. `SpiralLift`: RED expected. If Native and Ares agree today, the case is reported before the RED commit and kept only as a control.
3. `Auto`: RED expected through the `Auto` → `Spiral` mapping (Ares's generic mapping would pick Slope). The same before-commit rule applies.
4. **Control:** `NormalLift` (the fixture as is). It must match Native now and after the slice.

If case 4 differs from Native today, the RED step stops and reports before committing. The `mk4-layer-change` witness is checked as evidence in the GREEN request (its approach sections), not as a new test.

**GREEN gate:**
- Paired FULL in the exact environment of the integration batch it is compared with, with no PASS→FAIL.
- FAIL→PASS is listed, with Type1 flips separately.
- The `mk4-layer-change` first motion difference is reported.

## Next candidate (not in this slice)
**Type2 final purge skip** (`Print.cpp:5574-5593`). When `tool_ordering.back().wipe_tower_partitions == 0`, the beta sets `generate_final_purge = false` and leaves `final_purge` empty, so `WipeTowerIntegration::finalize` (`GCode.cpp:2141-2147`) emits nothing.

Ares's `wipe_tower/data.rs` follows the older revision. It calls `set_layer(last.print_z)` and always generates the purge.

Witness: `mk4-layer-change`. The tower ends at Z 6.2 and the print at Z 10. Ares appends a tower block at Z 10 (`G1 Z10.4`, travel to `X46.752 Y203.803`, `; CP TOOLCHANGE START` with no toolchange); Native goes straight to the filament end G-code.
