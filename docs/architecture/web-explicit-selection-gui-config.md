# Web explicit preset selection: the GUI's full_fff_config

## Decision

When the web slices with presets the user selected, the config is composed as the GUI composes
it: `PresetBundle::full_fff_config`. The CLI's `--load-settings` composition stays as it is.

This is a source-cited rewrite slice of `OrcaSlicer/src/libslic3r/PresetBundle.cpp`, at
OrcaSlicer 40ecc66f7 (v2.5.0). It moves one existing Ares entry from the CLI composition onto
the already ported `profiles/full_fff_config.rs`.

## Upstream boundary

| upstream | lines | behaviour |
| --- | --- | --- |
| `PresetBundle::full_fff_config` | PresetBundle.cpp:5536-5542 | starts from `FullPrintConfig::defaults()`, applies the edited print, the default filament and the edited printer, then `project_config` |
| | 5636, 5679 | each filament's `filament_id` is its preset's `filament_id` |
| | 5791-5795, 5797 | erases `compatible_prints`, `compatible_prints_condition`, `compatible_printers`, `compatible_printers_condition` and `inherits`, then `different_settings_to_system` |
| | 5571-5595, 5628-5647, 5673-5712, 5778-5788 | collects the group entries from the edited print, each filament and the edited printer |
| | 5823-5841 | stores `compatible_machine_expression_group`, `compatible_process_expression_group`, `inherits_group`, `different_settings_to_system` and `print_compatible_printers` only when some entry is non-empty |
| | 5821 | writes the collected `filament_ids` |
| `PresetBundle::resolve_vendor_preset` | 7793-7817 | a system preset without its own `filament_id` inherits its parent's, across bundles |
| `PresetBundle::load_selections` | 4196-4216 | sets `project_config.nozzle_volume_type` from the stored types of the base printer, or the printer's `default_nozzle_volume_type`, padded to its length |

The CLI composition that stays unchanged:

- OrcaSlicer.cpp:2251-2254 and 3616-3621: `filament_ids` from each loaded file's own key.
- OrcaSlicer.cpp:3097-3103 and 3403-3408: every key of the loaded process, including
  `compatible_printers_condition`.

## Ares today

`with_edited_profile_source` (`crates/ares-core/src/profiles/entry.rs:246-312`) has two
branches:

- The from-project selection (`{from_project: true, ...}`) goes through
  `project_refresh::selection` (entry.rs:266) to `project_selection::refresh`, which already
  calls `full_fff_config::full_fff_config` (`project_selection.rs:143`).
- An explicit selection (`{printer, process, filaments}`) goes to
  `catalog.catalog.compose(&selection)` (entry.rs:286-288). That is the CLI composition
  (`profiles/catalog/compose.rs`), so `filament_ids` come from each leaf file's own key
  (compose.rs:115-127) and `compatible_printers_condition` stays.

The CLI does not reach that branch:

- `--load-settings` calls `slice_with_load_settings`, which composes through
  `profiles/load_settings.rs:154`.
- The CLI's 3MF preset refresh sends only from-project selections
  (`crates/ares-cli/src/project_presets.rs:111-123`).

So the CLI binary never reaches the explicit-selection branch. Two kinds of caller do:

- the wasm adapter;
- the Rust tests of the public `slice_with_profiles*` entries. 43 `ares-cli` parity test files
  (about 63 tests) pass an explicit `{machine, process, filaments}` selection and compare the
  bytes with Native `--load-settings` (`tests/orca_parity/runner/native_user_presets.rs:112-147`).
  For them, this branch is the CLI composition.

## Change

### Composition is chosen by the request type

Upstream has two compositions of the same selected presets: the GUI's
`PresetBundle::full_fff_config` and the CLI's `--load-settings` (OrcaSlicer.cpp:2251-2254,
3097-3103, 3403-3408, 3616-3621). Both stay, and the caller names which one it wants:

