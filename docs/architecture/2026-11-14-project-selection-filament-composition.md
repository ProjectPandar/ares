# Project preset selection: filament composition

## Problem

The Always selection (`{"from_project": true, "load_and_select": "always"}`,
`docs/architecture/project-preset-selection-always.md`,
`docs/architecture/project-preset-selection-multifilament-always.md`) selects one filament
preset per project filament, then composes the project config from the selection
(`refresh` in `crates/ares-core/src/profiles/project_selection.rs`). For the filaments it
copies only scalar options, taking each from the last filament, and keeps every filament
vector of the project as it was. OrcaSlicer's plater composes them from the selected
presets instead. For a two-extruder project (Bambu Lab H2D, a TPU and a PLA):

- filament 2 keeps the project's values (`nozzle_temperature` 240 against the preset's 220,
  `filament_cost` 24.99 against 15.99);
- filament keys the project does not store take option defaults with one element instead
  of the preset values (`filament_retract_length_nc`, `filament_pre_cooling_temperature`,
  `filament_dev_*`);
- `filament_ids` keep the project's ids instead of the presets' `filament_id`;
- `filament_extruder_variant` keeps the project's one variant per filament, so the slice
  cannot pick the TPU's High Flow variant for the High Flow extruder `filament_map` puts it
  on.

The presets the selection loads also miss a step: a filament preset with two variants that
does not set a variant option keeps the one-element default, where OrcaSlicer pads it to
the variant count when it loads the preset.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f7`):

- `src/libslic3r/PresetBundle.cpp:6366-6430`, `load_config_file_config`: the multi-filament
  branch scatters the project values into one config per filament and loads each with
  `load_external_preset`, the first with `Always`; `filament_presets[i]` is the loaded
  preset's name.
- `src/libslic3r/PresetBundle.cpp:5534-5840`, `PresetBundle::full_fff_config`, with
  `apply_extruder` false, the form `full_config_secure` (`:5470-5472`) gives the plater's
  project save (`src/slic3r/GUI/Plater.cpp:19779`):
  - one filament (`:5621-5651`): the edited filament preset is applied whole;
    `filament_self_index` is one `1` per `filament_extruder_variant` entry; the id is the
    edited preset's `filament_id` (`:5636`);
  - several filaments (`:5652-5774`): filament `i` is `filaments.find_preset(name, true)`
    of `filament_presets[i]`, the edited copy for the selected preset; its id is that
    preset's `filament_id` (`:5679`); a scalar option is the first filament's
    (`:5731-5735`); a vector option is the filaments' values appended in order
    (`:5738-5757`); `filament_self_index` numbers each filament's
    `filament_extruder_variant` entries (`:5763-5773`);
  - `filament_settings_id` is `filament_presets` and `filament_ids` the collected ids
    (`:5819`, `:5821`).
- `src/libslic3r/Preset.cpp:313-371`, `extend_default_config_length`, which
  `PresetCollection` runs on each preset it loads (`:1930`, `:1944`): a variant option is
  resized to its class's variant count, `filament_extruder_variant`'s length for filament
  options.

The variant a filament slices with is chosen later, from `filament_map` and the extruder's
volume type (`update_values_to_printer_extruders`, `full_fff_config` with `apply_extruder`
true, `:5626`, `:5721`); the slicing path already ports it. It needs the presets' variant
vectors in the project config to choose from.

## Destination

- `crates/ares-core/src/profiles/project_selection.rs`, `compose_filaments`: the filament
  part of `full_fff_config(false)` above, over the filament states the selection loaded.
  A filament's preset is the first state's edited copy when the names match, else the
  record of that name in the first state's records, which is the collection after the last
  load.
- `crates/ares-core/src/profiles/project_selection/collection.rs`, `load_config`: the
  existing `extend_default_config_length` port (`profiles/preset_normalization.rs`) runs on
  each loaded preset before `normalize_machine`.

## Included

- Every filament option of one or several filaments, `filament_self_index` and
  `filament_ids`.
- Variant padding of every preset the selection loads.

## Deferred

- `inherits_group`, `compatible_machine_expression_group`,
  `compatible_process_expression_group` and `different_settings_to_system`, which
  `full_fff_config` regroups (`:5837-5840`).
- `filament_map` resizing to the filament count (`:5560-5562`).
- The `nil` replacement of `extend_default_config_length` (`set_nil_to_default`).

## Failure modes and how each is ruled out

1. **A filament option differs from the selected preset's.** A synthetic two-extruder
   project (variants Standard and High Flow, the TPU mapped to the High Flow extruder) has
   two system filaments with two variants each and project values that differ from them or
   are missing. The preset-select probe loads it; the test asserts the Native premise (both
   presets selected unchanged), composes `full_fff_config(false)` from the probe's selected
   and loaded presets, and compares every filament option of the Always output with it,
   typed by the option registry.
2. **The single-filament path differs.** The same project with the TPU only.
3. **Non-first filaments read the wrong preset.** The three-filament case whose second
   filament loads the selected preset's name takes the selected preset's edited values.
4. **The other selection cases change.** The rest of the project preset selection cases
   run against the probe.

## Commits and acceptance

1. `docs(core)`: this document.
2. `test(core)`: the composition cases and the three-filament values; they fail before the
   fix.
3. `fix(core)`: the composition and the preset padding.

Checks: fmt, clippy, the wasm32 check, the project preset selection cases with the
preset-select probe, and the ares-core cases. The cases write the probe requests and
results, the Native composition and the Always composition under
`ARES_PARITY_ARTIFACT_ROOT`.
