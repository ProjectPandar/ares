# Native headless 3MF export with independent saved-project presets

## Upstream boundary

In OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`), `CLI::run` imports the first 3MF with `LoadModel | LoadConfig | AddDefaultInstances | LoadAuxiliary` (`src/OrcaSlicer.cpp:1740–1768`), setting `is_bbl_3mf` from the archive's `Application`/`OrcaSlicer` metadata markers (`src/libslic3r/Format/bbs_3mf.cpp:4160–4187`). Only BBL-marked archives populate `current_printer_name`, `current_process_name`, `current_filaments_name`, `current_inherits_group`, `filament_count`, and create `upward_compatible_machine` (`src/OrcaSlicer.cpp:1759, 1820–1896`).

`CLI::run` reads `--load-settings` and `--load-filaments` independently (`src/OrcaSlicer.cpp:2280–2470`):

- `--load-settings` accepts zero, one (machine-only or process-only), or two (machine and process) preset files (`2280–2366`).
- `--load-filaments` reads filament slots independently (`2369–2470`). `filament_count` takes `load_filament_count` only when `filament_count == 0` (`2528–2529`), so unmarked archives keep `filament_count == 0` whenever no filament file is supplied.
- Compatibility adjudication (`2922–3004`, `src/libslic3r/Preset.cpp:940–970`) checks all four combinations of supplied vs retained machine and process against `is_compatible_with_printer`, rejecting incompatible pairs with `CLI_PROCESS_NOT_COMPATIBLE` (`-17`). For unmarked archives `current_printer_name` and `current_printer_system_name` remain empty, so an incompatible new machine cannot enter the BBL machine-switch branch (`2972`) while a process-only replacement compares against an empty printer name (`Preset.cpp:968`).
- `load_project_preset` (`3167–3200`, `Preset.cpp:1090–1119`) fills absent default-preset keys for any retained machine or process when its resolved system name is non-empty (BBL-marked archives only), excluding the seven host connection keys (`print_host`, `print_host_webui`, `printhost_apikey`, `printhost_cafile`, `printhost_user`, `printhost_password`, `printhost_port`).
- Preset groups (`inherits_group`, `different_settings_to_system`) are resized to `filament_count + 2` (`3202–3205`). Only supplied preset kinds clear or replace their columns and apply their options over `m_print_config` (`3099–3162`, `3213–3256`, `3344–3411`, `3509–3774`); omitted kinds retain their embedded values and group entries.
- The single-filament mapped prime-tower disable rule requires `is_bbl_3mf && load_filament_count > 0 && load_filaments_set.size() == 1` (`2531–2535`, `4316–4345`).
- Mixed-filament metadata resizing runs only when `filament_count > 0` (`4243–4244`). `FullPrintConfig` normalization (`4252–4259`) precedes `CLI::export_project` -> `store_bbs_3mf` -> `_BBS_3MF_Exporter::_save_model_to_file` (`8081–8086`, `8376–8405`, `src/libslic3r/Format/bbs_3mf.cpp:6418, 6755–6769, 8199–8205, 8574–8610`).

## Rust destination

Extend the saved-project export in [Native headless 3MF export with explicit project presets](cli-export-3mf-saved-presets.md) by reusing the existing saved-project preset composition rather than adding another composition path in the exporter:

- `crates/ares-core/src/project/export/saved_presets.rs`: remove the complete-selection restriction (`2 settings + 1 filament`) for saved 3MF archives while preserving `require_complete_selection` for plain geometry-only 3MF inputs in `project_settings.rs`. Defer BBL machine-switch exports that would require writing a project-embedded `(auto)` preset file (`bbs_3mf.cpp:6777–6786, 8208`).
- `crates/ares-core/src/profiles/load_settings/project.rs` and `crates/ares-core/src/profiles/load_settings.rs`: route saved-project exports with any independent subset of flattened `--load-settings` and/or `--load-filaments` — including pass-through when both lists are empty — through `Project::from_3mf_with_profiles`.
- `crates/ares-core/src/project/domain/profile_input/three_mf.rs`, `overlay.rs`, `overlay/cli_bookkeeping.rs`, and `overlay/identity_slots.rs`: match upstream BBL vs unmarked handling of `filament_count`, retained default-preset key completion (`apply_retained_preset_defaults` only on BBL-marked archives), per-kind `different_settings_to_system` and `inherits_group` updates (clearing only supplied kinds and retaining omitted kinds), filament device-option application only when filaments were actually supplied (`selected > 0`), and pre-merge compatibility adjudication.
- `crates/ares-core/src/project_slice/compatibility.rs` and `crates/ares-core/src/project_slice/profile_entry.rs`: port Branch 3 (process-only) and BBL-family distinction in `reject_incompatible_load_settings` over the pre-merge embedded config for 3MF inputs (`OrcaSlicer.cpp:2922–3004`).
- `crates/ares-cli/src/export_3mf.rs`: forward `SliceError::ProcessNotCompatible` through `slice_error_to_failure` (`-17` / POSIX exit `239`) while keeping unsupported export errors on the `-13` project-export failure boundary.

## Included and deferred

Included for one saved single-nozzle FFF 3MF project (one plate, one printable translation-only instance of one ordinary unpainted part, no cached slice or auxiliary assets, no explicit arrangement or orientation), both BBL-marked and unmarked:

- `--load-settings machine.json;process.json` without `--load-filaments` (retains saved filament).
- `--load-filaments filament.json` without `--load-settings` (retains saved machine and process).
- `--load-settings machine.json` alone (same machine name or a printer already compatible with the embedded process) and `--load-settings process.json` alone (retaining the omitted kinds).
- Single-kind `--load-settings` combined with `--load-filaments` (`machine + filament`, `process + filament`).
- Pass-through `--export-3mf` with neither `--load-settings` nor `--load-filaments` (retains all embedded presets).
- Rejection of incompatible process-only replacement on a BBL-marked saved project with exit `-17` (`CLI_PROCESS_NOT_COMPATIBLE`) and no output archive.

Deferred:

- BBL machine-switch export emitting `Metadata/process_settings_1.config` (`OrcaSlicer.cpp:3009–3053`, `bbs_3mf.cpp:6777–6786, 8208`).
- Inherited external presets, `--uptodate` / installation resource refresh, sparse empty filament list slots, multi-filament or multi-nozzle saved exports, sliced export combined with preset flags, plate overrides, extra instances, and non-ordinary parts.
- Plain geometry-only 3MF and STL export rules remain unchanged.

## Failure modes

- An incompatible machine/process combination (such as process-only with `compatible_printers` excluding the BBL project's printer, or machine-only with an incompatible printer on an unmarked project) exits with `CLI_PROCESS_NOT_COMPATIBLE` (`-17`, POSIX `239`), empty stderr, and writes no archive.
- A deferred saved-project export shape or BBL machine-switch project-embedded preset fails with `CLI_EXPORT_3MF_ERROR` (`-13`, POSIX `243`) and writes no archive.
- Invalid or unresolved (`inherits`) preset files fail before writing an archive.

## Verification

Drive both Ares and pinned OrcaSlicer v2.5.0 (official and seeded modes) with the same top-level `--export-3mf` argv over synthetic BBL-marked and unmarked saved projects carrying non-default embedded machine, process, filament, `enable_prime_tower`, `different_settings_to_system`, and `inherits_group` values. Exercise settings-only (`machine;process`), filaments-only, machine-only, process-only, single-setting plus filament, and pass-through exports. Compare every uncompressed ZIP member by name, exact archive order, and byte payload, and assert repeated Ares exports produce identical ZIP bytes. For incompatible process-only on a BBL-marked saved project, assert matching exit code (`239`), stdout/stderr bytes, and absence of the output archive.