- `pub enum Composition { Gui, Cli }` in `profiles/`. The explicit branch of
  `with_edited_profile_source` matches it exhaustively: `Cli` keeps `Catalog::compose`, and `Gui`
  takes the function below.
- `ProfileSliceRequest` stays the CLI request. `CliProfileSliceRequest` wraps it and is CLI too.
- A new `GuiProfileSliceRequest<'a>(pub ProfileSliceRequest<'a>)` is the plater's request.
- The public entries take either type through one sealed trait. The trait yields the inner
  request and its `Composition`, with static dispatch. The entries are `slice_with_profiles*`,
  `validate_with_profiles`, `profile_settings`, `profile_placed_project`,
  `slice_plater_with_profiles`, `PlaterSession::profile_settings` and
  `load_stl_files_with_profiles`.
- The wasm adapter builds `GuiProfileSliceRequest` at its 8 request sites: `src/profiles.rs`
  135, 183, 231 and 262; `src/plater.rs` 101 and 316; `src/plater/stl_files.rs` 65 and 104.
  `ares-cli` and every Rust test keep `ProfileSliceRequest`.

The alternative was a required `composition` field on `ProfileSliceRequest`, with no default.
The compiler would then list every caller. But there are 80 request literals in 66 files, about
68 of them in tests, including all 43 parity files, so that variant edits each of them for no
change in behaviour. The typed request edits the 8 wasm sites and the entry signatures, and no
test file. Every caller still states its composition through the type it builds: the CLI types
have always meant the CLI composition, and the GUI one is new and used only by wasm.

`Composition` decides only explicit selections. A from-project selection keeps its policy from
the selection JSON: `load_and_select: always` is the plater, otherwise the CLI
(`project_refresh.rs:129-130`).

### The GUI composition

For `Composition::Gui`, the explicit branch composes through a new function in
`profiles/project_selection/`, which:

1. Loads the process, printer and filament collections from the catalog with
   `Collection::load` (`project_selection/collection.rs:64-134`). Its records already carry
   the inherited `filament_id` (`load_config`, collection.rs:341-379), so 7793-7817 needs no
   new code.
2. Builds a `PresetCollectionState` for each selected name. This mirrors
   `PresetCollection::select_preset`: the selected record is also the edited copy, unmodified.
3. Sets `nozzle_volume_type` in the project config by `load_selections` (4196-4216) when the
   request carries none. The wasm `nozzle_volume_type` override of a dual-nozzle printer stays
   as it is.
4. Composes the selected presets' values with the catalog, which loads the same resolved presets
   as `--load-settings`, and collects from the records what full_fff_config decides beyond those
   values: the group keys, and the preset options no preset file sets. A loaded preset holds every
   option of its kind (5536-5542, 5721-5774), and the G-code prints each held key but the banned
   and nil ones (GCode.cpp:8305-8370). These apply over the merged config, so a key the opened
   3MF holds keeps its value. For an STL there is no opened config, so they apply over the
   composed one.
5. Returns the composed profile in the shape `ProfileInput` takes today (`profile`, `raw`,
   `filament_raws`).

The CLI machine-switch steps that `ProfileInput` runs stay unchanged in this slice:
`shrink_to_new_bed` (OrcaSlicer.cpp:3766-3796), the one-colour prime-tower rule (4323-4342),
and the `nozzle_volume_type` fill when absent (6981-6985). The GUI does not shrink on a printer
switch; that difference is its own slice.

Step 3 has to put the types in the project config, not in the overrides. Ares treats an
override as the CLI's `--nozzle-volume-type`: a new vector that differs from the 3MF's triggers
the flush rebuild (`options/flush_matrix.rs:62-71`).

The flush rebuild is part of the CLI's preparation before `Print::apply` (OrcaSlicer.cpp:3851-4023).
The plater applies full_fff_config as it holds it, so the explicit GUI route resolves the loaded
config the way an opened project's slice does, with no context from the 3MF it opened. The web
sends the session's flush data and `filament_colour` as overrides, and a CLI colour override
rebuilds the matrix from the colours. On the U1 slice that rebuild gives 61/203 where the session
holds 84/280.

