# Web sidebar: preset persistence, filament grouping, plate/nozzle, dual extruder

Status: approved by coordinator review; implementing. Branch `web/ui-presets-persistence`.

This is a web GUI slice under ARD-0106: it ports `src/slic3r/GUI` sidebar
behavior of OrcaSlicer 2.4.2 (pinned `8500fcdcc`). Slicing stays in
`ares-core`. The sidebar's choices reach the slice only as project-config
values (`curr_bed_type`, `nozzle_volume_type`, `filament_map_mode`,
`filament_map`), passed through the existing override boundary
(`ConfigOverrides`, the project-settings layer), and as preset names. Three
small catalog reads move into `ares-core` because upstream implements them in
`libslic3r` (see "Engine API"). All paths below are relative to
`OrcaSlicer/src/`.

## Current state (scaffold being replaced)

- `web/app/src/sidebar/Sidebar.tsx` prints Plate type and Nozzle as read-only
  `kv-row`s. They show "—" whenever there is no project, and with a project
  they show only `nozzle_diameter[0]`.
- `compatibleGroups` puts filaments into two flat lists, "User presets" and
  "System presets", with no vendor or type grouping, no ordering and no search.
- Only `ares.theme`, `ares.mode`, `ares.gcodeWindow` and `ares.vendors` are
  stored. Preset selections, imports, bed type and extruder settings are lost
  on reload.
- A dual-extruder printer (H2D) gets the single-extruder layout. There are no
  per-nozzle groups, no volume type and no `filament_map` control.

## 1. Browser persistence (AppConfig mirror)

Upstream sources:

| Upstream | Persisted value |
| --- | --- |
| `libslic3r/PresetBundle.cpp:2917-2980` `export_selections` | Section `presets`: the selected printer. Per printer (`AppConfig::set_printer_setting`, `AppConfig.hpp:239-249`): `print`, `filament`, `filament_NN`, `curr_bed_type`. |
| `libslic3r/PresetBundle.cpp:2625-2650` `update_selections` + `slic3r/GUI/Tab.cpp:6229-6233` | When `remember_printer_config` is set (default true, `AppConfig.cpp:436-437`), selecting a printer restores that printer's process and filaments. |
| `slic3r/GUI/Plater.cpp:2510-2540` (`Sidebar::update_presets` printer branch) | On printer change the bed type is the printer setting `curr_bed_type`, else `Preset::get_default_bed_type` (`libslic3r/Preset.cpp:966-990`). |
| `slic3r/GUI/Plater.cpp:9431-9470` `on_select_bed_type` | A bed type change is stored globally and per printer. |
| `libslic3r/AppConfig.cpp:1555-1580`, `PresetBundle.cpp:2880-2900`, `slic3r/GUI/Tab.cpp:7121-7138` `TabPrinter::set_extruder_volume_type` | Section `nozzle_volume_types` per base printer, falling back to `default_nozzle_volume_type`. |
| `slic3r/GUI/FilamentGroupPopup.cpp:25-48` | `prefered_filament_map_mode`, default `Auto For Flush` (`AppConfig.cpp:456-457`). |
| `slic3r/GUI/MainFrame.cpp:3723-3745` `load_config_file` → `PresetBundle::import_presets` | Imported user presets survive a restart because Orca writes them to the user folder. Ares stores the imported files instead. |
| `AppConfig.cpp:301, 330`; `GUI_App.cpp:7775-7802` `user_mode` | UI preferences `show_gcode_window`, dark mode and settings mode. |

Design:

- `web/app/src/state/persistence.ts` is the only module that touches browser
  storage. It holds one `localStorage` record, `ares.config`:
  `{ version: 1, ui: {theme, mode, gcodeWindow}, vendors, rememberPrinterConfig,
  preferredFilamentMapMode, selected: {printer} | null,
  printers: { [printerName]: {process?, filaments[], currBedType?} },
  nozzleVolumeTypes: { [basePrinter]: string[] } }`. The record is small and is
  read synchronously before the first render, so the theme does not flash.
