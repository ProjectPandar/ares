# PrintConfig beta option inventory

Source boundary: `src/libslic3r/PrintConfig.{cpp,hpp}`, old
`8500fcdccaa10b5099ac20d252af3a7c560046f1` to beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`. This is an audit, not a
production port or runtime qualification of every new feature. See
`print-config-beta-defaults-audit.md` for the initial included/deferred boundary.

## Complete common/FFF key delta

- Common/FFF registrations: 675 -> 805; 131 new keys, one removed (`silent_mode`).
- Literal counts include 76 SLA-only declarations in each revision; these are
  unchanged and deferred. Comments and nonliteral prefixes are not new keys.
- Three plugin-loop keys, three belt-remap keys, twelve existing axis-loop keys
  and the nullable filament-override generator are accounted separately from
  literal declarations. The override list grows16 ->19 keys.
- `filament_self_index` adds a static header member but its definition already
  exists in the old revision: it is not a new registry key.
- `PrintConfigConstants.hpp` is byte-identical between the cited revisions.

New definitions below are configuration-owner inventory. Their geometry,
walls/fill, G-code emission and processor consumers are NOT approved by a
metadata declaration. No dormant belt/mixed/plugin/parallel-printing execution
is implicitly accepted. Line numbers refer to the beta `PrintConfig.cpp`.

| New key | Upstream kind | Definition/generator line |
|---|---|---|
| `adaptive_infill_gradient` | `coEnum` | 3775 |
| `adaptive_infill_interior_density` | `coPercent` | 3765 |
| `adaptive_infill_mode` | `coEnum` | 3727 |
| `bed_exclude_volume_mode` | `coEnum` | 1073 |
| `bed_exclude_volumes` | `coString` | 1063 |
| `belt_frame_tilt_angle` | `coFloat` | 7756 |
| `belt_frame_tilt_decouple` | `coBool` | 7745 |
| `belt_printer` | `coBool` | 7697 |
| `belt_printer_infinite_y` | `coBool` | 7706 |
| `belt_purge_tower_object` | `coBool` | 8499 |
| `belt_purge_tower_width` | `coFloat` | 7818 |
| `belt_slice_rotation` | `coEnum` | 7718 |
| `belt_slice_rotation_angle` | `coFloat` | 7732 |
| `belt_support_floor_offset` | `coFloat` | 7797 |
| `bottom_color_penetration_layers` | `coInt` | 1545 |
| `bottom_layer_direction` | `coFloat` | 3681 |
| `bottom_surface_fill_order` | `coEnum` | 2666 |
| `brim_ears_outer_only` | `coBool` | 2206 |
| `build_plate_tilt_x` | `coFloat` | 7672 |
| `build_plate_tilt_y` | `coFloat` | 7685 |
| `center_of_surface_pattern` | `coEnum` | 8188 |
| `cooling_filter_enabled` | `coBool` | 4632 |
| `deretract_speed_extruder_change` | `coFloats` | 6483 |
| `enable_belt_purge_tower` | `coBool` | 7807 |
| `enable_mixed_color_sublayer` | `coBool` | 8244 |
| `enable_pre_heating` | `coBool` | 8837 |
| `exclude_object_tolerance` | `coFloat` | 4707 |
| `extra_brim_width` | `coFloat` | 2099 |
| `extruder_bed_exclude_volumes` | `coStrings` | 1083 |
| `extruder_clearance_dist_to_rod` | `coFloat` | 2815 |
| `extruder_max_nozzle_count` | `coInts` | 8939 |
| `extruder_nozzle_count` | `coInts` | 6352 |
| `extruder_nozzle_stats` | `coStrings` | 6341 |
| `extruder_nozzle_volume_type` | `coEnums` | 6363 |
| `fan_direction` | `coEnum` | 4537 |
| `farthest_point_timelapse` | `coBool` | 6943 |
| `filament_change_length_nc` | `coFloats` | 8861 |
| `filament_dev_ams_drying_ams_limitations` | `coStrings` | 8755 |
| `filament_dev_ams_drying_heat_distortion_temperature` | `coFloats` | 8764 |
| `filament_dev_ams_drying_temperature` | `coFloats` | 8758 |
| `filament_dev_ams_drying_time` | `coFloats` | 8761 |
| `filament_dev_chamber_drying_bed_temperature` | `coFloats` | 8767 |
| `filament_dev_chamber_drying_time` | `coFloats` | 8770 |
| `filament_dev_drying_cooling_temperature` | `coFloats` | 8776 |
| `filament_dev_drying_softening_temperature` | `coFloats` | 8773 |
| `filament_extruder_compatibility` | `coInts` | 3611 |
| `filament_flush_temp_fast` | `coInts` | 3168 |
| `filament_is_mixed` | `coBools` | 3545 |
| `filament_map_2` | `coInts` | 3106 |
| `filament_max_temperature_drop_when_ec` | `coFloats` | 8849 |
| `filament_mixed_components` | `coStrings` | 3551 |
| `filament_mixed_gradient` | `coBools` | 3563 |
| `filament_mixed_gradient_curve` | `coStrings` | 3577 |
| `filament_mixed_gradient_per_part` | `coBools` | 3589 |
| `filament_mixed_gradient_range` | `coStrings` | 3570 |
| `filament_mixed_sublayer_ratios` | `coStrings` | 3557 |
| `filament_nozzle_map` | `coInts` | 3151 |
| `filament_plugin_config_overrides` | `coString` | 1304 |
| `filament_pre_cooling_temperature` | `coInts` | 8879 |
| `filament_pre_cooling_temperature_nc` | `coInts` | 8908 |
| `filament_preheat_temperature_delta` | `coFloats` | 8971 |
| `filament_prime_volume` | `coFloats` | 8955 |
| `filament_prime_volume_nc` | `coFloats` | 8963 |
| `filament_ramming_travel_time` | `coFloats` | 8869 |
| `filament_ramming_travel_time_nc` | `coFloats` | 8898 |
| `filament_ramming_volumetric_speed` | `coFloats` | 8888 |
| `filament_ramming_volumetric_speed_nc` | `coFloats` | 8918 |
| `filament_retract_after_wipe` | `coPercents nullable` | 8785 |
| `filament_retract_length_nc` | `coFloats` | 8928 |
| `filament_retract_length_toolchange` | `coFloats nullable` | 8785 |
| `filament_retract_restart_extra_toolchange` | `coFloats nullable` | 8785 |
| `filament_volume_map` | `coInts` | 3118 |
| `flush_multiplier_fast` | `coFloats` | 8293 |
| `gcode_remap_x` | `coEnum` | 7782 |
| `gcode_remap_y` | `coEnum` | 7785 |
| `gcode_remap_z` | `coEnum` | 7788 |
| `gcode_skip_config_block` | `coBool` | 4672 |
| `group_algo_with_time` | `coBool` | 8829 |
| `handle_hotend_as_extruder` | `coBool` | 8842 |
| `high_precision_gcode` | `coBool` | 4726 |
| `hole_to_polyhole_max_edges` | `coInt` | 8586 |
| `hotend_cooling_rate` | `coFloats` | 8853 |
| `hotend_heating_rate` | `coFloats` | 8857 |
| `imex_carriage_margin` | `coFloat` | 7097 |
| `imex_firmware_managed_zones` | `coBool` | 7035 |
| `imex_gantry_count` | `coInt` | 7046 |
| `imex_head_filament_map` | `coString` | 7127 |
| `imex_mode_active_tools` | `coStrings` | 7141 |
| `imex_mode_gcodes` | `coStrings` | 7147 |
| `imex_mode_names` | `coStrings` | 7135 |
| `imex_nozzle_clearance_x` | `coFloat` | 7081 |
| `imex_nozzle_clearance_y` | `coFloat` | 7089 |
| `imex_parallel_mode` | `coString` | 7121 |
| `imex_tool_layout` | `coEnum` | 7062 |
| `imex_tools_per_gantry` | `coInt` | 7054 |
| `imex_viz_theme` | `coEnum` | 7106 |
| `infill_complete_top` | `coBool` | 4857 |
| `initial_layer_fan_speed` | `coInts` | 4138 |
| `is_imex` | `coBool` | 7025 |
| `leading_brim_length` | `coFloat` | 2077 |
| `machine_bed_mass_Y` | `coFloat` | 5487 |
| `machine_hotend_change_time` | `coFloat` | 8816 |
| `machine_max_force_Y` | `coFloat` | 5477 |
| `machine_max_printed_mass` | `coFloat` | 5497 |
| `machine_prepare_compensation_time` | `coFloat` | 8833 |
| `plugins` | `coStrings` | 6007 |
| `prime_volume_mode` | `coEnum` | 8310 |
| `print_plugin_config_overrides` | `coString` | 1304 |
| `printer_plugin_config_overrides` | `coString` | 1304 |
| `retract_after_wipe` | `coPercents` | 6126 |
| `separated_infills` | `coBool` | 8176 |
| `slicing_pipeline_plugin` | `coStrings` | 6013 |
| `small_support_perimeter_speed` | `coFloatsOrPercents` | 2735 |
| `small_support_perimeter_threshold` | `coFloats` | 2747 |
| `sparse_infill_smooth_factor` | `coPercent` | 3867 |
| `support_cooling_filter` | `coBool` | 4628 |
| `support_fast_purge_mode` | `coBool` | 8947 |
| `toolchange_cyclic_first_layer` | `coBool` | 7263 |
| `toolchange_cyclic_order` | `coString` | 7252 |
| `toolchange_ordering` | `coEnum` | 7236 |
| `top_color_penetration_layers` | `coInt` | 8165 |
| `top_layer_direction` | `coFloat` | 3671 |
| `top_surface_expansion` | `coFloat` | 2586 |
| `top_surface_expansion_direction` | `coEnum` | 2610 |
| `top_surface_expansion_margin` | `coFloat` | 2598 |
| `top_surface_fill_order` | `coEnum` | 2647 |
| `unsupported_wall_last` | `coBool` | 5917 |
| `wait_for_temp_on_wipe_tower` | `coBool` | 7194 |
| `wipe_inward` | `coBool` | 6670 |
| `wipe_inward_distance` | `coFloatOrPercent` | 6682 |
| `wipe_tower_sparse_layers_combination` | `coBool` | 7216 |

## Existing-key default/type candidates

43 scalar declarations become nullable vectors:29 float -> floats,12
float-or-percent -> floats-or-percents, and2 bool -> bools. Numeric constructor
values are preserved; this is a shape/nullability change, not scalar parity.
`enable_arc_fitting` changes Boolean -> `ArcFittingMode` with Disabled default.
The remaining lexical candidate is `nozzle_volume` initializer-brace spelling;
both denote the same one-element0.0 vector, not a numeric-default change.

| Existing key | Old kind | Beta kind | Beta source line |
|---|---|---|---|
| `bridge_acceleration` | `coFloatOrPercent` | `coFloatsOrPercents` | 2339 |
| `bridge_speed` | `coFloat` | `coFloats` | 2043 |
| `default_acceleration` | `coFloat` | `coFloats` | 2309 |
| `default_jerk` | `coFloat` | `coFloats` | 3958 |
| `default_junction_deviation` | `coFloat` | `coFloats` | 3968 |
| `enable_arc_fitting` | `coBool` | `coEnum` | 4426 |
| `enable_overhang_speed` | `coBool` | `coBools` | 1959 |
| `gap_infill_speed` | `coFloat` | `coFloats` | 4405 |
| `infill_jerk` | `coFloat` | `coFloats` | 4009 |
| `initial_layer_acceleration` | `coFloat` | `coFloats` | 3930 |
| `initial_layer_infill_speed` | `coFloat` | `coFloats` | 4086 |
| `initial_layer_jerk` | `coFloat` | `coFloats` | 4019 |
| `initial_layer_speed` | `coFloat` | `coFloats` | 4077 |
| `initial_layer_travel_acceleration` | `coFloatOrPercent` | `coFloatsOrPercents` | 2329 |
| `initial_layer_travel_jerk` | `coFloatOrPercent` | `coFloatsOrPercents` | 4039 |
| `initial_layer_travel_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 4095 |
| `inner_wall_acceleration` | `coFloat` | `coFloats` | 3898 |
| `inner_wall_jerk` | `coFloat` | `coFloats` | 3989 |
| `inner_wall_speed` | `coFloat` | `coFloats` | 5957 |
| `internal_bridge_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 2056 |
| `internal_solid_infill_acceleration` | `coFloatOrPercent` | `coFloatsOrPercents` | 3919 |
| `internal_solid_infill_speed` | `coFloat` | `coFloats` | 6868 |
| `nozzle_volume` | `coFloats` | `coFloats` | 5802 |
| `outer_wall_acceleration` | `coFloat` | `coFloats` | 3888 |
| `outer_wall_jerk` | `coFloat` | `coFloats` | 3979 |
| `outer_wall_speed` | `coFloat` | `coFloats` | 2702 |
| `overhang_1_4_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 1991 |
| `overhang_2_4_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 2004 |
| `overhang_3_4_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 2017 |
| `overhang_4_4_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 2030 |
| `slowdown_for_curled_perimeters` | `coBool` | `coBools` | 1967 |
| `small_perimeter_speed` | `coFloatOrPercent` | `coFloatsOrPercents` | 2712 |
| `small_perimeter_threshold` | `coFloat` | `coFloats` | 2725 |
| `sparse_infill_acceleration` | `coFloatOrPercent` | `coFloatsOrPercents` | 3908 |
| `sparse_infill_speed` | `coFloat` | `coFloats` | 5055 |
| `support_interface_speed` | `coFloat` | `coFloats` | 7533 |
| `support_speed` | `coFloat` | `coFloats` | 7605 |
| `top_solid_infill_flow_ratio` | `coFloat` | `coFloats` | 1744 |
| `top_surface_acceleration` | `coFloat` | `coFloats` | 3878 |
| `top_surface_jerk` | `coFloat` | `coFloats` | 3999 |
| `top_surface_speed` | `coFloat` | `coFloats` | 8137 |
| `travel_acceleration` | `coFloat` | `coFloats` | 2319 |
| `travel_jerk` | `coFloat` | `coFloats` | 4029 |
| `travel_speed` | `coFloat` | `coFloats` | 8207 |
| `travel_speed_z` | `coFloat` | `coFloats` | 8216 |

## Rust ownership

- Definition/default/type/metadata tables:
  `crates/ares-core/src/options/registry/definitions/` and `registry.rs`.
- Exported configuration contract:
  `crates/ares-core/src/options/registry/print_config_def.rs`; the initial RED
  observes only public `ares_core::print_config_def_json()` bytes.
- Key collections: `crates/ares-core/src/options/registry/key_lists/`.
- New vector kinds, nullable decoding and enum semantics:
  `crates/ares-core/src/options/{config_types,decode,json_deserialize}.rs`.
- Typed import and normalization consumers:
  `options/{filament_options,process_options,printer_options,project_fdm_normalization*}`.
  Those consumers need their own source-backed slices, not a scalar fallback.
- Broader PrintConfig algorithm changes and downstream FilamentMixer/wipe tower,
  grouping/preheat, belt geometry, wall/fill and processor execution remain
  deferred. The existing Ares compatibility shells do not define beta policy.

The external audit receipt retains each old/new constructor expression,
header field deltas, source lines, fixture digest and repeated official/seeded
Native default-export identities. Those outputs are not repository goldens.
