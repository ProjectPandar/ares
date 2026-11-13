# Release settings fields

## Upstream source and Rust boundary

This slice ports configuration declarations, not slicing algorithms, from
OrcaSlicer `40ecc66f7` `src/libslic3r/PrintConfig.hpp` and `PrintConfig.cpp`.
The destination is `ares-core::PrinterOptions`, `ProcessOptions` and
`FilamentOptions`, their existing source groups, and their flat serializers.
`web/app/src/sidebar/processLayout.ts` is refreshed from that revision only
after its option values exist.

The process source boundaries are `PrintObjectConfig` (`PrintConfig.hpp`:
1290–1294), `PrintRegionConfig` (:1473–1612), `GCodeConfig` (:1784), and
`PrintConfig` (:1838–1842, :1983, :2070–2098). Printer fields come from
`GCodeConfig` and `PrintConfig` (:1787–2056), with dynamic preset fields from
`PrintConfigDef::init_common_params` (`PrintConfig.cpp`:1300–1314).
Filament cooling comes from `PrintConfig` (:2056); compatibility and plugin
metadata are dynamic configuration entries (:2213–2245 and :1300–1314).
These typed structures remain partial representations of the named upstream
configurations; this slice does not introduce a new slicing architecture.

## Included fields

All names below are upstream wire names. Defaults and enum spellings are
copied from the corresponding `PrintConfigDef` registrations.

### Process

Twenty-four fields are absent from the typed structures:

| Keys | PrintConfig.cpp definitions | Rust source group |
| --- | --- | --- |
| `enable_mixed_color_sublayer` | 8247–8255 | print_source |
| `hole_to_polyhole_max_edges` | 8589–8601 | region_source |
| `top_surface_fill_order`, `bottom_surface_fill_order` | 2647–2678 | region_source |
| `top_layer_direction`, `bottom_layer_direction` | 3671–3691 | region_source |
| `center_of_surface_pattern`, `separated_infills` | 8179–8209 | region_source |
| `adaptive_infill_mode`, `adaptive_infill_interior_density`, `adaptive_infill_gradient` | 3727–3799 | region_source |
| `small_support_perimeter_speed`, `small_support_perimeter_threshold` | 2735–2759 | region_source |
| `belt_purge_tower_width` | 7821–7832 | print_source |
| `wipe_tower_sparse_layers_combination` | 7218–7237 | print_source |
| `toolchange_ordering`, `toolchange_cyclic_order`, `toolchange_cyclic_first_layer` | 7238–7277 | print_source |
| `leading_brim_length`, `extra_brim_width`, `brim_ears_outer_only` | 2077–2111, 2206–2212 | object_source |
| `high_precision_gcode` | 4726–4736 | gcode_source |
| `slicing_pipeline_plugin` | 6013–6027 | print_source |
| `print_plugin_config_overrides` | 1300–1314 | print_source |

Eight existing region fields are missing only from the top-level process
serializer: `wipe_inward`, `wipe_inward_distance`, `unsupported_wall_last`,
`top_surface_expansion`, `top_surface_expansion_margin`,
`top_surface_expansion_direction`, `sparse_infill_smooth_factor`,
`infill_complete_top`. Their existing types, defaults and slicing consumers
are retained, not reimplemented.

### Filament

Six fields have no typed or project-merged value: `initial_layer_fan_speed`
(:4138–4150), `filament_plugin_config_overrides` (:1300–1314),
`compatible_printers`, `compatible_printers_condition`, `compatible_prints`,
`compatible_prints_condition` (:2213–2245). Cooling belongs to print_source;
the five dynamic preset fields belong to a preset_source group rather than
per-filament numeric vector normalization. Plugin overrides are scalar strings,
not string vectors. Existing project/preset metadata is not duplicated.

### Printer

Twenty-one fields have no typed value:

