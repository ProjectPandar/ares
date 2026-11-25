# Retained-preset refresh when slicing selected 3MF plates

## Upstream behavior

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/OrcaSlicer.cpp:2539–2817`, loads retained system presets before slicing
any plates. The explicit input branch requires both a BBL-family project
(`is_bbl_3mf`) and `--uptodate`. It uses `--uptodate-settings` for retained
machine and process files and `--uptodate-filaments` for retained filament
slots. When a list is absent it tries the corresponding installed flat
presets. Missing flat files do not themselves abort slicing. The retained
filament resolver and unlisted-key policy are at `:2775–2817`; the analogous
machine and process resolver is at `:3165–3199`. The plate selection does not
choose a different refresh policy.

| Preset | Witness option | Input and application in `OrcaSlicer.cpp` | Ownership in `src/libslic3r` |
| --- | --- | --- | --- |
| Machine | `extruder_clearance_dist_to_rod` | `:2542–2554,2561–2602` validate and stage the named file; `:2629–2662` tries the installed file; `:3239–3243` applies the retained config. | `Preset.cpp:1617`, `PrintConfig.cpp:2815` |
| Process | `bottom_color_penetration_layers` | `:2605–2616` validates and stages the named file; `:2670–2685` tries the installed file; `:3389–3393` applies the retained config. | `Preset.cpp:1202`, `PrintConfig.cpp:1545` |
| Filament | `filament_density` | `:2693–2732` validates list length, type, and name for each slot; `:2737–2765` tries installed files; `:3571–3589` distinguishes retained slots from replacements before the merge. | `Preset.cpp:1527`, `PrintConfig.cpp:3499` |

The retained merge preserves project overrides recorded as different from the
system preset. Supplying a system catalog alone does not enable replacement of
existing unlisted values: `--uptodate` is part of that decision (`:2775–2779`,
`:3165–3199`).

## Rust boundary

This change builds on the saved-project family check. **Non-BBL projects
continue without retained refresh. BBL-family projects add the opt-in
`--uptodate` refresh to the selected-plate route.** With no opt-in, the
selection route retains the existing behavior and does not read either refresh
list, even if paths were supplied.

The `ares-cli` selection dispatch reaches `slice_plates::run` before the
non-selection route's refresh loader. The selected path currently forwards
neither list and only supplies an embedded system catalog. Reuse
`native_refresh::load_admitted` for filesystem inputs, retained name and count
validation, and installed-resource lookup. Report input failures through the
selected route's existing `--outputdir` result channel. Reuse the admitted
project for slicing.

In `ares-core`, pass the resulting `NativeRefreshPresets` into
`project_slice::cli_plates` and use `ProjectPresetRefresh::native` on its
`ProjectSource`, following `project_slice::validated::slice_cli_project`. Apply
the existing `UpwardMachineInput::RetainedRefresh` source policy. Initialize
the refresh once, before prechecking individual selected plates. The existing
selection and publication adapter remains unchanged outside the preset input
it supplies. No new preset merge, config-key definitions, or file access is
added to core.

### Scope

This covers saved 3MF selection with no `--load-settings`, `--load-filaments`,
or `--project-preset-catalog` override. Mixing refresh with those inputs is
separate because they have their own precedence rules. Direct CLI syntax
without the Ares `slice` subcommand, STL slicing, and `--export-settings` are
also separate. Selection order, prechecks, filenames, and result reporting do
not change. The `--slice 0` controls protect the shared initialization path;
this change does not redefine all-plates selection.

## Public CLI acceptance

Start with a public CLI test using a synthetic BBL-family saved 3MF with
retained values `44`, `6`, and `1.25` for the options above. Supply matching
system files with `45`, `8`, and `1.35`. Run Native at test time with the same
slice options as Ares (`--slice 1 --outputdir out`, plus the refresh flags;
Ares uses its `slice` subcommand). Compare `plate_1.gcode` as ordered bytes,
normalizing only the generator line. Assert the Native values and preset IDs
so an ineffective fixture cannot pass. The original implementation must fail
on the retained values after both executables successfully export the plate.

Exercise both lists together, each list alone, and `--uptodate` with installed
resources. Include unrefreshed and inactive-list controls; the inactive list
may name files that do not exist. An opted-in list with the wrong filament
count must return the same error and result report as Native, without G-code.
Run the focused cases and existing family and plate selection controls against
both the official and seeded Native references. Generate all Native outputs
at test time.
