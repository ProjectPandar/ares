# Custom G-code motion-state invalidation

Rewrite slice of `GCode::placeholder_parser_process`'s motion-state invalidation at OrcaSlicer v2.5.0-beta `345efec64`. When expanded custom G-code sets acceleration or jerk itself, the writer forgets its cached acceleration and jerk, so its next `M204`/`M205` is emitted even if the value is unchanged.

## Problem
`mk4-layer-change/mk4base.3mf` plate 1 (Prusa MK4, `marlin2`): `before_layer_change_gcode` emits `M201 X{…} Y{…}` on every layer.
- **Native:** after each layer change re-emits `M204 T4000` before the first travel, and `M204 P4000` before the first print move.
- **Ares:** keeps its cached 4000 and emits neither line.
- The file has 621 `M204` lines in Native against 527 in Ares, with 94 missing.

Every missing line comes straight after `;AFTER_LAYER_CHANGE` / `;_SET_FAN_SPEED_CHANGING_LAYER`. For example, Native lines 3684-3686 are:

```
M204 P4000
; printing object cube_with_hole.obj id:1 copy 0
M204 T4000
M486 S1
```

and Ares has no `M204 T4000`.

Across the official-mode integration FULL, 32 FAIL cases have an acceleration command in a custom G-code and a different count of acceleration lines:
- 15 come through `before_layer_change_gcode` (the MK4 and Prusa profiles);
- 17 come through `machine_start_gcode`, `change_filament_gcode` and `machine_end_gcode` (BBL, Prusa XL, `m1020`, `vg1`, printer-variant, concentric-ironing).

In the BBL cases the first difference has another cause, the first-travel retract ordering. That cause is not part of this slice. This slice accounts for their later `M204` gap.

## Upstream boundary
- **`GCode::placeholder_parser_process`** (`GCode.cpp:5319`). After `parser.process` and `validate_output_vector_variables` (`:5360-5361`), and before the position write-back (`:5368-5372`):
  - it calls `custom_gcode_motion_state_changes(output)` (`:5362`);
  - on an acceleration change it calls `m_writer.invalidate_acceleration()` (`:5363-5364`);
  - on a jerk change it calls `m_writer.invalidate_jerk()` (`:5365-5366`).

  A throw from the parser or the validation skips all of this.
- **`custom_gcode_motion_state_changes`** (`GCode.cpp:842-876`) runs a `GCodeReader` pass over the expanded text:
  - **Acceleration:** `M204`, `M201` or `M202` (`ascii_iequals` on the command), or `SET_VELOCITY_LIMIT` whose raw line `icontains` `ACCEL=`.
  - **Jerk:** `M205`, `M207` or `M566` with an `X`/`Y` in the part before `;` (`custom_gcode_line_has_xy_parameter`, `:848-853`), or `SET_VELOCITY_LIMIT` whose raw line `icontains` `SQUARE_CORNER_VELOCITY=`.
  - Parsing stops once both are found.
- **`GCodeWriter::invalidate_acceleration`** sets `m_last_acceleration = 0` and `m_last_travel_acceleration = 0`, and **`invalidate_jerk`** sets `m_last_jerk = 0` (`GCodeWriter.hpp:165-166`). These caches are compared in:
  - `set_acceleration` (`GCodeWriter.cpp:437-441`);
  - `set_jerk_xy` (`:483-488`);
  - the Klipper combined `SET_VELOCITY_LIMIT` path (`:552-570`).

### Expansion sites through `placeholder_parser_process`, and their Ares equivalents

| Beta site | Template | Ares site |
|---|---|---|
| `GCode.cpp:1202` `append_tcr` | `filament_end_gcode` | `wipe_tower/type1/templates.rs` `end` (`render`, `:35`) |
| `:1459` `append_tcr` | `change_filament_gcode` | `wipe_tower/type1/emit.rs:113` → `templates::render` |
| `:1594` `append_tcr` | `filament_start_gcode` | `wipe_tower/type1/templates.rs` `start` |
| `:1618` `append_tcr` | `tcr_rotated_gcode` | `wipe_tower/type1/emit.rs:156` → `templates::render` |
| `:1922` `append_tcr2` | `tcr_rotated_gcode` | `wipe_tower/emit.rs:168` |
| `:4201` `_do_export` | `imex_mode_gcode` | none (IMEX not ported) |
| `:4206` `_do_export` | `file_start_gcode` | `file_start.rs:32` |
| `:4356` `_do_export` | `machine_start_gcode` | `machine.rs:196` |
| `:4406`, `:4436` `_do_export` | initial `filament_start_gcode` | `layer_gcode/startup.rs:105-111` |
| `:4670` `_do_export` | `printing_by_object_gcode` | `layers/transition.rs:185` |
| `:4839`, `:4847`, `:4850` `_do_export` | `filament_end_gcode`, `machine_end_gcode` | `finish.rs:260` (`append_template`) |
| `:5713`, `:5735` `emit_custom_gcode_per_print_z` | `machine_pause_gcode` | `layers/custom_gcode.rs` `render` |
| `:5718` | `color_change_gcode` | compiled out in Ares, as upstream's `#if 0` block (`layers/custom_gcode.rs`) |
| `:5741` | `template_custom_gcode` | `layers/custom_gcode.rs` `render` |
| `:6112` `process_layer_change_gcode` | `layer_change_gcode` | `layer_gcode.rs:213` |
| `:6199` `process_belt_brim_layer` | `before_layer_change_gcode` | none (belt printers not ported) |
| `:6477` `generate_timelapse_gcode` | `time_lapse_gcode` | `timelapse.rs:180` |
| `:6670` `process_layer` | `before_layer_change_gcode` | `layer_gcode.rs:122` |
| `:6712` `process_layer` | `time_lapse_gcode` | `timelapse.rs:180` |
| `:7406` `process_layer` | `wrapping_detection_gcode` | `layers/wrapping.rs:71-75` |
| `:9687` `_extrude` | `change_extrusion_role_gcode` and variants | `motion/path/role_change.rs:85-87` |
| `:10886`, `:11306` `set_extruder` | `filament_start_gcode` | `tool_transition.rs:287` |
| `:10941` `set_extruder` | `filament_end_gcode` | `tool_transition.rs:287` |
| `:11208` `set_extruder` | `change_filament_gcode` | `tool_transition/change.rs:62` |

