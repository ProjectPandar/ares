# Web: first-run setup wizard and per-model printer installation

This web slice ports OrcaSlicer's setup guide and its installed-printer model, pinned to
v2.5.0-beta `345efec64`. Paths are relative to `OrcaSlicer/`.

- The visibility and installation rules are `libslic3r` code, so they move into `ares-core` as
  catalog API.
- The guide pages are GUI code. They become React components that call that API.

## Current state (scaffold being replaced)

- `web/app/src/sidebar/VendorDialog.tsx` lists vendors with one checkbox each. A checked vendor
  loads all its presets, so every printer of that vendor shows in the printer combo.
- `ares.config` stores `vendors: string[]`.
- `PresetCatalogHandle::printers` (`crates/ares-core/src/profiles/catalog.rs`) lists every loaded
  printer. Its doc comment says the per-model/variant install matrix has no Ares counterpart.
- There is no first-run flow. A fresh browser starts with no vendor and an empty printer combo.

All of this is replaced. `VendorDialog`, the `vendors` field and that doc comment are removed.

## Upstream boundary

### When the wizard runs

- `GUI_App::config_wizard_startup` (`src/slic3r/GUI/GUI_App.cpp:10129-10135`) runs the wizard
  when there is no app config, or when `PrinterPresetCollection::only_default_printers`
  (`src/libslic3r/Preset.cpp:4599-4606`) holds. That means no visible printer whose name does not
  start with "Default", and user printers count as visible.
- `GUI_App::run_wizard` (`GUI_App.cpp:9930-9967`) maps start pages to `GuideFrame` pages:
  `SP_PRINTERS` opens `BBL_MODELS_ONLY`, and a first run opens the full guide.
- `GuideFrame::SetStartPage` (`src/slic3r/GUI/WebGuideDialog.cpp:325-370`) maps those pages to
  `resources/web/guide/0/index.html?target=` values:

  | Page | Target |
  | --- | --- |
  | `BBL_WELCOME` | `1` |
  | `BBL_REGION` | `11` |
  | `BBL_MODELS` | `21` |
  | `BBL_FILAMENTS` | `22` |
  | `BBL_FILAMENT_ONLY` | `23` |
  | `BBL_MODELS_ONLY` | `24` |

- Until the guide has finished once (`firstguide/finish`), the dialog has no close box
  (`GUI_App.cpp:9958-9961`). Escape closes only pages `23` and `24` (`WebGuideDialog.cpp:271`).
- Sidebar entry points that reopen the guide:
  - the printer panel title's `settings` button runs `run_wizard(RR_USER, SP_PRINTERS)`
    (`src/slic3r/GUI/Plater.cpp:2851-2857`), page `24`;
  - the printer combo's "Select/Remove printers (system presets)" item, `LABEL_ITEM_WIZARD_PRINTERS`
    (`src/slic3r/GUI/PresetComboBoxes.cpp:1026-1037`), opens the same page;
  - the filament panel title's `settings` button, tooltip "Set filaments to use", runs
    `run_wizard(RR_USER, SP_FILAMENTS)` (`Plater.cpp:3379-3387`), page `23`.
- The sidebar's printer edit icon is `btn_edit_printer` (`Plater.cpp:2933-2939`). It calls
  `switch_to_tab` and opens no menu. The combo's own `cog` button with `show_edit_menu` and its
  "Add/Remove presets" item (`PresetComboBoxes.cpp:978-996`, `:1137-1160`) is never added to
  `panel_printer_preset`'s sizer (`Plater.cpp:1043-1048`), and is hidden on Linux. So the sidebar
  reaches the models page only through the title button and the combo item.

### Selection after the wizard and after opening a project

