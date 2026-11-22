# Load settings and filaments independently when slicing a saved 3MF

Direct slicing of a saved 3MF must accept `--load-settings` without `--load-filaments`, and
`--load-filaments` without `--load-settings`. Absence means that no replacement was supplied for
that kind; it does not mean that the embedded options should be cleared or reconstructed from
slicer defaults.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/libslic3r/Format/bbs_3mf.cpp:4170-4186` identifies the BBL-family generator. In
  `src/OrcaSlicer.cpp`, `CLI::run:1759-1923` handles BBL-specific settings, but the imported 3MF
  configuration supplies `m_print_config` for either family (`:1994-1996`).
- `CLI::run:2280-2367` reads `load_configs` and selects the loaded machine and process separately.
  The machine and process branches at `:3212-3256` and `:3344-3406` use `update_full_config`
  (`:3099-3155`) to apply the supplied keys to the existing configuration. Not supplying one
  kind does not enable its replacement branch. Refresh rules are separate.
- `CLI::run:2369-2375,2416-2470,2528-2535` reads and indexes `load_filaments` independently. An
  embedded filament count is not replaced by the count of loaded files. `:3509-3510,3566-3572,
  3598-3622,3721-3728` update the supplied slots. The owning data structures are
  `libslic3r/PrintConfig.hpp:915` (`DynamicPrintConfig`) and `libslic3r/Config.hpp:451-474`
  (`ConfigOptionVector::set_at`). Entries not supplied keep their existing values.

## Rust boundary

- `ares-cli/src/direct_project.rs` currently treats the two options as a pair before dispatching
  either the positive plate or all plates path. Keep the loaded-preset route if either option is
  supplied, and pass an empty file list for an omitted option. With neither option, keep the saved
  project route. The CLI remains responsible for reading supplied files and reporting the slice.
- `ares-core/src/profiles/load_settings.rs` and `load_settings/project.rs` translate the independent
  file lists to a selected-profile overlay. The current embedded selection path only handles a
  standalone machine file. Extend the project-backed selection to carry a supplied machine and/or
  process with no supplied filaments. Keep the ordinary requirement for a filament selection when
  there is no embedded project configuration. The selected-profile representation is an adapter for
  applying the upstream `DynamicPrintConfig` updates, not a second source for absent preset kinds.
- `ares-core/src/project/domain/profile_input/three_mf.rs` and its overlay should preserve the
  embedded entries for absent kinds and apply supplied filament values by slot. On the loaded
  settings path, the CLI's post-mapping rule must see that no filament was loaded when only
  settings were supplied (`CLI::run:2531-2535`). Keep the existing BBL-family condition separate.
- All-plate slicing uses the same preset semantics through `ares-cli/src/slice_plates.rs`. No
  separate preset parser or public API is needed. `--datadir`, plate selection and reporting keep
  their existing roles.

## Included and deferred

Includes a saved FFF 3MF with embedded configuration, with only one of the two direct CLI options,
for a positive plate selection or `--slice 0`. Both BBL-family and other 3MFs use the independent
replacement rules. On settings-only loads, verify that changed process values take effect while
embedded filament values and identities remain. On filaments-only loads, verify that the supplied
slot changes without replacing the machine, process or remaining embedded slots.

Deferred: changes to the BBL-specific option presence and prime-tower family checks, installation
resource refresh, leading or interior empty filament list items, and no-preset vector expansion.
Do not change STL validation, GUI preset composition, or settings export behavior in this slice.

## Verification

Use synthetic projects and run the public direct CLI with the same argv on Ares and both Orca
reference modes. Test one option at a time with an observable option change, including settings
and filament values that must survive from the project. Include the positive and all-plate routes.
Compare whole G-code with the existing comparator and keep `result.json` artifacts; check status,
selected plate and stable output fields rather than asserting wall-clock durations. Compare the
broad CLI, preset and project families before and after, and run formatting, all-targets lint,
and the wasm32 check.
