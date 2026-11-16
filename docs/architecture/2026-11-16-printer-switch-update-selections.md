# Printer switch: `PresetBundle::update_selections` in the plater selection

Upstream baseline: OrcaSlicer v2.5.0 (40ecc66f7).

## Upstream sources

- `slic3r/GUI/Tab.cpp:7701-7744` `Tab::select_preset` for a printer. `update_compatible` keeps the
  current print and filaments while compatible (`:7703-7714`). With `remember_printer_config` on,
  its default (`libslic3r/AppConfig.cpp:544-545`), `PresetBundle::update_selections` runs next
  (`:7740-7742`) and replaces them.
- `libslic3r/PresetBundle.cpp:3914-4017` `update_selections`, and `load_selections` (`:4082-4164`),
  which restores the stored printer the same way at startup:
  - The print and slot 1 come from `select_preset_by_name_strict` on the names stored for the
    printer (`:3918-3923`). An empty or unknown name clears the selection
    (`Preset.cpp:4335-4357`).
  - `filament_presets` is slot 1 plus the stored `filament_NN` (`:3929-3937`). The project's
    filament count plays no part.
  - `filament_colour` is the stored `filament_colors`, resized with `"#26A69A"` (`:3941-3947`).
    `filament_multi_colour` is the stored value, or else the colours (`:3949-3954`).
    `filament_colour_type` is resized with `"1"` (`:3956-3961`).
  - `filament_map`, `filament_nozzle_map` and `filament_volume_map` are reset, one entry per
    colour (`:3963-3970`).
  - The flush matrix, flush vector and multiplier come from the stored values when present, and
    otherwise stay as they are in the project config (`:3978-3993`).
  - `update_compatible(Always)` then `update_multi_material_filament_presets` (`:4002-4003`). The
    first fills a cleared print with `PreferedPrintProfileMatch(nullptr, default_print_profile)`:
    no alias and no layer height, so the visible `default_print_profile` wins
    (`:8749-8768, 8837-8838`). It fills a cleared
    slot 1 with `PreferedFilamentsProfileMatch`: type `PLA` when there is no current filament, and
    the names in `default_filament_profile` (`:8801-8822, 8855-8863`). Only after that are the slots
    padded to `min_filament_count` with the last slot (`:8611-8617`).
- `export_selections` (`PresetBundle.cpp:4232-4292`) stores, per printer: print, slot 1,
  `filament_NN`, `curr_bed_type`, the palette (`filament_colors`, `filament_multi_colors`,
  `filament_color_types`), and the flush data, written with `std::to_string`.
- The setup wizard: `GuideFrame::run` stores the selection first (`WebGuideDialog.cpp:1224`).
  `apply_vendor_config` then runs `load_selections` (`PresetBundle.cpp:2074-2099`), which selects
  the wizard's preferred printer. It takes the print, the slots, the palette and the flush data from
  the printer stored before the wizard (`initial_printer_profile_name`, `:4045-4155`), not from the
  new printer. An opened project's presets are external presets (`load_config_model`), which
  `load_presets` drops: their names no longer select anything.
- The plater's project config is `full_fff_config`, with one filament per selected preset
  (`PresetBundle.cpp:5651-5660`), which the plater saves (`Plater.cpp:19779`). A plater selection
  therefore slices with its own filament count. The CLI keeps the embedded count
  (`OrcaSlicer.cpp:2528-2529`).

## Ares destination

- `web/app/src/state/presets.ts`:
  - `rememberedSelection`: the `update_selections` state for a printer, read from the stored
    printer settings.
  - `selectPrinter`: applies that state. `updateCompatible` takes its slots from the selection only.
  - `saveSelection`: `export_selections`, now with the palette and flush data.
- `web/app/src/state/wizard.ts` `finishWizard`: the wizard's `load_selections`. It passes the
  selection stored before the wizard to `selectPrinter`. With a project open and no preset
  selection, that is the project's palette and flush data, with a cleared print and slots.