- A printer selection, from Finish, Confirm or the combo, ends in
  `PresetBundle::update_compatible` (`src/libslic3r/PresetBundle.cpp:8632-8640`).
  `load_selections` leaves the print to that call (`:4034-4041`).
  - For prints it runs `PresetCollection::update_compatible` with
    `PresetSelectCompatibleType::Always` (`src/libslic3r/Preset.hpp:877-885`).
    `update_compatible_internal` (`Preset.cpp:3965-4010`) keeps a compatible selected print and
    unselects an incompatible one.
  - Without a selected print, `first_compatible_idx` (`Preset.hpp:831-855`) takes the first
    compatible, visible print of the highest `PreferedPrintProfileMatch` quality
    (`PresetBundle.cpp:8522-8572`):
    - a print with the previous print's non-empty alias wins outright. A system print's alias is
      its name before `@` (`PresetBundle.cpp:7735-7742`); a user print's is its whole bare name
      (`PresetCollection::set_custom_preset_alias`, `Preset.cpp:4414-4437`);
    - otherwise the quality is 1 for the printer's `default_print_profile`, plus 2 for a
      compatible visible print, times 10 when its `layer_height` is within 0.0005 of the previous
      print's.
  - With no print selected yet, the previous print is the collection's "- default -" preset
    (`m_idx_selected(0)`): no alias, and `PrintConfigDef`'s `layer_height` default.
  - The filament slots then go through the same call's filament pass, which
    `crates/ares-core/src/profiles/compatible/selection.rs` already ports: a compatible slot
    stays, and an incompatible one takes the preferred compatible filament.
- Opening a project: `Plater::priv::load_files` loads the project config when there is one
  (`src/slic3r/GUI/Plater.cpp:9514-9515`) through `PresetBundle::load_config_model` (`:9623`).
  That makes the project's presets current. A model file without a config leaves the selection.

### Model pages (`21`, `24`)

Source: `resources/web/guide/21/common.js`.

- `HandleModelList` (`:70-139`):
  - models are sorted by vendor, with `Custom` first;
  - there is one block per vendor;
  - a model is pre-checked when its `nozzle_selected` is non-empty, meaning it is installed.
- `CreateVendorBlock` (`:269-290`):
  - shows a "`n` / `m`" count and a vendor checkbox;
  - the vendor display name maps `BBL` to "Bambu Lab", `Custom` to "Custom Printer" and `Other`
    to "Orca colosseum".
- `ChooseVendor` / `UpdateVendorCheckbox` (`:348-370`): the vendor checkbox checks or unchecks all
  of that vendor's models, and shows as indeterminate when only some are checked.
- `CreatePrinterBlock` (`:292-326`):
  - one checkbox per model;
  - the nozzle diameters appear only as text ("Nozzle", values joined with " · ");
  - the display name drops the vendor prefix.
- `UpdateSidebarVendors` / `scrollToVendor` (`:328-346`): a vendor sidebar.
- `FilterModelList` (`:185-264`): a search box.
- Page `21` has Next, which calls `GotoFilamentPage` (`21/21.js:13-18`). Page `24` has Confirm
  and Cancel (`24/24.js`).
- Both pages call `OnExitFilter` (`common.js:416-444`), which sends
  `save_userguide_models {model: {model}}` and returns the number of checked models. With none
  checked, the button does nothing.
  - The "At least one printer must be selected." notice (`21/index.html:88-96`) is shown only by
    `OnExit` (`common.js:372-385`), which nothing calls in this release.

### Selection to installed variants

- `save_userguide_models` (`WebGuideDialog.cpp:607-660`):
  - sets every checked model's `nozzle_selected` to its full `nozzle_diameter` list;
  - marks that model's `default_materials` as selected filaments, when they exist in the filament
    list.
- `SaveProfile` (`WebGuideDialog.cpp:910-970`) writes the filament section, then clears the
  vendors map and calls `set_variant(vendor, model, nozzle, "true")` for each selected nozzle.
- `GuideFrame::apply_config` (`WebGuideDialog.cpp:1005-1212`) picks the preferred printer. It
  takes the first enabled `(model, variant)` that is new compared with the previous vendors map,
  looking at the default bundle (`ORCA_DEFAULT_BUNDLE = "Custom"`, `PresetBundle.cpp:187`) first and
  then the other vendors in map order
  (`:1066-1124`). Then it calls `PresetBundle::apply_vendor_config` with `overwrite = true`.

### libslic3r

- `PresetBundle::apply_vendor_config` (`src/libslic3r/PresetBundle.cpp:1952-2095`):
  - installs the vendor bundles that have any model;
  - replaces each `X @System` filament with `X` when a selected non-default vendor's
    `default_filaments` contains `X` (`:1990-2048`). Only `VendorProfile::from_ini` fills
    `default_filaments` (`Preset.cpp:466-474`, the INI `[default_filaments]` section), and the
    JSON vendor loader never does. Every vendor Ares loads is JSON, so this step keeps every name
    and Ares stores the checked filaments as they are;
  - stores the vendors and filaments (`:2050-2074`);
  - reloads presets with the preferred printer (`:2076`);
  - makes sure the active filament is one of the selected ones (`:2079-2095`).
