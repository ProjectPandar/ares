# Web project preset fill

## Problem

A 3MF project opened in the web plater keeps only the keys its
`Metadata/project_settings.config` stores. Every key it lacks takes the
collection default (`apply_retained_preset_defaults`,
`crates/ares-core/src/project/load.rs:191`). OrcaSlicer takes those keys from
the project's system presets instead.

`web/e2e/dualExtruder.spec.ts:92` shows the gap. The H2D fixture
`tests/parity/master-extruder-id/stock-right-only.3mf` was saved by
BambuStudio 02.06 and lacks options that OrcaSlicer 2.5.0 has. Its web G-code
config block differs from the OrcaSlicer CLI's in 28 keys, for example:

| key | OrcaSlicer | web |
|---|---|---|
| `extruder_clearance_dist_to_rod` | 50 | 40 |
| `hotend_heating_rate` | 3.6,3.6 | 2,2 |
| `top_color_penetration_layers` | 5 | 0 |
| `support_cooling_filter` | 1 | 0 |

The Ares CLI already ports this fill as `ProjectPresetRefresh`
(`--project-preset-catalog`). With a catalog of OrcaFilamentLibrary and BBL,
it slices the edited fixture to G-code whose config block and body match the
OrcaSlicer CLI's byte for byte; only the five header lines differ. The web
never uses that path.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f`):

- `src/OrcaSlicer.cpp:3167-3200`, `load_project_preset`: the project's printer
  and process are loaded over the collection default with
  `Preset::load_external_config`. Their system preset (`inherits_group`, else
  the stored preset name) is the base. A key the project lacks takes the
  system value. A key it has is kept unless `refresh_unlisted` holds and the
  key is not in `different_settings_to_system`. Bookkeeping keys are skipped.
- `src/OrcaSlicer.cpp:2775-2790` and the filament loop after them: the
  same refresh for each filament slot, and `refresh_unlisted`, which is true
  only for `--uptodate` or a non-empty `different_settings_to_system` entry.
- `src/OrcaSlicer.cpp:3182-3185`: when the resolver cannot find the system
  preset, it logs a warning and returns `nullptr`. The project keeps its
  values, and the keys it lacks take the collection default.
- `src/libslic3r/Preset.cpp:1090-1119`, `Preset::load_external_config`, the
  shared fill over `default_config`.
- `src/libslic3r/PresetBundle.cpp:6278-6371`,
  `PresetBundle::load_config_file_config`: the GUI load of a project config
  (`Plater.cpp:9623`) through `PresetCollection::load_external_preset`
  (`Preset.cpp:3028`), which applies the same fill to each collection when
  the project opens.
- `src/slic3r/Utils/PresetUpdater.cpp`,
  `check_installed_vendor_profiles`: the GUI's system presets are those of the
  installed vendors, plus OrcaFilamentLibrary and the default bundle. A
  project whose vendor is not installed finds no base.

## Destination

- **Core** (`crates/ares-core/src/profiles/project_refresh/preset.rs`,
  `system_config`): a printer or process system preset missing from the
  catalog follows `OrcaSlicer.cpp:3182-3185`. The project keeps its values,
  and `apply_retained_preset_defaults` fills the keys it lacks, as
  `ProjectPresetRefresh::load` already does after the refresh. Today this
  case fails with `system preset '<name>' was not supplied`. A filament slot
  already keeps the imported filament on a miss (`OrcaSlicer.cpp:2805-2808`).
- **Web** (`web/app/src/state/actions.ts`, `presets.ts`): a 3MF opens with
  `createPlaterSessionWithProfiles` and the selection
  `{"from_project": true}`, with placement `none`, over the installed-vendor
  catalog that `profileRequest` builds. The scene, its settings and the
  settings tabs then show the filled values. While the "Project presets"
  selection holds (`presetSelection` null), slicing goes through
  `plater.sliceWithProfiles` with the same selection instead of
  `plater.slice`. Sidebar edits stay override tokens applied after the fill,
  as they are today.
- No new wasm or worker API: `platerCreateWithProfiles`,
  `ProfileSliceRequest` and `slice_plater_with_profiles` already accept the
  `from_project` selection.

## Included

- Printer, process and filament fill from the installed system presets on
  project load and on slice, with the `refresh_unlisted` rule unchanged
  (`uptodate` false).
- The resolver-miss rule for the printer and process.

## Deferred

- The GUI's own replacement rule: `load_external_preset` refreshes every key
  not listed in `different_settings_to_system`, even when every entry is
  empty, and then matches the result against loaded presets
  (`profile_print_params_same`). The CLI keeps present keys in that case.
  The oracle and the existing core port follow the CLI rule.
- Selecting a matching system or user preset after load (the
  `LoadAndSelect::Always` branch); the "Project presets" item stays as
  `9b3158ccf` left it.
- `--uptodate` in the web.
- Loading a vendor that is not installed in order to resolve a project's
  presets: the GUI does not do this either.

## Failure modes and how each is ruled out

1. **A project whose vendor is not installed can no longer load or slice.**
   Before the core fix, any miss is an error. A new CLI parity case renames
   the fixture's `print_settings_id` to an unknown name. OrcaSlicer slices
   it ("Success."), keeping the project's process values with defaults for
   the keys it lacks (`top_color_penetration_layers` 0), and printer and
   filament values from their system presets (`extruder_clearance_dist_to_rod`
   50, `support_cooling_filter` 1). Ares must produce the same G-code. The
   existing web specs import Bambu projects without installing BBL and must
   keep passing.
2. **Filled keys reach the slice but not the scene, or the reverse.** Both use
   the same selection. dualExtruder checks the slice (`:92`), and a new
   assertion checks that a printer settings field of a key the fixture lacks
   (`extruder_clearance_dist_to_rod`) shows 50 after load.
3. **Sidebar edits are lost or overridden by the fill.** dualExtruder's
   `filament_map`, `filament_map_mode` and `nozzle_volume_type` edits are in
   the compared G-code.
4. **Placement moves project objects.** The selection's placement is `none`,
   and dualExtruder compares the G-code moves byte for byte.
5. **Imported user presets are used as a base.** `system_config` keeps
   rejecting user-scope positions; the "Project presets" path never resolves
   a user preset.

## Commits and acceptance

1. `docs(web)`: this document.
2. `test(core)`: the unknown-process CLI parity case in
   `crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/project_preset_refresh.rs`,
   compared with the OrcaSlicer CLI's G-code. It fails first with
   `system preset '…' was not supplied`.
3. `fix(core)`: the resolver-miss rule in `system_config`.
4. `test(web)`: the dualExtruder settings-field assertion. It fails first, like
   `:92`.
5. `feat(web)`: the session load and slice through the `from_project`
   selection.

The checks are fmt, clippy `--workspace --all-targets -D warnings`, the wasm32
check and clippy, typecheck, and lint. The web E2E runs dualExtruder,
presetSlice, slice, settings, printerWizard, startupSelection, presetImport
and the project-import specs with the 2.5.0 oracle. The artifacts are
`target/web-e2e/dual-extruder/{web,native}.gcode` and the CLI parity case's
G-code pair.
