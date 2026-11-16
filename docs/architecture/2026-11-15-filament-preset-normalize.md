# Always selection: filament presets load with one value per filament

## Problem

The plater's Always selection composes each filament option from the selected filament presets.
A Bambu Lab H2D filament preset stores some non-variant vectors with one entry per extruder
variant, such as `slow_down_min_speed` with four entries in `Generic TPU @BBL H2D` and three in
`Bambu PLA Basic @BBL H2D`. Ares keeps every entry, so the H2D project's config holds
`slow_down_min_speed = 10,10,10,10,20,20,20`. OrcaSlicer's GUI export of the same project holds
`10,20`.

## Upstream reference

- `PresetCollection::load_presets` calls `Preset::normalize` on every loaded preset after it
  applies the parent or the default preset (Preset.cpp:1947).
- `Preset::normalize` (Preset.cpp:555-599), for a config that holds `filament_diameter`:
  - `n` is the size of `filament_diameter`. A filament preset has no
    `single_extruder_multi_material`, so that branch is the one taken (Preset.cpp:558-565).
  - `DynamicPrintConfig::set_num_filaments(n)` (PrintConfig.cpp:10654-10673) resizes each
    `PrintConfigDef::filament_option_keys()` vector (PrintConfig.cpp:9055) the config holds,
    except `default_filament_profile` and `filament_options_with_variant`.
  - Each `Preset::filament_options()` vector the config holds (Preset.cpp:1524, :1737) is
    resized to `n`, except `compatible_prints`, `compatible_printers`,
    `filament_options_with_variant` (PrintConfig.cpp:10153) and `filament_dev_options`
    (PrintConfig.cpp:10294) (Preset.cpp:576-591). `filament_settings_id` is resized too
    (Preset.cpp:592-598).
- `ConfigOptionVector::resize` (Config.hpp:687-716) truncates a longer vector and extends a
  shorter one with its own first value, or with the default's first value when it is empty.
- `PresetBundle::full_fff_config(false)` then appends each filament's vector in order
  (PresetBundle.cpp:5736-5757), so a normalized preset adds one value per filament.

## Destination

`crates/ares-core/src/profiles/project_selection/collection.rs`, `load_config`: after
`extend_default_config_length`, a filament preset's config goes through the filament part of
`Preset::normalize`. The resize lives in
`crates/ares-core/src/profiles/preset_normalization.rs` beside `normalize_machine`. It uses the
filament option keys of `project_refresh::keys` (`Preset::filament_options()`) and the dev keys
of `project_refresh/filament.rs` (`filament_dev_options`).

## Included

- Resizing the `Preset::filament_options()` vectors of a loaded filament preset to the size of
  its `filament_diameter`, skipping the variant, dev and compatibility keys.
- `filament_settings_id` when the preset holds it.
- The `set_num_filaments` keys that are not in `Preset::filament_options()` (the printer
  retraction keys, `filament_colour`, `default_filament_profile`) need no port.
  `remove_invalid_keys` erases them right after `normalize` (Preset.cpp:1949), and the Ares
  filament preset config never holds them.

## Deferred

- The printer branch of `Preset::normalize` (`set_num_extruders`). `normalize_machine` already
  covers the machine presets, and this slice does not change it.
- `handle_legacy_sla` (Preset.cpp:600): no SLA presets are loaded.

## Failure modes and how each is ruled out

1. **A variant key loses its variants.** `filament_options_with_variant` keys are skipped, as
   upstream skips them. `filament_max_volumetric_speed` keeps `3.2,3.2,3.2,3.2` and `25,40,25`.
2. **A dev key is cut.** `filament_dev_options` are skipped.
3. **A preset with several `filament_diameter` entries is cut to one.** `n` is the preset's own
   `filament_diameter` size, not 1.
4. **An empty vector cannot be extended.** It takes the option default's first value, as
   `resize(n, defaults.option(key))` does.
5. **The CLI paths change.** Only the Always collection load runs it; the CLI selection and
   refresh do not go through `Collection::load`.

## Tests

`crates/ares-cli/tests/project_preset_selection/filament_composition.rs`:
- two filament presets of a two-extruder printer. Each holds a non-variant vector with one
  entry per extruder variant (`slow_down_min_speed`) and a variant vector
  (`filament_max_volumetric_speed`);
- the Always output's filament options equal `full_fff_config(false)` composed from the
  presets the preset-select probe loads: one `slow_down_min_speed` value per filament, every
  variant value kept.

`web/e2e/dualExtruder.spec.ts` checks the H2D project end to end, against the plater's export.