- `Preset::set_visible_from_appconfig` (`src/libslic3r/Preset.cpp:967-995`):
  - a system printer with a non-empty `printer_model` and `printer_variant` is visible exactly
    when `AppConfig::get_variant(vendor, model, variant)` is set;
  - a preset without a vendor (a user preset), or with an empty model or variant, keeps its
    visibility.
- `PresetBundle::load_installed_printers` (`PresetBundle.cpp:3685-3690`) applies that rule to
  every printer.
- `PresetBundle::load_selections` (`PresetBundle.cpp:4021-4070`) selects the preferred printer via
  `PrinterPresetCollection::find_system_preset_by_model_and_variant` (`Preset.cpp:4569`).
  - Without a preferred printer, it selects the stored printer.
  - `PresetCollection::select_preset_by_name` (`Preset.cpp:4230-4249`) selects the first visible
    printer when the named one is missing or not visible.
- `PresetBundle::load_installed_filaments` is already ported (`profiles/compatible/installed.rs`).
  It seeds the installed filaments with the visible printers' `default_materials`.

### Filament page (`22`)

- `GuideFrame::BuildProfileJson` (`WebGuideDialog.cpp:1385-1460`) tags each system filament with
  the `[model++variant]` pairs of its `compatible_printers`.
- Page `22` (`22/common.js:80-140`) shows a filament when its tag is empty, or names a selected
  `(model, variant)`.
- `FinishGuide` → `ResponseFilamentResult` (`22/common.js:486-520`) refuses to finish when no
  filament is checked, and shows "At least one filament must be selected." (`22/index.html:115`).

## Destination

### Core (`crates/ares-core/src/profiles/catalog/`)

- `installed_printers.rs`: `InstalledPrinters`, the `AppConfig` vendors section
  (`vendor → model → variants`, ordered maps, as in `AppConfig::VendorMap`).
  - `PresetCatalogHandle::set_installed_printers` stores it. A catalog that never receives one
    lists every loaded printer, as it does today. Visibility only filters these listings: a preset
    named explicitly, as profile slicing does, is found whether or not it is visible.
  - `printers()` and `printer_models()` apply `set_visible_from_appconfig`.
  - `system_printer(model, variant)` ports `find_system_preset_by_model_and_variant`.
  - Imported (user) printers stay visible.
- `guide.rs`:
  - `guide_models(vendor model files)` reads `VendorProfile::PrinterModel` (id, name, variants,
    `default_materials`) with the existing reader in `vendor_models.rs`, so TS never parses the
    files.
  - `guide_filaments()` lists the loaded system filaments with `filament_vendor`,
    `filament_type` and the `(model, variant)` tags of `BuildProfileJson`.
- `compatible/print_selection.rs` (`crates/ares-core/src/profiles/`):
  `PresetCatalogHandle::compatible_print(printer, current)` returns the print a selection of
  `printer` holds after `update_compatible` with `Always`. That is `current` while compatible,
  else the best `PreferedPrintProfileMatch` print, or `None` when no print is compatible. A
  `current` that is not a loaded print stands for "- default -".
- The rule that decides whether the wizard runs, `only_default_printers`, is the core's
  `printers()` being empty. Ares has no "Default Printer" preset, so "only defaults" means none.

### Data (`web/packages/ares-profiles`)

- `loadPrinterModels(): Promise<{ vendor, files }[]>`: each vendor's `machine_model_list` files,
  emitted at build time as one small chunk.
  - It is data only, so the wizard lists every vendor's models without loading the vendors'
    presets.

### WASM and worker

- Bindings on the preset catalog for:
  - `setInstalledPrinters`;
  - `guideModels`;
  - `guideFilaments`;
  - `systemPrinter`;
  - `compatiblePrint`.

### App (`web/app/src`)

- `wizard/SetupWizard.tsx`: a modal with two steps, models (pages `21` / `24`) then filaments
  (page `22`).
- `wizard/ModelPage.tsx`:
  - vendor sidebar, search, vendor blocks with the count and the tri-state vendor checkbox;
  - model cards with the nozzle text and Orca's display-name rules.
- `wizard/FilamentPage.tsx`: the page-`22` list and its notice.
- `state/wizard.ts`: the guide's models and filaments, and
  `finishWizard(installed, filaments, models)`, which mirrors `apply_config` →
  `apply_vendor_config`, including the active-filament check (`PresetBundle.cpp:2079-2095`) on the
  app's selection.
