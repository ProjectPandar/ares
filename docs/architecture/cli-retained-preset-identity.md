# CLI Retained-Preset Identity During `--uptodate` Refresh

## Upstream Boundary

Source-cited Rust rewrite slice of OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) covering retained preset identity updates in `CLI::run`:

- **`src/OrcaSlicer.cpp:1819–1896`**: reads the saved project's `printer_settings_id`, `print_settings_id`, `filament_settings_id`, and `inherits_group`. When `inherits_group` has at least two entries, the last non-empty entry supplies `current_printer_system_name`, entry `0` supplies `current_process_system_name`, and intermediate entries supply `current_filaments_system_name`; empty or missing entries fall back to the saved user names.
- **`src/OrcaSlicer.cpp:2539–2690`**: when `is_bbl_3mf && up_config_to_date`, flat refresh files loaded from `--uptodate-settings` (`:2540–2624`) or from installed `profiles/BBL/{machine,process}_full/<system_name>.json` (`:2626–2690`) stage the loaded file's `name` onto the config before moving it into `load_machine_config` / `load_process_config`:
  - machine (`:2561`, `:2644`): `config.set("printer_settings_id", config_name, true);`
  - process (`:2615`, `:2684`): `config.set("print_settings_id", config_name, true);`
- **`src/OrcaSlicer.cpp:2693–2768`, `:3571–3604`, `:3738`**: retained filament refresh never sets `filament_settings_id` on `load_filaments_config`. In the filament merge loop, `loaded` is `false` for retained slots, so `:3592–3604` does not touch `filament_settings_id` or `inherits_group`, and `:3738` explicitly skips `filament_settings_id`.
- **`src/OrcaSlicer.cpp:3098–3162` (`update_full_config`), `:3202–3255` (machine), `:3344–3405` (process)**:
  - `:3203–3204` resizes `inherits_group` to `filament_count + 2` without clearing retained entries. Only replacement branches (`!new_printer_name.empty()` at `:3228–3231` and `!new_process_name.empty()` at `:3377–3380`) overwrite `inherits_group`.
  - Retained `--uptodate` machine (`:3216–3222`) and process (`:3348–3365`) branches decode only the authored `different_settings_to_system` column via `Slic3r::unescape_strings_cstyle` (not `PresetBundle::project_different_keys`).
  - `update_full_config` keeps keys listed in that authored keep set (unless released by `--skip-modified-gcodes`) and skips metadata keys `{compatible_prints, compatible_printers, model_id, inherits, dev_model_name, name, from, type, version, setting_id, instantiation}` (`:3148–3150`). Because `printer_settings_id` and `print_settings_id` are not skipped, any non-empty flat `load_machine_config` or `load_process_config` overwrites `m_print_config`'s corresponding settings ID with `config_name`, unless the saved override column explicitly lists that identity key.
- **`src/OrcaSlicer.cpp:3164–3200` (`load_project_preset`) and `src/libslic3r/PresetBundle.cpp:5520–5532` (`PresetBundle::project_different_keys`)**: runs only when a retained machine or process has no flat refresh config (`load_machine_config.empty()` / `load_process_config.empty()`), covering both non-`--uptodate` runs and `--uptodate` runs that fall back to the system preset resolver. Both `bookkeeping_keys` (`:3171–3172`) and `ignore_settings_list` (`PresetBundle.cpp:5520–5523`) exclude `printer_settings_id` and `print_settings_id`, so the saved user names stay intact.

## Rust Destination Boundary

`crates/ares-core/src/profiles/project_refresh/preset.rs::refresh` currently combines the flat-file `--uptodate` refresh (`update_full_config`) and the system-catalog fallback (`load_project_preset`) in a single temporary adapter loop. Both paths previously dropped `printer_settings_id` and `print_settings_id` through `NativeRefreshFile` option filtering, `dirty_keys`, and `bookkeeping`.

Update `crates/ares-core/src/profiles/project_refresh/preset.rs::refresh` to follow the upstream split for retained machine and process presets:

