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
| | 5790-5794 | erases `compatible_prints`, `compatible_prints_condition`, `compatible_printers`, `compatible_printers_condition` and `inherits` |
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

So the explicit-selection branch serves only the wasm adapter and the Rust tests of the
public `slice_with_profiles*` entries.

## Change

### Core

In entry.rs, the explicit-selection branch stops calling `Catalog::compose`. It composes
through a new function in `profiles/project_selection/`, which:

1. Loads the process, printer and filament collections from the catalog with
   `Collection::load` (`project_selection/collection.rs:64-134`). Its records already carry
   the inherited `filament_id` (`load_config`, collection.rs:341-379), so 7793-7817 needs no
   new code.
2. Builds a `PresetCollectionState` for each selected name. This mirrors
   `PresetCollection::select_preset`: the selected record is also the edited copy, unmodified.
3. Sets `nozzle_volume_type` in the project config by `load_selections` (4196-4216) when the
   request carries none. The wasm `nozzle_volume_type` override of a dual-nozzle printer stays
   as it is.
4. Calls `full_fff_config::full_fff_config` on the opened project's config. For an STL there is
   no opened config, so it uses the project option defaults.
5. Returns the composed profile in the shape `ProfileInput` takes today (`profile`, `raw`,
   `filament_raws`).

The CLI machine-switch steps that `ProfileInput` runs stay unchanged in this slice:
`shrink_to_new_bed` (OrcaSlicer.cpp:3766-3796), the one-colour prime-tower rule (4323-4342),
and the `nozzle_volume_type` fill when absent (6981-6985). The GUI does not shrink on a printer
switch; that difference is its own slice.

Step 3 has to put the types in the project config, not in the overrides. Ares treats an
override as the CLI's `--nozzle-volume-type`: a new vector that differs from the 3MF's triggers
the flush rebuild (`options/flush_matrix.rs:62-71`). A project value of one Standard per head
with no override does not. That matches the measured upstream GUI-route result below.

Compatibility checks, `curr_bed_type` and the colour sync are unchanged. Every other web
request (from-project, plater sessions, STL batches) keeps its current branch.

`catalog/compose.rs`, `load_settings.rs` and `cli_metadata.rs` are not edited.

### Affected entries

All of these reach entry.rs:286-288 with an explicit selection:

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

- Every `ares-cli` parity and `--load-settings` case. Their bytes go through
  `load_settings.rs` and never reach the changed branch.
- The CLI 3MF project-preset refresh (`project_presets.rs`, `ProjectPresetPolicy::Cli`).
- The `ares-cli` `orca_parity` and `ares-core` profile tests that call `slice_with_load_settings`
  or `--load-settings`.

The implementation lists the Rust tests that call the changed entries with an explicit
selection. Each one either expects the GUI composition with an upstream citation, or switches
to the CLI entry if it was checking CLI behaviour.

## End-to-end references

The browser's own slice is unchanged; only references move from the CLI composition to the
GUI one.

- **`printerSwitchFilaments.spec.ts`, U1 slice.** The reference becomes the defined export B
  from `web-e2e-upstream-references.md` section 2: `cliSwitchedExport` with
  `nozzle_volume_type` set to one Standard per head (the session's own value).
  `compatible_printers_condition` is erased rather than taken from the process.
  `filament_ids` keeps the session's inherited ids (`OFDSrzZ8` for the four Generic PLA @U1
  slots). The measured Native slice of B already matches the browser's flush matrix line
  (84/280). Its remaining differences are `filament_ids` and `nozzle_volume_type`, which this
  route change removes.
- **`presetSlice.spec.ts`, 3MF legs.** `withSelectedPresets` already erases
  `compatible_printers_condition` and takes the session's ids. That matches the new browser
  output, so only the comment changes, to cite this route.
- **`presetSlice.spec.ts`, STL leg.** `withLoadedMultiColour` over the `--load-settings`
  reference changes to the GUI composition. The `compatible_printers_condition` line is removed,
  and `filament_ids` comes from the selected presets' inherited ids, taken from a preset-select
  probe session for the same selection.
- **STL slices with the first-run selection, and `profileSlice`, `multiExtruderVariant` and
  `bedDefault`.** Each reference that comes from the CLI composition gets the same two line
  changes from a probe session, or moves to a plater export where one exists. The complete list is
  whatever the implementation's paired suite on main shows as changed. Every spec that changes
  is named in the hand-off report, together with its reference change.

## Included and deferred

Included:

- full_fff_config composition for explicit selections (5536-5542, 5636, 5679, 5790-5794, 5821);
- the inherited filament ids (7793-7817) through the existing collection load;
- the `load_selections` volume types (4196-4216);
- the U1 reference B, and the U1 flush matrix line that section 2 of the references document
  moved here.

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
and the U1 slice passing.
