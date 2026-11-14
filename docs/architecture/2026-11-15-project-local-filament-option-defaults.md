# Always selection: a project-local filament preset takes the option defaults the project lacks

## Problem

A first-run project (no vendor installed besides Custom) opens with a project-local filament
preset `<name>(<file>.3mf)`. `tests/parity/orca_cli_ender3.3mf` stores no
`filament_retract_length_nc`. The plater's config for it holds `filament_retract_length_nc = 10`
(the preset-select probe's `full_fff_config`); the browser composes nil, so its G-code drops the
`; filament_retract_length_nc = 10` line that the plater and the CLI reference print.

## Upstream reference

- The plater loads the 3MF config over `FullPrintConfig::defaults()` (Plater.cpp:9463) before
  `PresetBundle::load_config_model` (Plater.cpp:9622); the probe does the same with
  `DynamicPrintConfig::full_print_config()`.
- `load_config_file_config` builds the project-local filament preset with
  `PresetCollection::load_external_preset` (PresetBundle.cpp:6354 for one filament, the loop
  after it for several). `Preset::load_external_config` starts from the default preset and
  `apply_only`s every key of the combined config, so each option key takes the 3MF value or the
  option default.
- The default filament preset nils every nullable filament override key
  (`is_filament_extruder_override_key`, PrintConfig.cpp:130-139; PresetBundle.cpp:508-518). The
  combined config does not: it holds the option default. For the generated override keys that
  default is nil too; `filament_retract_length_nc` defaults to 10 (PrintConfig.cpp:8931-8940).

## Destination

`crates/ares-core/src/profiles/project_selection/config.rs`: `project_value` is a key of the
completed project config: the 3MF value or, for a `FullPrintConfig` option the 3MF lacks, the
option default (`project_refresh/keys.rs`, `full_config_value`). `PresetConfig::apply_project`
(Preset.cpp:1090-1105) and `load_external_filaments` (`external.rs`, PresetBundle.cpp:6365-6388)
read the project through it. The project config itself stays as loaded, so the selection's
presence checks on it do not change.

## Included

- Every preset the Always selection loads from the project: process, printer, a single filament
  (`Collection::materialize`) and several filaments (`load_external_filaments`).
- With several filaments, filament `i` takes a vector option's value with
  `ConfigOptionVector::get_at` (Config.hpp:676-680): the first value when the vector is shorter,
  as a one-value option default is.

## Deferred

- `pellet_flow_coefficient`: the plater's config for these projects holds `0.4157` and Native
  prints the option when the config holds it (the H2D export slice), so the browser line matches
  the plater, and the CLI-route reference, which slices the 3MF config without it, lacks it. This
  is inferred from the config: the Native CLI rejects the plater's export of these projects with
  -17 (OrcaSlicer.cpp:2575-2620), so no plater G-code exists to compare. No code change.

## Failure modes and how each is ruled out

1. **An override key whose option default is nil gets a value.** The option default is used, and
   it is nil for those keys; the test compares all of them with the plater's config.
2. **A key the project holds is replaced.** Only keys the project lacks take the default.
3. **Process or printer presets change.** Outside the filament override keys the default preset
   holds the option default, so their values are the same; the existing selection tests cover them.
4. **Presence checks on the project config change.** The completion is per key, read where a
   preset loads; the project config is not completed.

## Tests

`crates/ares-cli/tests/project_preset_selection/filament_composition.rs`: a project whose
filament preset is not installed and which stores none of the override keys. Native premise: the
filament is project-local and the probe's `full_fff_config` holds `filament_retract_length_nc =
["10"]`. Every filament override key of the Always output equals the probe's `full_fff_config`.