The Ares site list is checked again against `rg 'template::render|render_for_filament'` when the RED is written. Any site missing from the table is added, or reported.

## Ares destination
- **New `motion/motion_state.rs`.** It ports `custom_gcode_motion_state_changes` and the `GCodeReader` command split it relies on: the leading-blank skip, the first word as the command, and the raw line kept for the `icontains` checks.
  - It exposes `invalidate_after_custom_gcode(rendered: &str, state: &mut EmitState)`.
  - That function sets `state.last_acceleration` and `state.last_travel_acceleration` to `None` on an acceleration change, and `state.last_jerk` to `None` on a jerk change. `None` is Ares's "nothing cached", which `motion/acceleration.rs` and `motion/jerk.rs` already treat like upstream's 0.
- **`motion::apply_writer_state(name, config, state)`** is the shared `:5361`/`:5368` write-back that almost every site already calls.
  - It gains the rendered text: `apply_writer_state(name, config, rendered, state)`.
  - It calls `invalidate_after_custom_gcode` after `validate_vector_size`, so a failed validation does not invalidate, and before the position write-back. This follows upstream's order.
- **Sites that skip `apply_writer_state`** call `invalidate_after_custom_gcode` directly after a successful render:
  - `layers/custom_gcode.rs` `render`;
  - `layers/transition.rs:185` (`printing_by_object_gcode`).

  Their missing position write-back is an existing gap and not part of this slice.
- **`file_start.rs`** is not changed. `file_start_gcode` (`:4206`) expands before the writer has emitted any acceleration, so its caches are still 0 and invalidating cannot change the output. `print_machine_envelope` writes its `M201`/`M204` raw (`GCode.cpp:5455-5465`) and does not touch the caches.

## Included and deferred
- **Included:** `GCode.cpp:842-876` and `:5362-5366` at every ported site above, and `GCodeWriter.hpp:165-166`.
- **Not ported (no Ares site):**
  - `imex_mode_gcode` (`:4201`);
  - the belt-printer `before_layer_change_gcode` (`:6199`);
  - `color_change_gcode` (`:5718`, compiled out in Ares).
- **Not in this slice:**
  - the BBL first-travel retract ordering, owned by another slice;
  - the position write-back gaps of `layers/custom_gcode.rs` and `layers/transition.rs`;
  - `mk4-layer-change`'s tower differences (the tower-lift and final-purge slices);
  - processor `M73`/`M74` and the config-footer vector lengths.
- **Start-gcode witness: none is possible.** At `machine_start_gcode` (`:4356`) and the initial `filament_start_gcode` (`:4406`/`:4436`) the writer caches are still 0, for the reason given under `file_start.rs`. Invalidating there cannot change the output. The BBL cases' `M204` in `machine_start_gcode` therefore makes no difference; `change_filament_gcode` and `machine_end_gcode` do. The second witness below exercises a different per-layer site instead.

## Oracle and RED
A section oracle with no whole-output parity claim. It goes in a new `custom_gcode_motion_state.rs` under the `orca_parity` tree as a `#[path]` child (`orca_parity.rs` stays byte-identical). Native and the CLI slice the same project.

**Section definition (labelled `acceleration and jerk lines`).** Per `;LAYER_CHANGE`, the ordered list of:
- the `M204`, `M205`, `M201`, `M202` and `SET_VELOCITY_LIMIT` lines;
- the `;TYPE:` role lines, so each acceleration or jerk line is placed inside the role block it precedes.

This captures both count and position, and does not depend on travel coordinates, which other open slices still change in this file.

**Cases.** All use `mk4-layer-change/mk4base.3mf` with `before_layer_change_gcode` / `layer_change_gcode` edits and `enable_prime_tower` set to 0, which keeps the tower final-purge differences (a separate slice) out of the last layer:
1. **`before_layer_change_gcode`** (`:6670` → `layer_gcode.rs:122`), fixture as is. RED: Native re-emits `M204 T4000`/`P4000` after each layer change.
2. **`layer_change_gcode`** (`:6112` → `layer_gcode.rs:213`): the `M201 X{…} Y{…}` line removed from `before_layer_change_gcode` and a literal `M201 X4000 Y4000` added to `layer_change_gcode` (the template form makes Native abort with the tower off: `interpolate_table` rejects its negative first-layer `extruded_weight_total`). RED.
3. **Jerk:** `M205 X8 Y8` appended to `layer_change_gcode`. Native re-emits the role jerk (`M205 X7 Y7` or `M205 X9 Y9`) after each layer change. RED.
4. **Control:** the `M201` line removed from `before_layer_change_gcode`, so no custom G-code touches the motion state. It must match Native now, and catches over-invalidation after GREEN.

Before the RED commit:
- Cases 2 and 3 are run against Native. If one shows no re-emission in Native, it is dropped and reported.
- If case 4 differs from Native today, the RED step stops and reports.

**GREEN gate:**
- Paired FULL in the exact environment of the integration batch it is compared with, with no PASS→FAIL.
- FAIL→PASS is listed.
- For the 32 cases in the scan, the change in acceleration-line count is reported, with the BBL cases listed separately because they share the retract-ordering cause.