- Imported preset files go to IndexedDB (database `ares`, store
  `presetImports`, entries `{version: 1, filename, bytes}`), because
  `.orca_bundle` archives can be larger than the localStorage quota. They load
  at startup in import order through the existing `parsePresetBundle` and
  `catalog.addImport`.
- Validation happens only at this boundary. If a record has the wrong
  `version`, does not parse as JSON, or has a field of the wrong shape, it is
  deleted and the defaults are used. An import entry that fails to parse is
  deleted and the other imports still load. A stored printer, process or
  filament name that is missing from the catalog is dropped, the same way
  `select_preset_by_name_strict` drops it.
- The old `ares.theme`, `ares.mode`, `ares.gcodeWindow` and `ares.vendors` keys
  are removed and not migrated (AGENTS: no legacy fallback). The coordinator
  allowed dropping old entries.
- Writes happen when a selection action runs (`selectPrinter`,
  `selectFilament`, bed type, and so on), mirroring `export_selections` after
  each `update_presets`. There is no write-on-every-render.
- Process edits made in the sidebar (`overrides`) are not persisted. Orca
  discards unsaved preset edits too (UnsavedChangesDialog). See D3.

## 2. Filament list grouping, classification and search

Upstream `slic3r/GUI/PresetComboBoxes.cpp`, `PlaterPresetComboBox::update`
(`:1105-1495`):

- `:1170-1186` lists only visible presets, plus the selected one.
  `:1233-1236` keeps incompatible presets in a separate list that is shown
  only when `show_unsupported_presets` is set (`:1429-1430`, default false,
  `AppConfig.cpp:347`). The list is therefore compatible-only for the
  selected printer (and process).
- `:1216-1226` caches `filament_vendor[0]` (with "Bambu Lab" shown as
  "Bambu") and `filament_type[0]` for every preset.
- `:1313-1350` orders "System presets" by `filament_orders` (12 Bambu names),
  then `first_vendors {"", "Bambu", "Generic"}`, then
  `first_types {PLA, PETG, ABS, TPU}`, then name.
- `:1351-1407` groups system presets into sub-menus by vendor (`groupName2`,
  the vendor). User presets follow `group_filament_presets`:
  `0` All ("Custom"), `1` None (default, `AppConfig.cpp:440-441`), `2` by type,
  `3` by vendor. Within a group they sort case-insensitively with empty keys
  last, and "Unspecified" is the label for an empty key.
- `:1410-1430` sets the section order: Project-inside, User, Bundle, System,
  Unsupported. `:1456-1461` adds the trailing "Add/Remove filaments" item.

The filament selection dialog is opened by "Add/Remove filaments"
(`LABEL_ITEM_WIZARD_FILAMENTS`, `PresetComboBoxes.cpp:955`). It is the
guide page `resources/web/guide/23/index.html` together with `22/common.js`:

- `SortUI` (`22/common.js:22-214`) builds a printer column, a filament-type
  column and a vendor column. Generic vendors come first,
  `FilamentPriority = [pla, abs, pet, tpu, pc]`, and `VendorPriority = [generic]`
  (`:3-4`). A filament is listed only when its `models` match a selected
  printer model and nozzle (`:86-117`). That is the compatible-only rule.
- `initInputEvents` (`:539-600`) implements search: each column has a search
  bar that hides items not containing the lower-cased query, and the item
  column has a filter with `::checked` / `::unchecked`.
- The checked set is the installed filament set (`AppConfig` section
  `filaments`). `PresetBundle::load_installed_filaments`
  (`libslic3r/PresetBundle.cpp:2541-2600`) seeds it from the printer model's
  `default_materials` when no compatible filament is installed, and
  `set_visible_from_appconfig` turns it into `Preset::is_visible`.

Design:

- `web/app/src/sidebar/filamentGroups.ts` is a pure module that ports
  `:1216-1226` and `:1313-1407`. Its input is the compatible names, each
  name's `{filament_type, filament_vendor}` and the user-name set. Its output
  is `PresetGroup[]` with optional `subgroup` labels.
- `PresetCombo` renders sub-groups as nested `role="group"` sections and gains
  a search box. The combo itself has no search upstream; its matching rule is
  the dialog's lower-case substring test. Search is used only for filament
  combos.