Compatibility checks, `curr_bed_type` and the colour sync are unchanged. Every other web
request (from-project, plater sessions, STL batches) keeps its current branch.

`catalog/compose.rs`, `load_settings.rs` and `cli_metadata.rs` are not edited.

### Affected entries

Built from a `GuiProfileSliceRequest`, all of these reach entry.rs:286-288 with an explicit
selection and `Composition::Gui`:

| crate | entry |
| --- | --- |
| ares-core | `slice_with_profiles`, `slice_with_profiles_processed`, `slice_with_profiles_validated`, `slice_with_profiles_processed_validated` (`profiles/entry.rs`) |
| ares-core | `validate_with_profiles` (entry.rs:180) and `profile_settings` (`profiles/settings.rs:25`) |
| ares-core | `slice_plater_with_profiles` (`plater/slice.rs:55`), `PlaterSession::profile_settings` (`plater/session/input.rs:27`), the STL batch loader (`plater/stl_files.rs:34`) |
| ares-wasm | `sliceWithProfiles`, `sliceWithProfilesBytes`, `sliceWithProfilesProcessed`, `sliceWithProfilesProcessedBytes`, `profileSettings` (`src/profiles.rs`) |
| ares-wasm | `PlaterSession.sliceWithProfilesProcessed` (`src/plater.rs:304`), the STL batch `profileSettings` (`src/plater/stl_files.rs:55`) |

In the web app, `sliceWithPresets` and `presetValues` (`web/app/src/state/presetSlice.ts`)
use these entries whenever a preset selection exists. The first-run config selects
"MyKlipper 0.4 nozzle" (`web/e2e/failureDiagnostics.ts:11-24`), so most STL slices in the e2e
suite take this branch too.

### CLI cases that must not change

- Every `ares-cli` parity and `--load-settings` case:
  - the binary composes through `load_settings.rs`;
  - the 43 parity test files that call `slice_with_profiles*` keep `ProfileSliceRequest`, so
    they keep `Composition::Cli`.
- The CLI 3MF project-preset refresh (`project_presets.rs`, `ProjectPresetPolicy::Cli`).
- The `ares-cli` `orca_parity` and `ares-core` profile tests that call `slice_with_load_settings`
  or `--load-settings`.

No Rust test changes its request type. The `Always` suite (`tests/project_preset_selection/`)
sees the group-key change of the from-project branch below. Each assertion that changes there
cites the upstream line it now follows.

## End-to-end references

The references move from the CLI composition to the GUI one.

- **`printerSwitchFilaments.spec.ts`, U1 slice.** The reference becomes the defined export B
  from `web-e2e-upstream-references.md` section 2: `cliSwitchedExport` with
  `nozzle_volume_type` set to one Standard per head (the session's own value).
  `compatible_printers_condition` is erased rather than taken from the process.
  `filament_ids` keeps the session's inherited ids (`OFDSrzZ8` for the four Generic PLA @U1
  slots). The measured Native slice of B prints the session's flush matrix (84/280). Its remaining
  differences are `filament_ids` and `nozzle_volume_type`, which this route change removes,
  and the flush line, which the route keeps without the CLI preparation above.
- **`presetSlice.spec.ts`, 3MF legs.** `withSelectedPresets` already erases
  `compatible_printers_condition` and takes the session's ids. `withoutCliGroups` then removes
  the all-empty groups the session lacks. `different_settings_to_system` stays: the deferred
  one-colour prime-tower rule records `enable_prime_tower`, and the process edit dirties the
  process column as the plater's edited print does (5586-5595).
- **`presetSlice.spec.ts`, STL leg.** The `--load-settings` reference changes to the GUI
  composition. The `compatible_printers_condition` line is removed, and `withoutCliGroups`
  removes the groups the plater session lacks. `filament_plugin_config_overrides` comes from the
  session's first filament (5726-5735), as `withSelectedPresets` sets it for the 3MF legs. Its
  `filament_ids` already equals the session's.
