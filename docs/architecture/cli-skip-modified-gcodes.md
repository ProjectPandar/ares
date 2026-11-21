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
  stay unchanged. This part does not require Native preset file routing.
- The automatic retained-filament merge is already represented by
  `profiles/project_refresh/preset.rs::refresh` and
  `profiles/project_refresh/filament/slot.rs::apply`. Apply the same
  keep-set exception there on the CLI path when the existing catalog
  resolves a retained slot. Keep the current selection and variant logic;
  do not create new catalog supply or extend the option to GUI/WASM profile
  selection. Include the ordinary resolver-backed filament case only if
  both slicers can use matching sources through their existing paths.

### Deferred

- The source-file retain branches need the Native retained-preset refresh
  path (`ares-cli/src/native_refresh.rs`, core
  `profiles/project_refresh/native.rs` and the retained merge). The existing
  catalog-only `--uptodate` adapter cannot stand in for
  `--uptodate-settings` or `--uptodate-filaments`: the merge must know which
  kinds/slots came from an actual refresh file. Add the machine/process and
  file-driven filament cases once that path is available. The saved-project
  CLI requests are the temporary adapters for this same upstream boundary.
  Connecting the policy to the separate `slice_plates` selection/output
  route depends on that route sharing retained refresh; do not make a
  second refresh implementation just for plate selection.
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
2. **Ordinary retained filament.** Use a fixture whose named filament is
   resolvable on both sides (matched Native system preset and Ares catalog).
   Give its saved slot a distinct `filament_start_gcode`, without passing
   `--load-filaments`. With no refresh flags, show that
   omission keeps the saved script and the skip flag uses the system script.
   Exercise both a recorded edit and the no-declared-changes `keep_project`
   case. Check a non-G-code override is retained and that other slots are
   not replaced by this slot's source.
### Deferred file-driven verification

**Saved machine script with retained refresh.** Reuse the single-plate
saved-project fixture and flattened file layout from the retained-refresh
CLI cases. Give the project a user machine name inheriting a differently
named system machine. Put a short distinct harmless command in its
`machine_start_gcode` and list the key in the last override column. Give
the matching refresh source a different nonempty command; retain a
second non-G-code machine override in that column. Run
`--uptodate --uptodate-settings <machine.json>` with and without the flag;
also run the modified project with the flag but without `--uptodate`.
Assert first on Native's pre-config command and override column: the
command changes to the source only in the enabled retained-refresh
case and only the script key leaves the column. An unchanged control
stores the same start script as the source, without marking it; run it
with and without the flag and require identical output for the pair.
Compare Ares with Native separately in all cases. Reuse the file-driven
fixture for a retained filament with a nonempty source script and an
explicit replacement control to ensure the flag only affects retained
slots. Keep these cases separate from the plate event witness.

Use fail-first CLI tests for each slice, then run the affected parity cases
in both modes and the normal formatting, workspace Clippy, and WASM checks.
