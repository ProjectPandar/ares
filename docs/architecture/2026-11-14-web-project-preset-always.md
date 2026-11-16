# Web project presets: the GUI Always selection

## Problem

The web plater opens and slices a 3MF through the selection `{"from_project": true}`
(`web/app/src/state/projectPresets.ts`, `docs/architecture/web-project-preset-fill.md`).
That selection runs the core port of the OrcaSlicer **CLI** refresh,
`load_project_preset` (`src/OrcaSlicer.cpp:3164-3198`, printer and process; the filament
loop at `:2775-2814`). Its rule keeps every value the project stores unless `--uptodate`
is given or a `different_settings_to_system` entry is non-empty.

The web is the plater, not the CLI. When the GUI opens a project it loads the project
config into its preset collections with `LoadAndSelect::Always`, which refreshes every
unprotected value from the base preset even when all difference entries are empty. The
two rules give different values for the same project, so the browser slice differs from
what OrcaSlicer's plater would slice.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f7`):

- `src/slic3r/GUI/Plater.cpp:9622`: opening a project with settings calls
  `PresetBundle::load_config_model`.
- `src/libslic3r/PresetBundle.hpp:590-591`, `load_config_model`: calls
  `load_config_file_config` with `is_external` true.
- `src/libslic3r/PresetBundle.cpp:6158-6365`, `load_config_file_config`: splits the
  project metadata and loads the process and printer (`:6300`) and a single filament
  (`:6354`) with `PresetCollection::load_external_preset(…, LoadAndSelect::Always, …)`;
  `:6366-6430`, the multi-filament traversal. `:6246-6255` throws "Invalid configuration
  file" when a project with `extruder_variant_list` lists a different number of
  `filament_extruder_variant` and `filament_self_index` entries.
- `src/libslic3r/Preset.cpp:1090-1119` (`Preset::load_external_config`) and
  `:3009-3262` (`profile_print_params_same`, `load_external_preset`): materialization,
  protected keys from `PresetBundle::project_different_keys` (`PresetBundle.cpp:5520-5532`),
  and the selection of an existing or project-local preset. A stored name with no loaded
  preset becomes the project-local preset `<name>(<file>)` (`Preset.cpp:3173-3230`).
- `src/slic3r/Utils/PresetUpdater.cpp:1063-1067`: the GUI's system presets are those of the
  installed vendors and of `Custom` and `OrcaFilamentLibrary`, which are always installed.
- `src/libslic3r/PresetBundle.cpp:5818-5820`, `full_fff_config`: the sliced config names
  the selected process, filament and printer presets.
- `src/OrcaSlicer.cpp:2575-2620, 2653-2654`: the CLI main's check that the process preset
  lists the printer (`CLI_PROCESS_NOT_COMPATIBLE`). The plater has no such check.

The core port of the selection is the Always slice
(`docs/architecture/project-preset-selection-always.md`): the selection
`{"from_project": true, "load_and_select": "always"}` on the existing JSON request entry
(`crates/ares-core/src/profiles/project_refresh.rs`, `selection`), for one or more
filaments.

## Destination

- **Web** (`web/app/src/state/projectPresets.ts`): every 3MF opens and slices with
  `{"from_project": true, "load_and_select": "always"}`. The `{"from_project": true}`
  selection is deleted from the web; there is no branch on the filament count.
- **WASM / JS adapter**: unchanged. The selection stays a JSON string the plater
  session passes to the core (`platerCreateWithProfiles`, `slice_plater_with_profiles`);
  the bindings move bytes and JSON in and out only.
- **Core** (`crates/ares-core/src/project_slice/state.rs`, `prepare_project_source`): the
  port of the CLI main's compatible-printers check (`project_slice/compatibility.rs`,
  `reject_incompatible_process`) runs for every project source today, including the
  Always selection. It moves behind the CLI paths: it does not run when the source carries
  the Always selection (`ProjectPresetRefresh`, `profiles/project_refresh.rs`). The CLI
  selection `{"from_project": true}` and the plain and `--load-settings` CLI slices keep it.

## Native reference for opened projects

Tests of the opened project's presets compare against the GUI route: the preset-select
probe (`ARES_ORCA_PRESET_SELECT_BIN`, `tools/chain-probe/preset-select.patch`) runs
`load_config_model` with `LoadAndSelect::Always` over the vendors the browser has installed
plus `Custom` and `OrcaFilamentLibrary`, and reports the selected and edited presets.
`web/e2e/presetSelectOracle.ts` runs it for a 3MF. The CLI's `load_project_preset` is no
longer a reference for these tests: it keeps project values the plater refreshes and keeps
the stored preset names.

The e2e first-run config installs only `Custom`, so a project of another vendor opens
with project-local presets, in Native as in the browser. The specs that read the opened
project's presets move to the probe:

- `web/e2e/presets.spec.ts` and `web/e2e/projectPresets.spec.ts`: the combos show the
  names the probe selects (`Snapmaker A250 (0.4 nozzle)(default-prisms.3mf)` for
  `default-prisms.3mf`), not the stored ones.
- `web/e2e/sliceBlocker.spec.ts`: its fixture `bbl-jerk-tail-precision/stock-control.3mf`
  lists two `filament_self_index` entries for three `filament_extruder_variant` entries.
  The probe rejects it at `PresetBundle.cpp:6246-6255`, so the plater cannot open it. The
  copies the spec writes list one `filament_self_index` entry per variant, as
  `bbl-type1-tower/two-categories.3mf` does; the probe opens them.