- **STL slices with the first-run selection, and `profileSlice`, `multiExtruderVariant` and
  `bedDefault`.** Each reference that comes from the CLI composition gets the same two line
  changes from a probe session, or moves to a plater export where one exists. The complete list is
  whatever the implementation's paired suite on main shows as changed. Every spec that changes
  is named in the hand-off report, together with its reference change.

### Group keys

The H2D sidebar slice (`dualExtruder.spec.ts`, after the flow options) prints
`; different_settings_to_system = ;;;` and `; inherits_group = ;;;`. The plater export it is
compared with has neither line. Both lines come from two gaps on the GUI route, and both are part
of this slice.

1. `profiles/full_fff_config.rs` ports the erase list and the filament composition, but not the
   group composition above (5571-5595, 5628-5647, 5673-5712, 5778-5788, 5797, 5823-5841). The
   from-project route starts from the 3MF's own config (`project_selection.rs:142`), so the 3MF's
   all-empty groups stay. After this slice, `full_fff_config.rs` removes the five group keys,
   collects them from the selected presets as upstream does, and stores each one only when some
   entry is non-empty.
2. The G-code config export creates `different_settings_to_system` and `inherits_group` at
   filament count + 2 whenever they are absent
   (`options/config_export/full_config.rs:213-227`). That is the CLI's resize in `CLI::run`
   (OrcaSlicer.cpp:3202-3205), and the GUI slice never runs it: `append_full_config` prints only
   the keys the applied config holds (GCode.cpp:8272-8274). After this slice the resize runs only
   for configs loaded on the CLI route. Configs composed by `full_fff_config` keep what it stored.

Both GUI branches change: the from-project (`Always`) branch, which already calls
`full_fff_config.rs`, and the explicit branch this slice moves onto it. The CLI does not reach
`full_fff_config.rs`: its 3MF refresh takes the `Cli` policy through `preset::refresh`
(`profiles/project_refresh.rs:181-192`), and `--load-settings` composes through
`load_settings.rs`. The export change is the only edit on a path the CLI also runs, so the CLI
byte equality in Verification covers it.

The sidebar edits reach the GUI route as overrides, and applying them records the changed keys
the way the CLI records its command-line options (OrcaSlicer.cpp:4168-4220). That creates
`different_settings_to_system` at filament count + 2 even when no preset owns a changed key. The
plater holds its edits before full_fff_config collects the groups, so the store rule
(5823-5841) runs after the edits on both GUI branches.

The reference stays as it is: `withoutCliGroups` (`web/e2e/platerExport.ts:97-112`) already
removes the Native CLI's all-empty lines when the plater config lacks them.

## Included and deferred

Included:

- full_fff_config composition for explicit selections (5536-5542, 5636, 5679, 5790-5794, 5821);
- the inherited filament ids (7793-7817) through the existing collection load;
- the `load_selections` volume types (4196-4216);
- the U1 reference B, and the U1 flush matrix line that section 2 of the references document
  moved here;
- the group keys (5571-5595, 5628-5647, 5673-5712, 5778-5788, 5797, 5823-5841) on both GUI
  branches, and the CLI resize (OrcaSlicer.cpp:3202-3205) only on the CLI route.

Deferred:

- `shrink_to_new_bed` versus the GUI's unshrunk switch;
- the one-colour prime-tower rule on the explicit route;
- per-preset user edits beyond the selected record (the web sends edits as overrides);
- the H2D 0.2 flow-option app gap.

## Verification

In order:

1. The Rust tests that call the changed entries.
2. A complete `ares-cli` parity run, which must stay unchanged.
3. The specs above.
4. The whole web suite, paired with main.

The pass condition is no spec that passes on main failing, every changed reference explained,
the U1 slice passing, and the H2D sidebar slice of `dualExtruder.spec.ts` equal to its plater
export.