1. Identify whether the retained kind is backed by a flat `NativeRefreshFile` under `uptodate == true` (`PresetLookup::Native` with `presets.file(kind, slot)`).
2. On that flat-file branch (`OrcaSlicer.cpp:2539–2690`, `:3098–3162`, `:3213–3254`, `:3345–3405`):
   - Stage `file.name()` onto `printer_settings_id` (machine) or `print_settings_id` (process) before the default-script step and merge loop.
   - Use `authored_dirty_keys(&dirty, slot)` as the keep set instead of `dirty_keys` (which injects `PresetBundle::project_different_keys` identity protections).
   - Do not filter that kind's own settings ID through `bookkeeping`, and apply `update_full_config`'s keep-set skip rule directly (`protected.contains(key)`).
   - Leave `inherits_group` untouched and keep `filament_settings_id` unchanged.
3. On every non-flat branch (`uptodate == false`, `PresetLookup::Catalog`, or `PresetLookup::Native` falling back to `system_catalog` because `presets.file(kind, slot)` is `None`), keep the existing `load_project_preset` behavior (`dirty_keys` + `bookkeeping`) so saved user preset names remain preserved.

## Included and Deferred Behavior

- **Included**:
  - Machine `printer_settings_id` and process `print_settings_id` adoption from flat `--uptodate` refresh files (`--uptodate-settings` and installed `profiles/BBL/{machine,process}_full` resources).
  - Authored `different_settings_to_system` protection when a saved column explicitly lists `printer_settings_id` or `print_settings_id`.
  - Preservation of saved `inherits_group` entries and retained `filament_settings_id` user names.
  - Preservation of distinct saved user names when `--uptodate` is omitted, when `--uptodate` has no flat file for a kind (`load_project_preset` fallback), or when a kind is served from the system catalog.
- **Deferred**:
  - Other `update_full_config` vs `load_project_preset` key-filter differences unrelated to preset identity (`compatible_printers_condition`, printer connection keys such as `print_host` / `printhost_*`, and `print_compatible_printers` propagation from refreshed process files).
  - Non-standard `inherits_group` lengths (`size < 2` or `size != filament_count + 2`) during project refresh lookup.
  - Plater / GUI `PresetBundle::load_config` preset selection (`ProjectPresetPolicy::Always`) and `--load-settings` replacement selection.

## Failure Modes

- Overwriting `printer_settings_id` or `print_settings_id` when `--uptodate` is omitted or when a kind falls back to the system preset catalog instead of loading a flat refresh file.
- Updating `printer_settings_id` for machine refresh while forgetting symmetric `print_settings_id` handling for process refresh (or vice versa in partial `--uptodate-settings` lists).
- Overwriting `filament_settings_id` or clearing `inherits_group` during retained refresh.
- Ignoring an explicitly authored `printer_settings_id` or `print_settings_id` entry in `different_settings_to_system`.

## Verification

Add end-to-end CLI parity tests in `crates/ares-cli/tests/orca_parity/cli_uptodate_refresh/preset_identity.rs` using a saved 3MF with distinct user names for machine, process, and filament whose `inherits_group` names the stock system parents. Compare complete G-code against live runtime Native using the unchanged `load_settings_cli::banner_normalized` comparator across:

1. `--uptodate` with flat `--uptodate-settings` (and installed `profiles/BBL/{machine,process,filament}_full` resources): adopts machine and process source IDs while preserving `filament_settings_id` and `inherits_group`.
2. Partial `--uptodate-settings` (machine-only and process-only with stock resource fallback): adopts only the flat-refreshed kind's ID and preserves the catalog-refreshed kind's saved user ID.
3. Non-`--uptodate` controls (bare and with inactive `--uptodate-settings`): preserves all distinct saved user IDs.
4. Bare `--uptodate` without flat files (catalog fallback): preserves all distinct saved user IDs.
5. Authored `printer_settings_id` and `print_settings_id` in `different_settings_to_system`: keeps both saved user IDs under flat `--uptodate` refresh.