- `state/presets.ts`: the catalog loads the vendors that have an installed model, plus
  `OrcaFilamentLibrary`, and sets the installed printers on it. `selectPrinter` asks the core's
  `compatiblePrint` for the print. It passes the printer's remembered print, else the previous
  selection's, so React makes no preference decision.
- `state/actions.ts`: opening a 3MF with a project config calls `selectPrinter(null)`, the
  existing "Project presets" selection.
- `sidebar/PrinterSection.tsx`: the title `settings` button and the "Select/Remove printers
  (system presets)" combo item open the wizard at the models-only page.
- `sidebar/FilamentSection.tsx`: the title `settings` button, "Set filaments to use", opens the
  existing filament dialog (`sidebar/FilamentDialog.tsx`, the page-`23` port), as the combo's
  "Add/Remove filaments" item does.

## Behaviour

### First run

The wizard opens modally before Prepare is usable when either holds:

- there is no `ares.config` record;
- no system printer is visible and no imported printer exists.

It has no close button and Escape does nothing. The steps are:

1. **Models.** Next with no checked model does nothing, as upstream's `OnExitFilter` does.
2. **Filaments.** The list is pre-checked with:
   - the installed filaments;
   - the checked models' `default_materials`.

   Finish with none checked shows "Error / At least one filament must be selected." with OK.
   The wizard stays open.

### Finish

