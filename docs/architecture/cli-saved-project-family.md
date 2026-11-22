# Saved 3MF configuration follows the generator family in the CLI

## OrcaSlicer v2.5.0 boundary

`src/libslic3r/Format/bbs_3mf.cpp:4170–4186` identifies a BBL-family project from its generator metadata. `src/OrcaSlicer.cpp::CLI::run` uses that result at `:1759`, `:1820–1894` to collect saved preset names, count saved filament presets, and create `upward_compatible_machine` when absent. With no replacement preset files, the count for a non-BBL project remains zero (`:2528–2529`), even when the embedded print config has more than one filament. The CLI sets `inherits_group` and `different_settings_to_system` to the bookkeeping count plus two (`:3202–3205`). This count does not resize the embedded physical filament palette.

The same family branch supplies the names for composing retained presets. Retained filaments are resolved and merged only for BBL-family 3MFs (`CLI::run :2775–2815`, `:3509–3796`); that merge can grow short variant vectors to cover the retained slots. The machine/process fill (`:3164–3200`) uses `libslic3r/Preset.cpp::Preset::load_external_config` (`:1090–1118`) only when a retained system name was captured. A non-BBL project instead passes its embedded values through the ordinary `FullPrintConfig` defaults (`CLI::run :4255–4259`). This is not `DynamicPrintConfig::set_num_filaments` (`libslic3r/PrintConfig.cpp:10654–10673`).

## Rust destination

In the saved-project CLI path, use the family already present in `ProjectDocuments` to select which preset composition and bookkeeping to perform. The existing catalog and saved-config initializers in `project_slice/source.rs`, `project_slice/state.rs`, and `project/domain/preset_refresh.rs` are the current counterpart of the cited CLI composition. Keep the catalog merge and embedded bookkeeping count for BBL projects. For non-BBL projects, skip retained preset refresh and collection defaults, size both bookkeeping groups to two, and do not create a missing `upward_compatible_machine`. Keep the embedded filament options and existing keys. Keep the generic project and Plater routes on their existing composition; the GUI's `PresetBundle::load_config_file_config` (`libslic3r/PresetBundle.cpp:6158` onward) is a distinct path.

Explicit replacement, preset updating, settings export, Plater, and G-code serialization are outside this change. No change is made to which filaments the model uses.

## Verification

Slice a Native-exported synthetic two-slot 3MF with no preset flags. Run the same command against a copy with only its generator metadata marked non-BBL. Compare each whole G-code file to Native using the existing generated-by banner normalization, and check both commands succeed. The BBL project exercises retained preset vector expansion; the non-BBL copy exercises the embedded-vector and two-column paths.
