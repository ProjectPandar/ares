# Pressure advance variant layout

This is a source-cited rewrite slice of `OrcaSlicer/src/libslic3r` `filament_options_with_variant` and
the CLI per-filament preset refresh that resizes those options. It is pinned to upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It completes the variant layout of the six pressure
advance options, whose per-filament gather already exists. It adds no Ares-owned layout.

## Source boundary

- `src/libslic3r/PrintConfig.cpp:10199-10205`: `filament_options_with_variant` lists
  `enable_pressure_advance`, `pressure_advance`, `adaptive_pressure_advance`,
  `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs` and
  `adaptive_pressure_advance_bridges`.
- `src/libslic3r/PrintConfig.cpp:11431-11457`, `normalize_filament_values_to_variants`: a listed
  option stored once per filament, or once for all of them, is laid out one value per entry of
  `filament_self_index`.
- `src/OrcaSlicer.cpp:3658-3793`: the CLI per-filament preset refresh.
  - A listed option kept from the project (`:3698-3701`) and a refreshed one (`:3757-3759`) are
    first grown to the filament's slot range with `resize(n, opt_vec_src)`, then passed to
    `set_with_restore_2`.
- `src/libslic3r/Config.hpp:687-718`, `ConfigOptionVector::resize`:
  - An empty vector is filled from the supplied default's front (`:698-711`).
  - A non-empty vector is extended with its own front value (`:712-715`). The supplied source
    does not change values the vector already had.

## Destination

- `crates/ares-core/src/options/registry/key_lists.rs`, `FILAMENT_OPTIONS_WITH_VARIANT_KEYS`: add
  the six options in sorted position. The load layout (`project/raw_settings/filament_variants.rs`),
  the refresh (`profiles/project_refresh/filament.rs`, `filament/slot.rs`) and the parameter size
  (`options/parameter_size.rs`) read this list and need no other change.
- `crates/ares-core/src/profiles/project_refresh/filament.rs`, `replace_variants`: grow a short
  destination with its own front value, or the source front when it is empty, as
  `ConfigOptionVector::resize`. Today it always pads with the source front.
- The per-filament gather in `options/project_variants/filament.rs` is unchanged.

## Included

- Creality Ender-3 projects with four variant slots of one filament
  (`filament_self_index = 1,1,1,1`): the six options are written once per slot.
- Prusa MK4 projects with two variant slots (`filament_self_index = 1,1`): the same, two values.
- The resize rule for every listed option. Two-filament BBL X1C projects whose single authored
  `enable_pressure_advance = 1` is not defined by the filament presets keep `1,1`. With the
  source-front padding, the second filament takes the preset value `0`.

## Deferred

- The remaining omitted members of `filament_options_with_variant` (pre-cooling, preheat,
  retraction, ramming) are owned elsewhere.
- Pressure advance command emission, adaptive pressure advance fitting and the dynamic slot
  selection are not changed.

## Failure modes

1. Only the list changes: the X1C precedence projects write `enable_pressure_advance = 1,0`.
2. Only the resize changes: the six options stay one value wide.
3. The resize replaces existing values with the source instead of only extending the vector.
4. An empty destination is not filled from the source front.
5. The list entry is not sorted, or duplicates an existing key.
6. `adaptive_pressure_advance_model`'s quoted CSV string is split while it is laid out.

## Tests

`crates/ares-cli/tests/orca_parity/option_coverage/pressure_advance_variant_layout.rs` uses the
variant footer check (Native vs API and CLI) on existing fixtures:

- Ender-3 four slots (`auto-brim-object-adhesion/auto_brim-pla.3mf`): all six options with four
  values, for example `enable_pressure_advance = 0,0,0,0` and
  `pressure_advance = 0.02,0.02,0.02,0.02`.
- MK4 two slots (`mk4-layer-change/mk4base.3mf`): all six options with two values.
- X1C two filaments (`bbl-pressure-advance-precedence/bbl-stock-pa-control.3mf`):
  `enable_pressure_advance = 1,1`, `pressure_advance = 0.02,0.02`. It already matches and must
  keep matching.