1. Each checked model becomes `installed[vendor][model] = every nozzle variant`.
2. The filament section is the checked filaments.
3. Both are stored, and the catalog reloads the vendors that have a model.
4. The preferred printer (`apply_config`'s rule) is selected.
5. The printer combo lists exactly the installed models, and the nozzle choices are exactly their
   variants.

### Re-open

The printer title button and the "Select/Remove printers (system presets)" item open the
models-only page with the installed models checked. Confirm writes the new installed set
(`overwrite`). The filament section becomes the union of two sets: the installed filaments,
which `SaveProfileData` pre-selects
(`WebGuideDialog.cpp:1743-1748`), and the checked models' `default_materials` (`:654`). Cancel
or Escape closes it unchanged.

Nothing is persisted or applied before Finish or Confirm. Upstream's `OnExitFilter` sends
`save_userguide_models` even with no model checked, but that changes only `GuideFrame`'s
in-memory `nozzle_selected`. `ares.config` and the catalog change only when the wizard finishes.

- If the selected printer is no longer visible, the first visible printer is selected
  (`load_selections`).

"Set filaments to use" opens the filament dialog with the installed filaments checked. Done
stores the checked set, and the filament combos list it.

### Print and project presets

- Each printer selection takes its print from `compatiblePrint`. After a first Finish of Generic
  Klipper Printer, that is MyKlipper's `default_print_profile`, "0.20mm Standard @MyKlipper"
  (`Custom/machine/fdm_klipper_common.json:233`), stored with the printer.
- Opening a 3MF that has a project config switches every combo to its "Project presets" entry,
  and the plate slices with the project config. The stored selected printer becomes `null`, as
  when the user picks the project printer in the combo. An STL keeps the selected printer.

### Persistence

`docs/architecture/web-ui-presets-persistence.md` is updated as follows:

- `ares.config` gains `installedPrinters: { [vendor]: { [model]: string[] } }`, the vendors
  section.
- The `vendors` field is removed. A stored record that still has `vendors` loses it on load: no
  printer is installed and the wizard opens.
  - This is the user's decision: a vendor-level selection is dropped, not expanded into models.
  - There is no legacy path (AGENTS).
- Other fields of such a record (theme, mode, per-printer settings) are kept, because they still
  match their schema.
- A stored selected printer that is not visible is dropped, as `select_preset_by_name_strict`
  does.

## Included and deferred

Included:

- the startup check;
- model pages `21` / `24`, with sidebar, search and vendor tri-state;
- the filament page `22`, with its notice;
- `save_userguide_models`, including default materials;
- `SaveProfile`;
- the `apply_config` preferred printer;
- the `apply_vendor_config` active-filament check (its `@System` substitution is a no-op for JSON
  vendors, see above);
- `set_visible_from_appconfig` for printers;
- `find_system_preset_by_model_and_variant`;
- `load_selections`' visible-printer fallback;
- the print kept or chosen by `update_compatible` with `Always` and `PreferedPrintProfileMatch`;
- the switch to the project's presets when a project config loads.

Deferred:

- **Model cover images.** `<vendor>/<model>_cover.png` is 443 files, 18 MB. Cards show upstream's
  fallback, `guide/img/printer-dummy.png`.
- **Other guide pages.** Welcome `1`, region `11` and the stealth mode setting. The filament-only
  page `23` stays the existing filament dialog. Its list is the selected printer's compatible
  filaments rather than page `22`'s `[model++variant]` filter over the installed models, and it
  has Done instead of Confirm and Cancel.
- **Guide extras.** The layout modes (`LayoutMode`), "Check for new printers", "Create" printer,
  and custom filament creation.
- **The filament list's scope.** Upstream builds it from every resource vendor; Ares lists the
  loaded vendors (the vendors of the checked models, plus `OrcaFilamentLibrary`). A filament of
  an unselected vendor whose `compatible_printers` is empty is therefore not listed.
- **Translations.** Strings are English, the `T(key, default)` defaults.
- **Matching a project's presets to loaded presets.** `load_config_model` matches the project's
  printer, print and filaments to system presets by name and embeds the rest as project presets
  (`PresetBundle.cpp`, `load_config_file_config`). Ares always selects the project config as a
  whole, the "Project presets" entry, and does not match system or embedded presets.

## Verification

A core API test (`crates/ares-core/tests/installed_printers.rs`) checks, over public catalog API:

- installing one model of a loaded vendor makes exactly its variants' printers visible;
- imported printers stay visible;
- `system_printer` finds the preset by model and variant;
- `guide_models` lists the vendor's models with their nozzle variants and default materials.

Its expected lists are computed in the test from the vendor JSON.

A Playwright test (`web/e2e/printerWizard.spec.ts`) checks:

1. A cleared profile opens the wizard, and it cannot be closed.
2. Checking one model and finishing makes the combo list exactly that model. The nozzle choices
   are exactly its `nozzle_diameter` variants.
3. Re-opening shows that model checked. Adding a second model adds exactly that model and its
   variants.
4. With every model unchecked, Confirm leaves the wizard open and the installed set unchanged.
   Finish with no filament shows "At least one filament must be selected.".
5. A reload keeps the selection and does not open the wizard. A stored `vendors`-only record
   opens the wizard.
6. A plate slices with the chosen printer.

Screenshots go in `target/web-e2e/printer-wizard`.

A second Playwright test (`web/e2e/wizardEntryPoints.spec.ts`) starts from the first-run record
and covers each entry point:

1. The printer title button opens the models page with the installed model checked. Adding
   A1 mini installs exactly its variants and selects it.
2. The combo's "Select/Remove printers (system presets)" item opens the page with both models
   checked. Unchecking A1 mini removes its presets, and the selection falls to the first visible
   printer by name.
3. "Set filaments to use" opens the filament dialog with the installed filaments checked.
   Checking one more installs it, and the filament combo lists it and can select it.

Its screenshots go in `target/web-e2e/wizard-entry-points`.

A core API test (`crates/ares-core/tests/compatible_print.rs`) checks `compatible_print` on the
Custom and BBL vendors, with expectations computed from the vendor JSON:

- with no current print, the printer's `default_print_profile`;
- a compatible current print stays;
- an incompatible print maps to the compatible print of its alias, where layer height alone
  would pick the default profile;
- with no alias match, the current layer height outranks the default profile;
- a user copy of a system print matches by layer height, not by its parent's alias;
- with no prints loaded, `None`.

A Playwright test (`web/e2e/projectPresets.spec.ts`) starts from the first-run record. An STL
keeps the system printer. A project then selects its printer and process as "Project presets",
stores no selected printer, and slices. Screenshots go in `target/web-e2e/project-presets`.

Every other spec starts from the stored result of step 2 (only Custom / Generic Klipper Printer
checked, then Finish), written to a page with no `ares.config` before the app loads. Step 2 asserts
that a real Finish stores exactly that record. It is not upstream's cancel path
(`WebGuideDialog.cpp:1257-1266`), which calls `set_variant` with the preset name
`ORCA_DEFAULT_PRINTER_MODEL` ("MyKlipper 0.4 nozzle", `PresetBundle.cpp:188`) where a
`printer_model` belongs, and so installs no visible printer. The app does not port that path.
