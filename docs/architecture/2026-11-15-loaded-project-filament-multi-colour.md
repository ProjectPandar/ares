# Always selection: `filament_multi_colour` of the plater's project config

Citations refer to OrcaSlicer 2.5.0 (`40ecc66f7`).

## Problem

The browser leaves `filament_multi_colour` to the loaded 3MF (c943d982b,
`2026-11-14-selected-preset-filament-multi-colour.md`), so a project without the key slices
without the line. The plater's G-code has it: the GUI slices `full_config(false)`
(Plater.cpp:11442), whose project part is `PresetBundle::project_config`, and
`GCode::append_full_config` (GCode.cpp:8272-8382) prints every key of that config.

## Upstream reference

- Startup: `PresetBundle::load_selections` (PresetBundle.cpp:3946 and 4109) sets
  `filament_colour` to the app config's `filament_colors` of the selected printer, padded with
  `#26A69A` to the startup filament count, and `filament_multi_colour` to the app config's
  `filament_multi_colors`, else to those colours (3953-3954 and 4116-4117).
- Wizard: `GUI_App::run_wizard` runs `GuideFrame` (GUI_App.cpp:9997). `GuideFrame::run` calls
  `export_selections` (WebGuideDialog.cpp:1223), then `GuideFrame::apply_config`
  (WebGuideDialog.cpp:1005-1205), which ends in `PresetBundle::apply_vendor_config`
  (PresetBundle.cpp:1952-2093): `load_presets` runs `load_selections` again (2076).
- Project open: `load_config_model` (Plater.cpp:9622) applies the 3MF's keys to
  `project_config`; a 3MF without `filament_multi_colour` leaves the value already there. Then
  the plater runs `sync_loaded_project_filament_config` (Plater.cpp:9755, 8905-8959): it
  resizes `filament_multi_colour` to the `filament_colour` count, and sets entry `i` to
  `filament_colour[i]` unless the entry's first space-separated colour already equals it
  (8927-8937).

So after a project opens, the value is `filament_colour`, except entries the 3MF stores whose
first colour matches. Its entry count is the project's filament count, whatever the startup
or wizard path left before.

## Destination

- `crates/ares-core/src/profiles/project_selection.rs`, `refresh`: the project config it
  returns holds the synced `filament_multi_colour`, computed from the loaded config's
  `filament_colour` and `filament_multi_colour`.
- `web/app/src/state/presets.ts`, `updateCompatible`: `filamentProject` keeps
  `filament_multi_colour` again (reverses c943d982b), so the selected-preset slice sends the
  project config's value. With no project open, the value is the slot state's
  `filament_colour`, as `load_selections` copies the colours.

## References

- Preset-select probe (`tools/chain-probe/preset-select.patch`, test-only):
  - `sync_loaded_project_filament_config` loses `static`, and the probe calls it after
    `load_config_model`, as Plater.cpp:9755 does.
  - Optional per-project steps after the load: `wizard` (the GuideFrame path above, with
    `preferred_printer_model` as WebGuideDialog.cpp:1053-1094; the filament section stays the
    app's, WebGuideDialog.cpp:1693-1695) and `select` (the collection selection of
    `Tab::select_preset`, Tab.cpp:7701-7714, and `PresetBundle::set_filament_preset` from
    `Plater::priv::on_select_preset`), then `process_edit`, a sidebar process edit:
    `ConfigOptionsGroup::on_change_OG` writes the value into the edited preset
    (OptionsGroup.cpp:694-730, Tab.cpp:775) and the option group runs `Tab::update_dirty`
    (Tab.cpp:9711-9718, 1601-1613), so `full_fff_config` lists the edited option in
    `different_settings_to_system` as the plater does (PresetBundle.cpp:5587-5594). No spec here
    uses `process_edit`; it serves the deferred group composition. With any step, the record also
    holds the bundle right after the load (`opened`).