- `web/app/src/sidebar/FilamentDialog.tsx` is a thin component over
  `filamentDialog.ts`. It shows the type, vendor and filament columns with
  search, limited to the selected printer's compatible filaments, and
  persists the checked set as `installedFilaments`. The combos hide filaments
  that are not installed (D1).
- A new Preferences toggle for `group_filament_presets` is out of scope.
  Ares uses the upstream default `1` (None) for user presets.

## 3. Plate type and nozzle

Upstream:

- `slic3r/GUI/Plater.cpp:2793-2835` `Sidebar::reset_bed_type_combox_choices`:
  the options are the `curr_bed_type` enum labels (`libslic3r/PrintConfig.cpp:1080-1098`,
  keys `:469-477`) minus the printer model's `not_support_bed_types`
  (`VendorProfile::PrinterModel`, read at `libslic3r/PresetBundle.cpp:4795-4810`).
  When the model is not known, all labels are shown.
- `Plater.cpp:641-646` shows the plate combo only when
  `PresetBundle::is_bbl_vendor()` (`PresetBundle.cpp:612-640`: the printer
  model belongs to the BBL vendor) or when `support_multi_bed_types` is set.
  Otherwise the combo is disabled and the plate is the default bed type
  (`:2534-2539`).
- `Plater.cpp:2645-2700` (`update_presets` printer branch) fills the nozzle
  options from `PresetCollection::diameters_of_selected_printer`
  (`libslic3r/Preset.cpp:3319-3328`): the `printer_variant` of every preset
  with the same `printer_model`, with the current `nozzle_diameter` added if
  it is missing. `Plater.cpp:652` shows the single nozzle combo only when the
  printer has one extruder.
- `Plater.cpp:1243-1300` `Sidebar::priv::switch_diameter` and
  `:1862-1876`: picking a diameter selects
  `PresetBundle::get_similar_printer_preset({}, diameter)`
  (`PresetBundle.cpp:3750`). Ares already ports this as
  `similarPrinterPreset`. When no preset matches, the "Configuration
  incompatible" warning is shown.

Design: `web/app/src/sidebar/printerOptions.ts` (pure) and
`PrinterSection.tsx`. The bed type is sent as the `curr_bed_type` override
for both preset slices and project slices. Changing the nozzle selects a
printer through `selectPrinter`. In project mode (a 3MF without a preset
selection) the model data comes from the project printer's `printer_model`.
If that vendor is not installed, the model is unknown: all bed types are
offered and the nozzle combo lists only the project's diameter.

## 4. Dual-extruder sidebar (H2D)

Upstream:

- `Plater.cpp:2645-2648`: the dual layout is used when `nozzle_diameter` has
  two values and the vendor is BBL. `layout_printer` (`:584-660`) then shows
  `ExtruderGroup`s "Left Nozzle" and "Right Nozzle"
  (`:2044-2046`, titles from `DevPrinterConfigUtil::get_toolhead_display_name`)
  and hides the single nozzle panel. `filament_map` value 1 is left and 2 is
  right (`FilamentMapPanel.cpp:42-50`). `MAIN_EXTRUDER_ID` 0 is the right
  nozzle and `DEPUTY_EXTRUDER_ID` 1 is the left (`DeviceCore/DevDefs.h:53-54`).
- `ExtruderGroup` (`Plater.cpp:418-460, 1023-1140`) has a Diameter combo per
  nozzle. `update_extruder_diameter` (`:2651-2671`) selects
  `nozzle_diameter[i]`. When the left and right diameters differ,
  `switch_diameter` (`:1249-1272`) asks which nozzle's diameter to use for
  both, because one print cannot mix diameters.
- The Flow combo (`combo_flow`, which drives `nozzle_volume_type[i]` through
  `TabPrinter::set_extruder_volume_type`) exists but is hidden in Orca
  (`Plater.cpp:1122-1123`, "Orca hack"). The values are the `NozzleVolumeType`
  enum Standard and High Flow (`PrintConfig.cpp:5348-5359`). The valid
  values for extruder `i` are those whose
  `get_extruder_variant_string(extruder_type[i], type)` (`PrintConfig.cpp:588-606`)
  appears in `extruder_variant_list[i]`. See D2.