- `web/app/src/state/persistence.ts` `PrinterSettings`: the stored palette and flush fields.
- `crates/ares-core/src/profiles/compatible/print_selection.rs` `compatible_print`: a named print
  that is not loaded is a cleared selection. No name keeps the collection's initial default
  selection (`Preset.cpp:1739`).
- `crates/ares-core/src/profiles/compatible/selection.rs` `select_compatible_filaments`: chooses
  compatible presets without padding; `normalize_filament_slots` pads afterwards.
- `crates/ares-core/src/project/domain/profile_input/three_mf.rs`: a catalog selection (the
  plater's presets) takes its filament count from the selection. `--load-settings` keeps the CLI
  bound.

## Included

The stored and never-used cases above, at a printer switch and at startup. The slice blocker and
the dual-nozzle filament map follow the selection's slot count.

## Deferred

- Mixed-filament metadata (`load_mixed_filament_settings`, `:3994-3996`) and the AMS counts
  (`:3972-3976`): the web has neither.
- The sidebar's `on_filament_count_change` (`Tab.cpp:7742`) for objects whose extruder index is
  past the new filament count.
- The `first_compatible()` pass for a slot that is still invisible after `update_compatible`
  (`:4005-4015`).
- The wizard's slots 2 and up of a multi-filament project: upstream keeps the dropped external
  names, which `update_multi_material_filament_presets` resolves. The web clears them.
- The external preset entry that `GuideFrame::run` stores for an opened project. The web stores no
  entry for it, because no later selection can read it.
- The plater route slices with CLI machine-switch semantics. In the GUI, a printer switch takes
  the flush data from `update_selections` when the extruder count changes, and does not shrink the
  model onto the new bed. Moving the route to GUI semantics is a separate slice, which removes the
  constructions below.

## Verification

`web/e2e/printerSwitchFilaments.spec.ts`. The reference is the preset-select probe
(`tools/chain-probe/preset-select.patch`), which runs `update_selections` after a printer select
on a fresh AppConfig.
- A one-filament 3MF with the never-used Snapmaker U1: the sidebar's process, filaments and
  colours equal the probe's plater session.
- The same selection sliced: the G-code equals OrcaSlicer slicing the plater's saved project, as
  the Native CLI leaves it when it loads the opened 3MF with the selected presets
  (`cliSwitchedExport` in `web/e2e/platerExport.ts`, OrcaSlicer.cpp at v2.5.0). Each change is
  computed from the opened 3MF and the selection:
  - the instances and the wipe tower move by the difference of the bed centres
    (`shrink_to_new_bed`, `:1899-1909, 4371-4382, 4414-4443, 4619-4685`);
  - `curr_bed_type` is the printer's `default_bed_type` (`Plater.cpp:3880-3900`,
    `Preset.cpp:1140-1150`);
  - `nozzle_volume_type` stays the 3MF's when it has one (`:6981-6985`);
  - `filament_plugin_config_overrides`, a scalar filament option the CLI skips (`:3715-3734`),
    comes only from the 3MF;
  - `compatible_printers_condition` is the loaded process's (`:3099-3102, 3405-3407`);
  - one filament preset in every slot, of one colour, turns the prime tower off and records it
    in `different_settings_to_system` (`:2531-2535, 3880-3888, 4291-4342`), sized filament
    count + 2 (`:3202-3205`).
  The case passes once the core keeps the selection's `filament_map` (`:6974-6980`) and
  recomputes the flush data on a CLI machine switch (`:3864-3870`). Both are separate core
  slices.
- A three-filament 3MF with the never-used single-nozzle Snapmaker A350: one filament and one
  colour, as in the probe.
- `web/e2e/presetSlice.spec.ts` opens a 3MF, runs the wizard and picks the printer: the palette is
  the project's, as `load_selections` gives it to the wizard's printer.
- `web/e2e/persistence.spec.ts`: the never-used user printer My X1C gets one slot
  (`PresetBundle.cpp:3929-3937`, `min_filament_count()` 1 at `PresetBundle.cpp:5419-5424`).
