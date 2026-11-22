# CLI route: two `is_bbl_3mf` conditions for non-BBL 3MFs

Native's `CLI::run` treats a 3MF written by another slicer differently from a BBL-family 3MF in two
places that Ares does not yet follow. This slice adds the `is_bbl_3mf` condition to both. Both
concern the CLI route only.

Upstream is OrcaSlicer v2.5.0 (40ecc66f7).

## Upstream boundary

`is_bbl_3mf` is `m_is_bbl_3mf` from the 3MF importer: the `Application` metadata starts with
`BambuStudio-` or `OrcaSlicer-`, or the model carries the `OrcaSlicer` version metadata
(`src/libslic3r/Format/bbs_3mf.cpp:4170-4186`). STL input never sets it.

### 1. `upward_compatible_machine` exists only for BBL 3MFs

`src/OrcaSlicer.cpp`:

- 1759-1923: `if (is_bbl_3mf)` block of the 3MF load. The `else` at 1924 covers every other input.
- 1894: `config.option<ConfigOptionStrings>("upward_compatible_machine", true)` reads the key with
  `create = true`. On a project config without the key this adds it, empty.
- 1994-1996: `config += m_print_config; m_print_config = config` for every 3MF. Only a BBL 3MF
  therefore brings the key into the CLI config.
- Loaded presets do not add it back:
  - `update_full_config` (3099-3103) copies only the keys the loaded preset file holds;
  - `resolve_preset` (2118-2123) returns early for a preset without `inherits`.
- `GCode::append_full_config` (`src/libslic3r/GCode.cpp:8272-8274`) prints every key of
  `full_print_config()`. The `--export-settings` document is written from the same config.

So a non-BBL 3MF loaded with flattened presets prints no `; upward_compatible_machine = ` line and
exports no `upward_compatible_machine` setting. A BBL 3MF prints the empty line.

### 2. The one-colour prime-tower rule needs a BBL 3MF

- 2531-2535: `disable_wipe_tower_after_mapping` turns on only when
  `is_bbl_3mf && load_filament_count > 0 && load_filaments_set.size() == 1`.
- 4321-4343 then sets `enable_prime_tower = false` and appends `enable_prime_tower` to
  `different_settings[0]`.
- On a non-BBL 3MF the rule never runs: with two used filaments of one loaded preset the tower stays
  (`Print::has_wipe_tower`, `src/libslic3r/Print.cpp:5189-5208`).

## Current Ares behaviour

`Project::documents.bbl_family_3mf` already mirrors `m_is_bbl_3mf` (`project/load.rs:207`), and
STL, assembly and multipart projects set it to `false`.

1. `Project::initialize_cli_loaded_project_config`
   (`crates/ares-core/src/project/domain/profile_input.rs:80-92`) inserts an empty
   `upward_compatible_machine` into any project raw that lacks it. Its comment cites
   `OrcaSlicer.cpp:1769`, the pre-v2.5.0 line; the v2.5.0 line is 1894. The registry entry has
   `full_config_default = false`
   (`options/registry/definitions/table/tail_terminal_suffix.rs:66`), so on the `--load-settings`
   route this insertion is the only reason the line is printed. Routes without preset files get the
   key earlier from the retained preset fill (see Deferred).
2. `mapped_prime_tower::disable_after_mapping`
   (`crates/ares-core/src/project/domain/profile_input/mapped_prime_tower.rs`) fires whenever every
   `filament_settings_id` entry names one preset. It does not read `bbl_family_3mf`.

Measured with one argv on official and seeded Native (they agree everywhere) and Ares at d651ba345.
The fixture is two cubes on filaments 1 and 2, exported by Native from the Creality Ender-3 0.4
presets with the tower on; the non-BBL copy replaces `Application` with `PrusaSlicer-2.8.1` and
drops the `OrcaSlicer` metadata.

| 3MF | `--load-filaments` | Native | Ares |
|---|---|---|---|
| BBL | A;A | tower off, column 0 `enable_prime_tower`, `upward_compatible_machine` line | same bytes |
| BBL | A;B | tower on, `upward_compatible_machine` line | same bytes |
| non-BBL | A;B | tower on, no `upward_compatible_machine` line | one extra line: `; upward_compatible_machine = ` |
| non-BBL | A;A | tower on (38579 lines) | tower off (9156 lines), column 0 `enable_prime_tower`, extra line |

B is a PLA copy of A with its own name, `filament_id` and 220 °C (CR-PLA with CR-PETG exits 194 on
every engine for mixed temperatures).