| Keys | PrintConfig.cpp definitions |
| --- | --- |
| `gcode_skip_config_block` | 4672–4682 |
| `build_plate_tilt_x`, `build_plate_tilt_y` | 7675–7699 |
| `belt_printer`, `belt_printer_infinite_y`, `belt_slice_rotation`, `belt_slice_rotation_angle` | 7700–7747 |
| `belt_frame_tilt_decouple`, `belt_frame_tilt_angle` | 7748–7770 |
| `gcode_remap_x`, `gcode_remap_y`, `gcode_remap_z` | 7772–7793 |
| `belt_support_floor_offset`, `enable_belt_purge_tower` | 7800–7820 |
| `printer_plugin_config_overrides` | 1300–1314 |
| `extruder_clearance_dist_to_rod` | 2815–2826 |
| `fan_direction` | 4537–4554 |
| `cooling_filter_enabled` | 4632–4643 |
| `wait_for_temp_on_wipe_tower` | 7196–7207 |
| `imex_firmware_managed_zones`, `imex_viz_theme` | 7037–7050, 7108–7128 |

They extend PrinterRemainingOptions. `BeltRotationAxis`, `RemapAxis`,
`FanDirection` and `ImexVizTheme` retain their upstream enum tokens.

Six apparent printer wire omissions already have project-merged values:
`nozzle_diameter`, `min_layer_height`, `max_layer_height`, `extruder_offset`,
`retract_length_toolchange`, `retract_restart_extra_toolchange`. They remain
owned by ProjectRuntimeOptions and must not be copied into PrinterOptions.

## Explicitly deferred behavior

For the twenty-four new process fields above, this slice adds storage and
settings display only. It does not implement mixed-color sublayers, polyhole
edge limits, surface ordering/direction/centering, adaptive infill gradients,
separated infill centering, small-support perimeter speed rules, belt towers,
tower layer combination, cyclic tool ordering, belt brims, higher-precision
G-code, or slicing plugins. Those fields can be edited or retained through
configuration APIs without introducing the corresponding slicing behavior.
The eight already implemented region fields listed above keep their existing
behavior; exposing them does not create a new implementation.

All twenty-one new printer fields listed above are definition/typed/display
ports only. Belt slicing, infinite-belt limits, belt support floors, physical
frame tilt and axis remapping, printer plugins, belt purge towers, modified
collision clearance, fan direction/filter behavior, wipe-tower temperature
waits, IMEX zone handling, viewer themes, and configuration-block suppression
are deferred. Existing output must not change as a side effect of field storage.

The six new filament fields add storage/display, not initial-layer cooling
logic, plugin execution, or compatibility-driven preset selection. Plugin
configuration dialogs and the slicing pipeline plugin runtime are deferred.

`TabPrinter::toggle_options` (:6746–6758 in `src/slic3r/GUI/Tab.cpp`) hides
belt details while belt printing is disabled. The web settings layer currently
ports only mode and value-presence visibility, not toggle_options behavior;
therefore that conditional visibility remains deferred rather than introducing
an isolated web-only rule. Printer and filament controls remain read-only.

## Failure modes and behavioral verification

Missing top-level serialization can hide an otherwise typed field. Duplicating
project-owned keys can corrupt the full config projection. Enum names, scalar
versus vector defaults, percent and nullable-vector spelling can differ from
upstream. New typed defaults can accidentally change G-code or insert a dynamic
key that was absent from the applied configuration.

Public option serialization/deserialization tests use source-derived literal
defaults and non-default inputs. Public scene loading checks that every release
layout key has a value in its tab or merged project map. The complete browser
settings scenario checks rendered groups, values and metadata in every mode.
The process table refresh is the final implementation step, not a prerequisite
that exposes empty rows.

Compare the unmodified fixture's complete G-code config block against live
OrcaSlicer in both native execution modes and compare the Ares output bytes
before and after the typed changes. If any G-code byte changes, cease this
slice and report the difference; do not change slicing behavior to hide it.
No native-generated output is stored in the repository.
