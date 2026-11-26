# CLI `--skip-modified-gcodes`

## Upstream boundary

Port the CLI policy for two existing OrcaSlicer data structures, at v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`): the saved preset override
columns in `libslic3r::DynamicPrintConfig` and `Model::plates_custom_gcodes`
(`libslic3r/Model.hpp:1743–1749`, with item types in `libslic3r/CustomGCode.hpp`).
The caller is `CLI::run` in `OrcaSlicer.cpp`. Neither the G-code template
renderer nor `ToolOrdering::assign_custom_gcodes` owns this policy.

`CLIMiscConfigDef` adds `skip_modified_gcodes` as `coBool`, default false
(`libslic3r/PrintConfig.cpp:13218–13221`); `OrcaSlicer.cpp:1605–1607` reads it
from the CLI config. `ConfigOptionDef::cli_args` maps underscores to dashes
(`libslic3r/Config.cpp:265–282`). `DynamicConfig::read_cli` accepts the bare
`--skip-modified-gcodes` or an attached value, e.g. `=1` or `=false`; it does
not consume a following value token or provide a `--no-skip-modified-gcodes`
spelling. The common parser also accepts one leading dash, treats an empty
attached value like the bare flag, normalizes the usual case-insensitive
boolean spellings (`1/true/yes/on/enabled`, `0/false/no/off/disabled`), and
then reads the first comma-separated item as a scalar bool
(`Config.cpp:1755–1783,1810–1854,1900–1906`; `Config.hpp:1929–1949`). Keep it
as CLI control state, not a print option or a template placeholder.

## Preset G-code during refresh

"Modified" means that the key belongs to the **keep set** for the preset
being refreshed. Normally that set is decoded from the corresponding saved
`different_settings_to_system` column: process first, then one entry per
filament, then machine. The skip branch does not compare text, inspect project
G-code for edits, or call `cli_different_settings`. For a retained filament
using `keep_project` (ordinary refresh with no declared changes in any column),
the set instead contains the saved config keys other than
`filament_extruder_variant` (`OrcaSlicer.cpp:2775–2815,3575–3615`). A script
that differs but isn't in the keep set is not an override for this operation;
a listed script can be removed from the keep set even when its value matches.

The source preset name comes from the corresponding nonempty `inherits_group`
entry if present, otherwise from the saved preset name (`:1851–1896`). Native
then uses the source selected by the existing refresh path: a matching
`--uptodate-settings`/`--uptodate-filaments` file, a flattened resource file,
or (for ordinary retained filaments) the resolved system preset. A flag by
itself does not resolve new sources or enable machine/process refresh.

The exact allowlist is `gcodes_key_set` (`:668–669`):

- `machine_start_gcode`, `machine_end_gcode`, `machine_pause_gcode`,
  `change_filament_gcode`, `before_layer_change_gcode`, `layer_change_gcode`,
  `template_custom_gcode`, `printing_by_object_gcode`, `time_lapse_gcode`,
  `wrapping_detection_gcode`;
- `filament_start_gcode`, `filament_end_gcode`.

Only keys visited by the source merge are eligible. When a source key is in
the allowlist and the keep set, remove it from the keep set and let the
ordinary assignment copy the **source value**, including its slot/variant
mapping. It does not inherently clear the G-code; an empty result is possible
when the source or `load_default_gcodes_to_config` supplies an empty value
(`:671–727`). Non-G-code overrides stay intact. When these branches update
the saved override column, preserve their existing serialization and remove
only the affected entries; the synthesized `keep_project` set is not written
back as an override column (`:3244–3253,3394–3403,3804–3811`).

| Native call site | Flow and effect |
| --- | --- |
| `:3212–3256` and `:3345–3406` | Machine and process pass the flag to `update_full_config` only with `--uptodate` and **no named replacement of that kind**. The lambda removes protected G-code and sets the source option (`:3098–3158`). A named `--load-settings` replacement takes `update_all=true` without the flag, even when the other kind is retained. Process uses the same allowlist but can only affect keys its source actually supplies. |
| `:3510–3811` (skip branch `:3670–3681`) | Retained filament merge: either the system/file refresh requested by `--uptodate` (`:2701–2767`) or the ordinary resolver-backed retained slots (`:2790–2815`). A slot explicitly replaced by `--load-filaments` bypasses the keep-set test. |
| `:3164–3200` | Separate saved-project machine/process fallback when no corresponding new config was loaded; `load_project_preset` does **not** consult the flag. Do not unprotect G-code in this general catalog merge merely because the flag is set. |

## Plate events

After any external event file is installed, `OrcaSlicer.cpp:4104–4135` erases
items whose type is exactly `CustomGCode::Custom` from every plate's list.
This is independent of preset refresh. There is no comparison to a system
preset or parent; the payload is irrelevant. Preserve the plate keys, event
mode, and surviving event order. `Template`, `PausePrint`, `ToolChange`, and
`ColorChange` are not removed. This must happen on model state before the
existing `Print::apply` and `GCode` paths, not by deleting emitted lines.
Do not reproduce the unchecked access to the first model when there is none.

## Rust destinations and scope

- On the existing saved-project route, carry the CLI bit from
  `ares-cli/src/args.rs::SliceArgs` in the execution-local `SlicerContext`
  through the CLI slice entry points. Apply the event removal to `Project`
  in `project/plate_custom_gcodes.rs` at the loaded-project seam in
  `project_slice` for both direct and profile-overlay routes, before
  print-state preparation. The existing archive parser and event emission
  stay unchanged.
- Retained machine and process refresh (`OrcaSlicer.cpp:3212–3256`,
  `:3345–3406`) runs through `profiles/project_refresh/native.rs` and
  `profiles/project_refresh/preset.rs::refresh` when `--uptodate` is active,
  no named replacement of that kind was supplied via `--load-settings`, and a
  Native refresh file is present for that kind. `NativeRefreshFile::from_json`
  admits every `PrintConfigDef` option (`ConfigBase::load_from_json`,
  `Config.cpp:941–1065`) so a process refresh file can supply any key in
  `gcodes_key_set`. Machine refresh materializes absent printer scripts via
  `load_default_gcodes_to_config(TYPE_PRINTER)` (`:678–709`); process refresh
  invokes `load_default_gcodes_to_config(TYPE_PRINT)` which adds no synthetic
  scripts, so only keys supplied by the process source are affected. Rewriting
  `different_settings_to_system` updates columns cumulatively across retained
  machine, process, and filament slots, and `project/domain/profile_input/overlay.rs`
  plus `overlay/cli_bookkeeping.rs` preserve retained columns while clearing
  only explicitly replaced kinds/slots. Selected-plate CLI execution
  (`project_slice/cli_plates.rs`) forwards the same CLI bit into the shared
  retained refresh path.
- Retained filament refresh (`OrcaSlicer.cpp:2775–2815,3510–3811`, skip branch
  `:3670–3681`) runs through `profiles/project_refresh.rs`,
  `profiles/project_refresh/preset.rs::refresh`,
  `profiles/project_refresh/filament.rs::apply`, and
  `profiles/project_refresh/filament/slot.rs::apply` for both file-driven
  (`--uptodate-filaments` / `filament_full`) and ordinary catalog-backed
  retained slots (forwarded via `project_slice/source.rs` and `state.rs` on
  CLI routes). Each nonempty filament source materializes absent or empty
  `filament_start_gcode` / `filament_end_gcode` via
  `load_default_gcodes_to_config(TYPE_FILAMENT)` (`:710–729,3590`: absent
  takes the `PrintConfigDef` default `" "`, empty array `[]` resizes to `""`).
  In the declared-edit branch (`refresh_unlisted`), released keys are removed
  only from that slot's `different_settings_to_system` column (`:3804–3811`).
  In the no-declared-changes `keep_project` branch (`!refresh_unlisted`),
  `filament_start_gcode` / `filament_end_gcode` are released to the system
  preset value without writing `keep_project` back as an override column.
  Explicit `--load-filaments` replacements remain unaffected. Do not extend
  the policy to GUI/WASM profile selection (`ProjectPresetPolicy::Always`) or
  non-CLI callers.

### Deferred

- Defer external event-file admission to its existing port, Native warning
  log text, unsliced project export, and adjacent preset selection rules.
  Do not add filtering of explicitly supplied presets, CLI option
  overrides, or generated G-code text. Keep non-CLI callers unchanged.

## Verification

Exercise the public CLI on synthetic saved 3MFs. Use the current wrapper
mapping: Native `--slice 0 --arrange 0 --outputdir native input.3mf` and
Ares `slice -o ares.gcode input.3mf` for a one-plate fixture, with the same
applicable flags and equivalent preset supply. Require both slicers to
succeed and compare the whole G-code (including the config block) using
only the existing `load_settings_cli::banner_normalized` comparator. Use
runtime Native output and retain argv, fixture digest, and both raw outputs.
The cases must execute against Native in official and seeded modes.

1. **Plate events without preset refresh.** Reuse the single-plate control
   and XML construction from `runner/plate_custom_gcode_emission.rs`. Add
   a `Custom` item with a harmless unique payload and a `PausePrint` or
   `Template` control. No flag emits both, the bare flag removes only
   the `Custom` item, and `=false` is identical to omission. Also check
   an unchanged project without a `Custom` event with and without the flag.

2. **Saved machine script with retained refresh.** Reuse the single-plate
   saved-project fixture and flattened file layout from the retained-refresh
   CLI cases. Put a short distinct harmless command in `machine_start_gcode`
   and list the key in the last override column alongside a non-G-code
   machine override. Give the matching refresh source a different nonempty
   command. Run `--uptodate --uptodate-settings <machine.json>` with and
   without the flag, plus `=false`, an unedited control, a no-`--uptodate`
   control, an explicit `--load-settings` replacement, a process-file-only
   fallback control, and empty vs absent source scripts.

3. **Saved process script with retained refresh.** Reuse the flattened
   refresh layout with a `gcodes_key_set` script (such as
   `layer_change_gcode`) listed in `different_settings_to_system[0]`
   alongside a non-G-code process override. Verify omitted-flag control,
   enabled release and column `[0]` rewrite when `process.json` supplies
   the script, no release when `process.json` omits the script, `=false`
   equivalence, catalog-only fallback without a process refresh file, and
   explicit `--load-settings` process replacement remaining unaffected.

4. **File-driven retained filament refresh.** Reuse the flattened refresh
   layout with `filament_start_gcode` and `filament_end_gcode` marked in the
   filament override column alongside a non-G-code filament override. Verify
   omitted-flag control, enabled release with source values (`[""]`, `[]`
   resized to `""`, absent defaulted to `" "`, and nonempty scripts),
   single-slot column rewrite on multi-filament inputs, selected-plate
   (`--slice 1`) routing, and explicit `--load-filaments` replacement
   remaining unaffected.

5. **Ordinary catalog-backed retained filament refresh.** Use fixtures whose
   named filaments resolve on both sides (matched Native system presets and
   Ares catalog) without `--uptodate` or `--load-filaments`. Verify passing
   whole-G-code omitted-flag controls first, then `--skip-modified-gcodes`
   for both declared-edit dirty columns (removing only the released script
   key from that slot's column) and no-declared-changes `keep_project`
   (updating the script without writing `keep_project` back to
   `different_settings_to_system`), plus multi-filament slot isolation and
   preservation of non-G-code overrides.

Use fail-first CLI tests for each slice, then run the affected parity cases
in both modes and the normal formatting, workspace Clippy, and WASM checks.