- Selected-preset lines (`web/e2e/selectedPresetLines.ts`): `filament_multi_colour` joins
  `SELECTED_PRESET_KEYS`, its value taken from the synced probe's `full_fff_config`, the same
  mechanism as `pellet_flow_coefficient`.
- Plater export: the 3MF the plater saves after the session, its project config the probe's
  `full_fff_config` (`full_config_secure`, Plater.cpp:19779) with the sidebar edits on top,
  sliced by the Native CLI. The Native CLI rejects the export of a first-run project with
  CLI_PROCESS_NOT_COMPATIBLE, -17 (OrcaSlicer.cpp:2575-2620): its presets are project-local
  `<name>(<file>.3mf)`.
  - Blocked by -17, the reference stays the CLI-route slice of the opened 3MF with the
    selected-preset lines: colorChange, plateInstances, plateMirror, plateMove,
    plateNumberOfInstances, plateRotate, plateScale, plateScaleToFit, plateScaleToFitCircle,
    slice, settings (orca_cli_ender3); multiPlate, platerSession, plateDeleteObject,
    platePlateDeleteAll (two-plates); plateDeleteInstance (combined-brim); import
    (default-prisms).
  - The plater export: plateNozzle (BBL X1 Carbon) and dualExtruder (BBL H2D). In dualExtruder
    the Native CLI prints the four groups `compatible_machine_expression_group`,
    `compatible_process_expression_group`, `inherits_group` and `different_settings_to_system`
    for an export whose config lacks them (OrcaSlicer.cpp:3202-3205 creates the last two), while
    the plater stores a group only when some entry is non-empty (PresetBundle.cpp:5776-5838). The
    reference removes such a line only when the probe's `full_fff_config` lacks the key, and
    only after asserting every entry is empty; a non-empty entry fails the spec. plateNozzle
    compares only the plate-type and bed-temperature lines.
  - presetSlice: after the wizard and the system preset picks, the browser slices on the explicit
    preset route, the `--load-settings` model. The 3MF legs stay the CLI `--load-settings`
    slices with the selected-preset lines of the plater session (the probe's `wizard` and
    `select` steps). The STL leg: the plater adds the STL to the open project, while the browser
    replaces the scene; the reference stays the CLI slice of the centered cube, its
    `filament_multi_colour` line its `filament_colour`, the no-project value
    (PresetBundle.cpp:4116-4117).

## Included

- `filament_multi_colour` after a project opens and with no project open.

## Deferred

- An app config `filament_multi_colors` entry of several colours whose first colour equals the
  project's colour: the plater keeps it; the browser has no app config colours and writes the
  single colour.
- The other keys `sync_loaded_project_filament_config` touches (`filament_map`,
  `filament_nozzle_map`, `filament_volume_map`, `filament_colour_type`,
  `has_filament_switcher`, `enable_filament_dynamic_map`).
- The group composition of `full_fff_config` (PresetBundle.cpp:5571-5594, 5620-5712,
  5776-5787, 5826-5838): the browser still prints the CLI's `inherits_group` and
  `different_settings_to_system` (OrcaSlicer.cpp:3202-3205) on every route, so the CLI-route and
  presetSlice references keep their group lines, and dualExtruder differs from the plater export
  on those lines.

## Failure modes and how each is ruled out

1. **A stored multi-colour entry is lost.** An entry whose first colour matches stays; the
   core test stores one.
2. **The entry count follows the startup or wizard state.** The count is the project's
   `filament_colour` count; the core test has three filaments and two stored entries.
3. **The line appears in CLI-route references the plater export cannot replace.** The
   selected-preset lines set it from the probe's `full_fff_config`.
4. **The probe no longer reports what `load_config_model` leaves.** The sync is what the plater
   runs right after it; every probe consumer is the plater's state.

## Tests

- `crates/ares-cli/tests/project_preset_selection/loaded_filament_sync.rs`: a project without
  the key and a three-filament project storing two entries. Native premise: the probe's
  `full_fff_config` holds the synced value. The Always output equals it.
- The browser specs above, against the references above.
