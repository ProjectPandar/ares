# Retained project preset membership

## Rewrite boundary

Port retained-project preset default construction from OrcaSlicer revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. This is an applied dynamic
configuration construction correction, not a new export pipeline.

## Original print-host correction

`d1b8a399e` addressed
`runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`.
That test slices a solid-plate STL using WEMAKE3D selected profiles with empty
user overrides and compares complete ordered output. The extra Ares
`bbl_use_printhost = 0` was caused by treating a registered option as a static
`FullPrintConfig` member.

The change made `bbl_use_printhost.full_config_default` false and made preset
default construction use canonical wire defaults independently of static
membership. These are distinct contracts: false is encoded as `0`, not the
presentation text `false`. Reverting the wire-default correction is not a
valid membership repair.

Removing the static membership exposed a second defect: the direct embedded
project path does not construct absent retained-preset members. The selected
STL path and the retained 3MF path must not share an unconditional export rule.

## Native ownership

- `Preset.cpp:1573-1610`, `s_Preset_printer_options`, owns printer preset keys;
  `:1592`, `:1595`, `:1601`, and `:1609` include the connection, bed, thumbnail,
  and grouping fields involved in the observed discrepancy.
- `Preset.cpp:1490`, within `s_Preset_print_options`, owns
  `ironing_expansion`; it is not a printer-only rule.
- `Preset.cpp:1787-1792` constructs default presets from their complete option
  lists and registered defaults.
- `Preset.cpp:1059-1089`, `Preset::load_external_config`, starts with the
  default preset and overlays project values, excluding connection credentials
  from the project overlay.
- `OrcaSlicer.cpp:3167-3200` calls this construction for retained printer and
  process identities when no replacement settings are selected. The default
  preset is still the construction base when system resolution misses.
- `PrintConfig.hpp:2181-2184` defines the separate static `FullPrintConfig`
  composition; the dynamic-only preset members need not be static members.
- `GCode.cpp:8244-8246,8338-8352` serializes the completed applied dynamic
  snapshot's actual keys, with banned-key and nil filtering.

## Existing Rust ownership and proposed change

`profiles/project_refresh/keys.rs::names` already ports the complete preset
lists through `keys/machine.rs`, `keys/process_first.rs`,
`keys/process_last.rs`, and `keys/filament.rs`. Use these lists, not a new key
set derived from output differences. `options/printer_options.rs` groups typed
fields but does not define Native preset ownership.

`profiles/project_refresh.rs::apply_retained_preset_defaults` already
materializes missing printer/process preset defaults for retained selections
on the profile-overlay path. The direct-project path must also perform this
construction for retained embedded identities before typed settings and the
applied export snapshot are finalized. Reuse the same membership/default
constructor, making the retained-versus-replaced selection explicit rather
than inferring it from the presence of a typed default value.

The direct load integration point is `project/load.rs` and its raw-to-typed
construction; `project_slice/state.rs` must receive the completed raw snapshot.
Keep catalog-aware refresh responsible for resolving system preset values.
Do not overwrite explicit project/user values, revive nil values, normalize a
merged project as a preset, or create absent identities. Apply filament defaults
only through the existing slot-aware filament composition boundary, not a
blanket union of filament keys into project settings.

Leave `options/config_export/full_config.rs` membership filtering unchanged.
Keep the static print-host membership false and canonical wire default
construction intact. Sparse STL profile replacement must retain its current
membership behavior. Existing typed/raw projection is a temporary shell for
Native's applied dynamic configuration.

## Required public comparisons

Both Native modes must pass the unchanged complete-output comparisons:

- `runner::native_jerk_semm_migration::overhang_wall_jerk_matches_native`
- `runner::adjacent_color::processed_layer_templates_preserve_ordered_color_change_items`
- `runner::adjacent_color::sync_delays_survive_color_and_pause_time_flushes`
- `runner::direct_stl_override_bookkeeping::direct_stl_no_override_bookkeeping_control_complete_output`

Preserve assertions, expected values, comparators, key ordering, and explicit
catalog composition. The adjacent-color application adapter correction stays
independent of the retained-project construction correction.