`--export-settings` on the same pair: the BBL document has `"upward_compatible_machine": []` on
both; the non-BBL Native document has no such key, Ares writes `[]`.

## Change

Both conditions belong to `CLI::run` (`OrcaSlicer.cpp:1759-1923`, 2531). The browser routes follow
the plater (`PresetBundle::full_fff_config`), so only the CLI call sites take the condition; every
other route keeps its current behaviour.

1. `initialize_cli_loaded_project_config` takes whether to create `upward_compatible_machine`
   (`OrcaSlicer.cpp:1759, 1894`, replacing the stale `:1769` citation). The two CLI call sites
   pass `documents.bbl_family_3mf`; every other call site passes `true`, as today.
2. `three_mf.rs` calls `disable_after_mapping` only for a BBL-family 3MF (2531). The plater
   route (`full_fff_config` is `Some`) never calls it.

### Callers of `initialize_cli_loaded_project_config`

| Caller | Route | Creates the key | Coverage |
|---|---|---|---|
| `profile_input/three_mf.rs:181` | profile overlay of a 3MF; CLI when `full_fff_config` is `None` (`--load-settings` / `--load-filaments`), plater otherwise | CLI: `bbl_family_3mf`; plater: always | tests 1 and 2; the plater legs of the web suite |
| `project/settings_export.rs:24` | `--export-settings` (CLI only; no wasm entry) | `bbl_family_3mf` | not observable yet: the retained preset fill adds the key earlier on this route (see Deferred) |
| `project_slice/state.rs:123` | 3MF without a profile overlay: the CLI direct slice and the browser's direct 3MF loads share it | always (unchanged) | deferred, see below |
| `project/domain/preset_refresh.rs:46` | `--uptodate` refresh, plater selection, system-catalog route, STL model with overrides | always (unchanged) | web suite |
| `print/validate.rs:89` | `validate_project_plate` | always (unchanged) | nothing reads the key |

Profile-overlay slices never reach `state.rs:123`: `slice_source` sends a source with a profile to
`profile_entry::prepare_profile_slice`, which uses only the `three_mf.rs` initialization.

## Verification

New `orca_parity` module, whole-file G-code, `result.json` and stderr against Native with the same
argv. The 3MF fixtures are exported by Native at test time as in the table above.

1. Non-BBL 3MF, `--load-filaments A;B`: identical to Native. At d651ba345 this differs by exactly
   the `upward_compatible_machine` line. The BBL 3MF with the same argv is the control and is
   identical on both sides.
2. Non-BBL 3MF, `--load-filaments A;A`, two used filaments: identical to Native, with the tower
   printed. This needs both changes. The BBL 3MF with the same argv is the control.
Test 1 fails before change 1; test 2 fails before both changes.

## Deferred

- The other `create = true` reads of the BBL block (1895 `print_compatible_printers`, 1896
  `different_settings_to_system`, 1899 `printable_area`): the Native-exported fixtures already carry
  all three keys, so no output changes.
- Machine files with `inherits`: `resolve_preset` can complete defaults there. The unflattened
  Ender-3 machine file exits 251 on Native and Ares alike, so this route needs another fixture.
- The CLI direct 3MF slice without preset files (`state.rs:123`). Native omits the key there for a
  non-BBL 3MF as well, but the call site is shared with the browser's direct 3MF loads and needs a
  CLI-only signal; its whole-file comparison is also blocked by the printer-key and filament-vector
  differences below.
- `--export-settings` on non-BBL 3MFs, including this key. The direct load behind it
  (`load_project_with_preparation`) runs `apply_retained_preset_defaults`
  (`project/load/settings.rs:11-12`), the counterpart of `load_project_preset`
  (`OrcaSlicer.cpp:3164-3199`). That fill adds every printer key the project lacks from its system
  machine preset, `upward_compatible_machine` among them, before
  `initialize_cli_loaded_project_config` runs. Native skips the fill for a non-BBL 3MF:
  `load_project_preset` returns on an empty system name, and `current_printer_system_name` is set
  only inside the BBL block (1863, 1867, 1888). The same fill gives the non-BBL document 16 more
  printer keys than Native's, and the filament vectors differ in length as well; all of this belongs
  to the non-BBL retained preset change.
- `load_filament_count > 0` and the set of loaded file names in 2531: no observable difference on
  the Ares CLI today. A direct 3MF with only one of `--load-settings` / `--load-filaments` is still
  rejected by Ares.
- `--export-3mf` with preset files on a 3MF input: Ares does not accept that argument combination.
