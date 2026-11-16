# Web slice blocker, compatible filaments and 3MF value warnings

## Problem

A project with filament presets for another printer, such as `Bambu PLA Matte @BBL A1M` with a
Bambu Lab H2D selected, could not be sliced in the web, and nothing said why:

- the filament combo showed the project's preset while the selection held none;
- the status bar said "Ready to slice" while Slice plate was disabled.

A 3MF with an out-of-range value, such as `raft_first_layer_expansion = -1`, failed to slice with
the CLI's range error. OrcaSlicer's GUI slices the same file and only warns at load.

## Source boundary

OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`.

### Compatible filament presets

- `PresetCollection::update_compatible_internal` (`Preset.cpp:3965-4005`) sets `is_compatible`
  from `is_compatible_with_printer` and `is_compatible_with_print`. `filter_compatible`
  (`profiles/compatible.rs`) already ports that decision.
- `PlaterPresetComboBox::update` (`PresetComboBoxes.cpp:1234-1300, 1501-1503`) lists incompatible
  filaments only under the `show_unsupported_presets` preference, which is off by default. Its
  value is the selected `filament_presets[idx]` (`:1237-1238`). `TabPresetComboBox::update`
  (`:452-456`) skips `!is_visible || !is_compatible` the same way.
- A printer switch:
  1. `Tab::select_preset` calls `update_compatible` with the filament mode `Always`, because
     `show_incompatible_presets` is off (`Tab.cpp:7705-7712`).
  2. With `remember_printer_config` on, its default (`AppConfig.cpp:544-545`), the printer's
     remembered filaments come back (`Tab.cpp:7739-7742`, `PresetBundle::update_selections`
     `PresetBundle.cpp:3914-3937`). `update_compatible(Always)` then runs again (`:4002`), and
     any slot still not found, invisible or incompatible takes `first_compatible()`
     (`:4005-4015`).
- `PresetBundle::update_compatible` (`PresetBundle.cpp:8517-8673`), filament branch with `Always`.
  `default_filament_profile` of the printer gives the preferred names (`:8636`).
  - One filament: the slot takes the filament collection's edited preset after
    `PresetCollection::update_compatible` (`Preset.hpp:877-885`). An incompatible or missing
    preset is replaced by `first_compatible(PreferedFilamentsProfileMatch(preset, preferred))`
    (`:8600-8628`).
  - More filaments: each slot whose preset is missing or incompatible takes
    `first_compatible(PreferedFilamentProfileMatch(preset, preferred[idx] or preferred[0]))`
    (`:8574-8598`, `:8664-8672`).
  - `PreferedFilamentProfileMatch`:
    - a preset with the slot preset's alias wins outright;
    - otherwise the score is 1, or 2 for the preferred name, plus 1 if visible;
    - the score is ×10 for the slot preset's `filament_type`.
  - `PreferedFilamentsProfileMatch`:
    - the alias wins;
    - otherwise the score is 1, or 2 for a preferred name;
    - the score is ×10 for the edited preset's `filament_type`, which is `PLA` without a preset.
  - `first_compatible_idx` (`Preset.hpp:831-855`) scans the name-sorted collection for compatible
    visible presets. The first best score wins.
- The alias of a vendor preset is its `alias` field. Without one, it is the name before `@`,
  trimmed, or the whole name when there is no `@` (`PresetBundle.cpp:7735-7744, 7817-7818`). A
  user preset's alias is its bare name, with the `@` suffix removed only for a root filament
  (`Preset.cpp:4414-4440`).
- System filament visibility is the installed set (`PresetBundle::load_installed_filaments`).
  User presets are visible.

### 3MF values out of range

- `Plater::priv::load_files` (`Plater.cpp:9462-9481`) applies the loaded 3MF config over
  `FullPrintConfig::defaults()` and calls `config.validate()` (`under_cli = false`). It shows
  "Invalid values found in the 3MF:\n" with one `-key: message\n` line per entry, then "\nPlease
  correct them in the Param tabs" as a warning notification. The project still loads.
- GUI slicing calls `BackgroundSlicingProcess::validate` (`BackgroundSlicingProcess.cpp:726-734`),
  which runs `Print::validate` and not `DynamicPrintConfig::validate`.
- The CLI rejects the values with `m_print_config.validate(true)` (`OrcaSlicer.cpp:4277-4282`) and
  exit code `CLI_INVALID_VALUES_IN_3MF`.

## Destination

- `crates/ares-core/src/profiles/compatible/selection.rs` (new):
  - `PresetCatalogHandle::select_compatible_filaments(printer, process, filaments, installed)`
    returns the filament per slot after `update_compatible(Always)`;
  - the alias and both match scores above;
  - `ares-wasm` exposes it as `PresetCatalog.selectCompatibleFilaments`.
- `crates/ares-core/src/plater`:
  - the plater slice entries run `Print::validate` without the CLI config checks
    (`DynamicPrintConfig::validate(true)` and the CLI override check);
  - `PlaterSession::invalid_values` returns the `config.validate()` map of the loaded project;
  - `ares-wasm` exposes it as `PlaterSession.invalidValues`.
  - The CLI and the non-plater entries keep their checks.
- `web/app`:
  - `selectPrinter` and a project load under a printer selection fill the filament slots from
    `selectCompatibleFilaments`;
  - the filament combo shows only the selected preset;
  - the status bar shows `presetSliceBlocker` as a warning;
  - a 3MF load with invalid values shows the upstream warning text.

  The React code only stores and renders what core returns.

## Included

- Choosing compatible filaments on a printer switch and when a project loads under a selected
  printer, with upstream's alias, preferred-name, visibility and type preferences.
- A filament combo whose value is the selection, and a status bar that names the blocker.
- The GUI split between the CLI config check and `Print::validate` for the plater entries.
- The 3MF invalid-values warning at load.

## Deferred

- Loading a project's own printer, process and filament presets when a printer is selected
  (`load_config_file_config`, `PresetBundle.cpp:6122-6419`). The web keeps the selected printer.
- The process branch of `update_compatible` (`PreferedPrintProfileMatch` with
  `default_print_profile`, `PresetBundle.cpp:8547-8570, 8637-8638`). The web still asks for a
  process when the printer has none remembered.
- The `show_unsupported_presets` and `show_incompatible_presets` preferences, and the
  `OnlyIfWasCompatible` mode they select.
- The spiral-vase correction dialog that replaces the `under_cli` spiral check in the GUI.
- The `update_selections` pass that replaces a remembered filament which is compatible but no
  longer visible with `first_compatible()` (`PresetBundle.cpp:4005-4015`). The web keeps it.
- Slicing on the H2D: its start G-code reads `nozzle_diameter_at_nozzle_id` (`GCode.cpp:1192,
  1292`), which the placeholder config does not have yet. The web test slices on the A1 mini.
- No compatible visible filament for a slot: `first_compatible_idx` then returns preset 0, the
  `-- default --` filament (`Preset.hpp:850-854`). The catalog has no such preset, so the slot
  becomes empty: its combo shows no name, and the status bar and the Slice button say "Select a
  filament preset for every filament to slice".

## Failure modes

- An incompatible filament stays selected or shows as the combo value after a printer switch.
- A slot is left empty when a compatible visible preset exists, so slicing stays blocked.
- The alias match is missing: `Bambu PLA Matte @BBL A1M` becomes the default PLA, not
  `Bambu PLA Matte @BBL H2D`.
- Ties are broken by catalog order instead of preset name order.
- An invisible (not installed) system filament is chosen.
- The status bar says "Ready to slice" while the Slice button is disabled.
- GUI slicing rejects a 3MF value that only the CLI rejects, or the CLI stops rejecting it.
- The load warning is missing, lists only one key, or blocks the load.

## Test plan

| Test | Before | After |
|---|---|---|
| `web/e2e/sliceBlocker.spec.ts`: an A1M project switched to the H2D lists only H2D-compatible filaments, takes the same-alias H2D presets, and names the remaining blocker in the status bar and on the Slice button; back on the A1 mini it slices | F | P |
| `web/e2e/sliceBlocker.spec.ts`: a 3MF with an out-of-range value warns at load and slices | F | P |
| `runner::stock_validation::project_value_out_of_range::a_negative_raft_expansion_in_a_project_fails_like_native` (CLI against Native) | P | P |

The web tests build their 3MF from a committed X1C fixture whose project config they rewrite. They
save screenshots under the web E2E target directory.
