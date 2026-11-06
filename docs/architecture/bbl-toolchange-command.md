# BBL toolchange command

This is a source-cited rewrite slice of `OrcaSlicer/src/libslic3r` `GCodeWriter::toolchange_prefix` and
`GCodeWriter::toolchange`, with the nozzle id their `GCode` callers pass. It is pinned to upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It separates the tool-selection prefix from the BBL
command form, which Ares currently shares. It adds no Ares-owned toolchange rule.

## Source boundary

- `src/libslic3r/GCodeWriter.cpp:793-800`, `toolchange_prefix()`: returns the manual filament change
  tag followed by `T`, `M135 T` (MakerWare), `M108 T` (Sailfish), or `T`. It has no BBL branch.
- `src/libslic3r/GCodeWriter.cpp:803-830`, `toolchange(filament_id, nozzle_id)`:
  - It emits only for multiple extruders, or for several filaments on a non-BBL printer.
  - A BBL printer without manual filament change writes `M1020 S<filament_id> H<nozzle_id>`.
  - Every other printer writes `toolchange_prefix()` followed by the filament id.
- `src/libslic3r/GCode.cpp:11237-11247`, `set_extruder`: the command is appended only when
  `custom_gcode_changes_tool(toolchange_gcode, toolchange_prefix(), new_filament_id)` (`:455`)
  finds no selection of the new filament in the rendered change template. The wipe tower path
  (`:1484-1490`) applies the same rule.
- `src/libslic3r/GCode.cpp:11070`: the nozzle id is
  `nozzle_id_for_gcode_placeholder(group_result, new_filament_id, new_extruder_id, layer)`.
- `src/libslic3r/GCode.cpp:241-247`, `nozzle_id_for_gcode_placeholder`: with a nozzle group result,
  `get_nozzle_id(filament_id, layer)`; without one, the extruder id.
- The group result is published on every static path: by layer, or one object by object
  (`GCode/ToolOrdering.cpp:3014-3015,3101-3102`), and by object (`Print.cpp:3711`).
  - For printers with one nozzle per extruder, the default nozzle list holds one nozzle per entry
    of `nozzle_diameter`, with `group_id` equal to the extruder index
    (`ToolOrdering.cpp:1892-1907`).
  - The filament's nozzle is its 0-based `filament_map` entry (`ToolOrdering.cpp:1917-1941`,
    `MultiNozzleUtils.cpp:142-157`). The by-layer grouping resolves the same way on these printers
    (`ToolOrdering.cpp:3093-3097`).
  - `get_nozzle_id` returns -1 when the filament is outside that map or its nozzle is outside the
    list (`MultiNozzleUtils.cpp:380-387,442-446,454-458`).
- `src/libslic3r/GCode/CoolingBuffer.cpp:40,523-525,915`: the cooling buffer recognizes a tool
  change by `toolchange_prefix()`.

## Destination

`crates/ares-core/src/project_slice/gcode_emit/motion/tool_selection.rs`:
- `prefix()` keeps only the manual tag, `M135 T`, `M108 T` and `T` cases. It stays the scan prefix
  of `custom_changes_tool` and the prefix passed to the cooling buffer
  (`cooling/initialize.rs`), so both consumers follow it.
- `toolchange()` writes `M1020 S<filament> H<nozzle>` for a BBL printer without manual filament
  change, and `prefix()` with the filament otherwise.
- `nozzle` is the port of `nozzle_id_for_gcode_placeholder` over the static group result: the
  filament's `filament_map` entry minus one when it names a nozzle of the default list (fewer than
  `nozzle_diameter` entries), else -1.

## Included

- BBL single-extruder change templates that select the tool themselves (`T[next_extruder]`): no
  extra command is written. Witness: X1C two-object layer labels.
- BBL change templates without a tool selection: `M1020 S<id> H<nozzle>`. Witness: the towerless
  X1C `M1020 S1 H0` / `M1020 S0 H0`.
- Non-BBL multi-tool output keeps `T<n>`. Witness: the Ender-3 layer-label control.

## Deferred

- The dynamic nozzle map (`support_dynamic_nozzle_map`) and multi-nozzle extruders
  (`extruder_max_nozzle_count > 1`), whose nozzle ids come from the per-layer grouping
  (`MultiNozzleUtils.cpp:183-235`). That includes the X2D dynamic group nozzle.
- The `hotend_id_for_gcode_placeholder` template values and the processor's `M1020` billing are
  not changed.

## Failure modes

1. The BBL branch stays in the scan prefix: a template's `T<n>` is missed and a second selection
   is appended.
2. The prefix change reaches the command: BBL printers write `T<n>` instead of `M1020`.
3. The `H<nozzle>` suffix is dropped, or uses the filament id.
4. A filament outside the nozzle list writes a clamped nozzle instead of -1.
5. The cooling buffer keeps the old prefix and splits the per-tool cooling at different lines.
6. Non-BBL or manual filament change output changes.

## Tests

`crates/ares-cli/tests/orca_parity/option_coverage/bbl_toolchange_command.rs` compares the
tool-selection lines (`T<n>` and `M1020 ...`) in order, Native vs API and CLI:

- X1C two objects (`bbl-layer-object-labels/two-object.3mf`): Native has no `M1020` line, and its
  `T0` / `T1` come from the change template.
- X1C towerless change (`m1020-filament-change/towerless-change.3mf`): Native writes
  `M1020 S1 H0` / `M1020 S0 H0` and no `T0` / `T1`.
- Ender-3 control (`bbl-layer-object-labels/non-bbl.3mf`): Native writes `T<n>` and no `M1020`. It
  already matches and must keep matching.