- Filament map mode: `FilamentGroupPopup.cpp:89-120` offers Filament-Saving
  (`Auto For Flush`), Convenience (`Auto For Match`) and Custom (`Manual`).
  `Auto For Match` needs a connected printer and falls back to Auto For Flush
  (`:250-253`), so it is disabled in the browser. The global mode and map are
  stored in the project config (`Plater.cpp:16949-16975`). In Manual mode,
  `FilamentMapDialog` / `FilamentMapManualPanel` (`FilamentMapPanel.cpp:21-97`)
  shows a Left Nozzle list and a Right Nozzle list of filaments, and the
  switch button swaps them (`:81-97`).

Design: `web/app/src/sidebar/ExtruderGroups.tsx` (two groups: diameter, plus
flow per D2) and `FilamentMapPanel.tsx`. The panel has a mode
radio and, in Manual mode, the left and right lists with "move to other
nozzle" buttons, plus drag where the pointer supports it. The logic lives in
`web/app/src/state/extruders.ts`. The values go to the slice as the
`nozzle_volume_type`, `filament_map_mode` and `filament_map` overrides.
`ares-core` already consumes them: `options/extruder_index.rs`,
`options/filament_map_recommend.rs`, and `options/different_extruders.rs`.

Engine check, done first in implementation: the H2D E2E below compares the
web G-code to the official OrcaSlicer G-code with `nozzle_volume_type =
High Flow` on one nozzle and Manual `filament_map`. If `ares-core` does not
resolve the per-extruder variant values from an overridden
`nozzle_volume_type` the way `PresetBundle::full_fff_config`
(`PresetBundle.cpp:3884-4045`, `update_values_to_printer_extruders`) does,
I stop and send a QUESTION. I will not patch the slicing path from this
branch.

## Engine API (ares-core → ares-wasm → @ares-slicer/wasm)

All new reads are on the existing `PresetCatalog` handle. Each one is a port
of a `libslic3r` read.

- `presetValues(kind, names, keys)` returns the resolved values (after
  `inherits`) of catalog presets, so TypeScript never parses preset bodies.
  This is `Preset::config` after `PresetCollection::load_presets`. It
  supplies `filament_type`, `filament_vendor` and `filament_colour` for
  grouping, and the printer values `printer_model`, `printer_variant`,
  `nozzle_diameter`, `support_multi_bed_types`, `default_bed_type`,
  `extruder_type`, `extruder_variant_list` and `default_nozzle_volume_type`.
- `printerModelInfo(model)` returns `{vendor, not_support_bed_types,
  default_materials, nozzle_diameters}`: `get_current_vendor_type`
  (`PresetBundle.cpp:612-640`), `VendorProfile::PrinterModel`
  (`:4785-4810`) and `diameters_of_selected_printer` (`Preset.cpp:3319-3328`).
  `vendor` is the vendor index `name` (e.g. "Bambulab"). The vendor
  envelope carries no file stem, so `is_bbl_vendor` maps that name back to
  its `@ares-slicer/profiles` id (`BBL`) in TypeScript.
- `installedFilaments(installed)` (D1) is the seed of `load_installed_filaments` (`PresetBundle.cpp:2541-2600`).

Destination: `crates/ares-core/src/profiles/catalog/{vendor_models,model_info,preset_values}.rs`,
with `profiles/compatible/installed.rs` for the seed. `compatible.rs` was
already over 400 lines, so its compatibility checks move to
`compatible/checks.rs`. `crates/ares-wasm/src/profiles.rs` is already
397 lines, so the catalog bindings move to `profiles/catalog.rs`. The JS
wrapper goes in `web/packages/ares-wasm-js/src/presetCatalog.ts`.

## Module layout (each file ≤400 lines)

`Sidebar.tsx` is split into `PrinterSection.tsx`, `FilamentSection.tsx` and
`ProcessSection.tsx`. Logic lives in `state/persistence.ts`,
`state/extruders.ts`, `sidebar/filamentGroups.ts`, `sidebar/printerOptions.ts`
and `sidebar/filamentDialog.ts`. The new store fields are
`plate: {bedType}`, `extruders: {nozzleVolumeType[], filamentMapMode,
filamentMap[]}` and `installedFilaments`.