The specs that compare the browser G-code with a CLI-route slice keep that reference whole:
the Native CLI `--slice` of the project (the plate specs through `web/e2e/platerOracle.ts`,
`multiPlate.spec.ts`, `colorChange.spec.ts`, the project edit of `slice.spec.ts`) or
`ares slice` (`import.spec.ts`, `slice.spec.ts`). The CLI route and the plater differ only in
the lines the selected presets decide, listed by key in `web/e2e/selectedPresetLines.ts`:

- `printer_settings_id`, `print_settings_id`, `filament_settings_id`: the selected names
  (`PresetBundle.cpp:5818-5820`);
- `filament_plugin_config_overrides`: a scalar filament option, taken from the first filament
  preset (`PresetBundle.cpp:5726-5735`); the plater's presets hold it, a project config may not;
- `; Printer :` and `; Profile :`: start G-code that prints `[printer_preset]` and
  `[print_preset]`, the selected names (`PrintApply.cpp:1660-1662`).

The expected G-code is the reference with exactly these lines set from the probe's live
selection for the 3MF the browser opened: a config line the selected presets hold is replaced,
or inserted at its sorted place in the config block; one they lack is removed; a start G-code
line is replaced where the reference has it. Every other byte is compared as before, so a
difference on any other line fails. No Native output is committed. A configured Native
reference without the probe fails the test; where the probe is not provisioned (CI) the
`ares slice` comparison is recorded as skipped with its reason, never passed.
`platerSession.spec.ts` opens its project without a preset selection and keeps its reference
unchanged.

`web/e2e/ksrProject.spec.ts` (`sliceProject`) and `web/e2e/presetSlice.spec.ts` (explicit
system presets) do not use the project selection and are unaffected. The specs that open
an H2D project (`dualExtruder`, `estimation`, `persistence`, `arrange`,
`plateArrangeVendors`) depend on a core fix of the Always selection: re-reading the
selected printer's `extruder_printable_area` fails today, where the probe loads
`0x0,325x0,325x320,0x320,25x0,350x0,350x320,25x320`. This chain rebases onto that fix;
whether `dualExtruder.spec.ts` then needs the GUI reference is decided by its result.

## Included

- Project load and slice through the Always selection for every 3MF, with placement
  `none` and sidebar edits as override tokens applied after it, as today.
- Deleting the CLI-rule selection from the web.
- Not running the CLI main's compatible-printers check on the plater's selection.

## Deferred

- Showing the core's `preset_selection` result in the preset combos beyond the selected
  name (the dirty state): the web keeps the "Project presets" item as it is.
- `--uptodate`: a CLI option the plater does not have.
- Published-project overlays and embedded presets, which the core slice defers too.
- The plater's other load checks (`Plater.cpp:9570-9600`: the modified-G-code and
  customized-preset dialogs).

## Failure modes and how each is ruled out

1. **The browser keeps a value the plater refreshes.** A single-filament X1C project
   whose process is the user preset `0.20mm Standard @BBL X1C - Mine` (parent
   `0.20mm Standard @BBL X1C`, every difference entry empty) stores `wall_loops = 3`;
   the system value is 2. The probe loads the project config over the BBL presets and
   reports the edited process. The browser G-code must hold that value. The test first
   asserts the Native premise: the parent is selected and its edited `wall_loops` is not
   the project's 3.
2. **The browser rejects a project the plater slices.** `default-prisms.3mf` (Snapmaker
   A250, not installed) opens with project-local presets; the process lists the system
   printer name, not the project-local one. The browser must slice it, and its G-code must
   name the printer, process and filament presets the probe selects. The test first
   asserts the Native premise: the printer is the project-local
   `Snapmaker A250 (0.4 nozzle)(default-prisms.3mf)`.
3. **The CLI loses its check.** The CLI parity cases of `CLI_PROCESS_NOT_COMPATIBLE`
   (plain 3MF and `--load-settings`, `crates/ares-cli/tests/orca_parity`) run unchanged.
4. **The combos or slice blockers show the CLI's presets.** presets, projectPresets and
   sliceBlocker compare with the probe as above.
5. **Sidebar edits or placement change.** The plate specs' byte comparisons cover both.
6. **The browser G-code differs from the plater's elsewhere than in the selected presets.**
   The CLI-route specs compare every line outside the listed keys byte for byte, and the
   listed lines exactly with the probe's selection.

## Commit order and checks

1. `docs(web)`: this document.
2. `test(web)`: `web/e2e/presetSelectOracle.ts` and `web/e2e/projectPresetSelection.spec.ts`,
   failure modes 1 and 2. On the current web both fail: the browser keeps
   `wall_loops = 3` and the stored preset names.
3. `test(web)`: presets, projectPresets and sliceBlocker move to the probe (failure mode 4).
4. `test(web)`: the CLI-route specs expect the probe's selected-preset lines (failure mode 6).
   On the current web they fail on those lines: the browser writes the stored names.
5. `fix(core)`: the compatible-printers check runs on the CLI paths only (failure mode 2;
   failure mode 3 holds).
6. `feat(web)`: the switch.

Checks: fmt, clippy, the wasm32 check, typecheck and lint; the CLI parity cases of
failure mode 3; the desktop and packages E2E projects in official and seeded modes,
compared test by test with the run before the switch. Artifacts are the probe requests
and results and the browser G-code under `target/web-e2e/project-preset-selection/`.
