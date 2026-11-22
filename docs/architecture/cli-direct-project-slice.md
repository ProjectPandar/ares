# Native-form project slicing with `--datadir` and `--slice 0`

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/libslic3r/PrintConfig.cpp:12780–12784,13142–13144` defines `slice` and
  `datadir`. `CLI::setup` in `src/OrcaSlicer.cpp:8169–8206,8237–8241`
  derives installed resources independently of the parsed data directory.
  `Slic3r::set_data_dir` in `src/libslic3r/utils.cpp:358–375` creates a
  missing nonempty root. Setup failure returns `CLI_INVALID_PARAMS` before
  slicing (`src/OrcaSlicer.cpp:1329–1333`).
- `CLI::run` lazily opens the read-only `PresetBundle` for explicit inputs
  that need user preset resolution (`src/OrcaSlicer.cpp:2083–2103,2118–2134`).
  `AppConfig::config_path` and `load_if_exists` use the data root
  (`src/libslic3r/AppConfig.cpp:1911–1925,1964–1974`);
  `PresetBundle::load_presets` and `load_user_presets` select the configured
  user folder or the default (`src/libslic3r/PresetBundle.cpp:881–905,1411–1428`).
  A saved project with no external preset inputs does not load that bundle
  merely because the option was supplied.
- `CLI::run` establishes the zero-selection preflight and visits every
  selected plate in index order (`src/OrcaSlicer.cpp:6441–6445,6514–6529`).
  Export with a directory uses `plate_<one-based index>.gcode`
  (`7182–7189`). `record_exit_reson` writes the Linux `result.json`
  (`535–568`); success uses the selection as `plate_index` (`8133–8136`)
  and includes the sliced plates in visitation order (`7273`). An error
  in plate validation reports that plate's one-based ID (`7020–7040`).

## Rust destination and scope

For a single direct 3MF project with `--outputdir`, accept `--datadir` in
`ares-cli/src/args.rs` and apply it in `direct_project.rs` before reading
slice inputs. Omitted and empty values do not create a directory; a new
nonempty root is created, an existing root is left intact, and failed
creation reports setup failure without starting the slice or writing a
slice result. Pass the root to the existing lazy loaded-preset producer in
`preset_sources.rs` when resolving explicit replacement presets. Do not
redirect `installed_resources.rs` or reload embedded project presets from
the data root.

Use the existing `slice_plates.rs` adapter and `ares_core::slice_cli_project_plates`
for direct selection zero, rather than calling the single-plate adapter
once per plate. The shared adapter must finish preflight before exporting
any plate. On success it writes numbered files and one report with
`plate_index: 0`; on failure it reports the actual failing plate and
preserves the existing batch publication rules. The positive-selection
adapter remains the existing compatibility shell around the selected-plate
API; its output and supported options are unchanged by this selection
change. No new core plate traversal or filesystem work in core is needed.

This slice does not change STL, settings-only export, or subcommand argv.
Report-only invocation without `--outputdir`, and extending the all-plate
adapter to additional actions such as strict slicing, external layer G-code
or statistics export, are separate changes. Do not silently discard any
such action on the newly admitted route.

## Verification

Exercise the built CLI and live OrcaSlicer with the same option list and
fixture path, no adapter-injected arguments, and separate working/output
roots. Use synthetic one- and two-plate projects in official and seeded
modes. Independently exercise `--slice 0` without `--datadir`, a positive
selection with `--datadir`, and both options together. Check root creation,
numbered output inventory, ordered report IDs, and whole-file G-code and
`result.json` using the existing generator and wall-clock comparators.
Include setup creation failure and a preflight rejection before publication.
Exercise loaded-preset resolution with a synthetic user preset so that
accepting the directory without using it cannot satisfy the contract.