## E2E plan (Playwright, system Chrome via `ARES_E2E_CHROME`)

Each spec writes screenshots plus a JSON record under `target/web-e2e/<spec>/`.

1. `persistence.spec.ts`
   - Install BBL, pick X1C, change the process, two filaments, Textured PEI
     plate, an imported user preset file, and the dark/expert UI prefs. Reload.
     Every combo and pref is restored, the import is listed under User presets,
     and the screenshots before and after are recorded.
   - Switch to another printer and back. Its own process and filaments are
     restored (`remember_printer_config`).
   - Corruption cases: write a `version: 0` record, invalid JSON, a bad field
     type and a corrupt IndexedDB import, then reload each time. The app starts
     with defaults, there are no `pageerror` events, and the bad entries are
     gone.
2. `filamentGroups.spec.ts`
   - The order and vendor sub-groups of the X1C filament combo equal a list
     computed in the test from the vendor JSON with the `:1313-1350` sort rules.
   - User presets come before system presets.
   - An incompatible filament (another printer's `@BBL H2D` preset) is absent.
   - Searching "petg" leaves only PETG items.
   - The dialog's type and vendor filters and search work, and an
     installed-set change survives a reload.
   - Screenshots of the open combo and the dialog.
3. `plateNozzle.spec.ts`
   - The plate options for a BBL printer equal the enum labels minus
     `not_support_bed_types`. A non-BBL printer without
     `support_multi_bed_types` has no plate combo.
   - Picking 0.6 mm selects the "… 0.6 nozzle" preset.
   - Slicing with Textured PEI: the G-code `curr_bed_type` and bed temperature
     equal OrcaSlicer `--load-settings` (`ORCA_APPDIR`) with the same bed type.
4. `dualExtruder.spec.ts` with `tests/parity/master-extruder-id/stock-right-only.3mf`
   (H2D, 2 filaments)
   - The Left and Right Nozzle groups render.
   - Manual map: TPU goes left and PLA goes right.
   - The right nozzle is set to High Flow (D2).
   - The slice is compared to official OrcaSlicer on the same project with the
     same `filament_map_mode`/`filament_map`/`nozzle_volume_type`, using the
     normalized byte comparison from `presetSlice.spec.ts`.
   - Reload restores the map mode and volume types.
   - Screenshots.

Also run: `npm run lint`, `npm run typecheck`. When Rust is touched, also run
`cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo nextest run -p ares-wasm`. Heavy work is pinned with
`taskset -c 4-7 nice -n 10` and `CARGO_BUILD_JOBS=4`.

## Commits (order)

1. `docs(web)`: this spec.
2. `feat(core)` + `feat(wasm)`: `presetValues` and `printerModelInfo`, with the
   `profiles.rs` split.
3. `feat(web)`: persistence module and IndexedDB imports, plus E2E 1.
4. `feat(web)`: filament grouping, search and dialog, plus E2E 2.
5. `feat(web)`: plate type and nozzle, plus E2E 3.
6. `feat(web)`: dual-extruder groups and filament map, plus E2E 4.

## Included / deferred

Included: everything above. Deferred: Bundle presets
(`.orca_bundle` bundle grouping `:1424-1425`), the `group_filament_presets`
and `show_unsupported_presets` preference UI, AMS/printer sync
(`add_ams_filaments`, `sync_extruder_list`, Auto For Match), per-plate bed
type and map mode (`PartPlate`), flush-volume matrices in persistence, and
cross-device sync.

## Decisions (coordinator review of the spec)

- D1 (Q1). The installed filament set hides combo items, as upstream
  `is_visible` does. A fresh install is seeded from the printer model's
  `default_materials` (`PresetBundle::load_installed_filaments`), and the
  dialog adds or removes filaments.
- D2 (Q2). The Flow (volume type) combo is shown for each nozzle, but only when
  that nozzle's `extruder_variant_list` entry offers more than one volume
  type.
- D3 (Q3). Process edits are not persisted, the same as Orca's discard of
  unsaved preset edits.
