# Type-owned vector nil impact inventory

Current Rust registry: 164 numeric/bool vector candidate keys. This inventory classifies their option families; it does not certify empty input as slice-legal for every key. No coFloatsOrPercents entry in this registry snapshot; typed OrcaFloatOrPercents exists separately.

## Bools

`activate_air_filtration`, `activate_air_filtration_during_print`, `activate_air_filtration_on_completion`, `activate_chamber_temp_control`, `adaptive_pressure_advance`, `adaptive_pressure_advance_overhangs`, `dont_slow_down_outer_wall`, `enable_overhang_bridge_fan`, `enable_pressure_advance`, `filament_is_support`, `filament_multitool_ramming`, `filament_soluble`, `long_retractions_when_cut`, `reduce_fan_stop_start_freq`, `retract_when_changing_layer`, `slow_down_for_layer_cooling`, `wipe`

## BoolsNullable

`filament_adaptive_volumetric_speed`, `filament_long_retractions_when_cut`, `filament_retract_when_changing_layer`, `filament_wipe`, `long_retractions_when_ec`

## Floats

`adaptive_pressure_advance_bridges`, `deretraction_speed`, `extruder_printable_height`, `fan_cooling_layer_time`, `fan_max_speed`, `fan_min_speed`, `filament_change_length`, `filament_cooling_final_speed`, `filament_cooling_initial_speed`, `filament_cost`, `filament_density`, `filament_diameter`, `filament_loading_speed`, `filament_loading_speed_start`, `filament_max_volumetric_speed`, `filament_minimal_purge_on_wipe_tower`, `filament_multitool_ramming_flow`, `filament_multitool_ramming_volume`, `filament_stamping_distance`, `filament_stamping_loading_speed`, `filament_toolchange_delay`, `filament_tower_interface_pre_extrusion_dist`, `filament_tower_interface_pre_extrusion_length`, `filament_tower_interface_purge_volume`, `filament_tower_ironing_area`, `filament_unloading_speed`, `filament_unloading_speed_start`, `first_x_layer_fan_speed`, `flush_multiplier`, `flush_volumes_matrix`, `flush_volumes_vector`, `grab_length`, `machine_max_acceleration_e`, `machine_max_acceleration_extruding`, `machine_max_acceleration_retracting`, `machine_max_acceleration_travel`, `machine_max_acceleration_x`, `machine_max_acceleration_y`, `machine_max_acceleration_z`, `machine_max_jerk_e`, `machine_max_jerk_x`, `machine_max_jerk_y`, `machine_max_jerk_z`, `machine_max_junction_deviation`, `machine_max_speed_e`, `machine_max_speed_x`, `machine_max_speed_y`, `machine_max_speed_z`, `machine_min_extruding_rate`, `machine_min_travel_rate`, `material_correction`, `max_layer_height`, `min_layer_height`, `nozzle_diameter`, `pellet_flow_coefficient`, `pressure_advance`, `relative_correction`, `retract_length_toolchange`, `retract_lift_above`, `retract_lift_below`, `retract_restart_extra`, `retract_restart_extra_toolchange`, `retraction_distances_when_cut`, `retraction_length`, `retraction_minimum_travel`, `retraction_speed`, `slow_down_layer_time`, `slow_down_min_speed`, `travel_slope`, `wipe_distance`, `wipe_tower_x`, `wipe_tower_y`, `wiping_volumes_extruders`, `z_hop`

## FloatsNullable

`filament_cooling_before_tower`, `filament_deretraction_speed`, `filament_flow_ratio`, `filament_flush_volumetric_speed`, `filament_ironing_inset`, `filament_ironing_spacing`, `filament_ironing_speed`, `filament_retract_lift_above`, `filament_retract_lift_below`, `filament_retract_restart_extra`, `filament_retraction_distances_when_cut`, `filament_retraction_length`, `filament_retraction_minimum_travel`, `filament_retraction_speed`, `filament_wipe_distance`, `filament_z_hop`, `nozzle_volume`, `retraction_distances_when_ec`

## Ints

`additional_cooling_fan_speed`, `additional_fan_full_speed_layer`, `chamber_minimal_temperature`, `chamber_temperature`, `close_additional_fan_first_x_layers`, `close_fan_the_first_x_layers`, `complete_print_exhaust_fan_speed`, `cool_plate_temp`, `cool_plate_temp_initial_layer`, `during_print_exhaust_fan_speed`, `eng_plate_temp`, `eng_plate_temp_initial_layer`, `filament_adhesiveness_category`, `filament_cooling_moves`, `filament_map`, `filament_printable`, `filament_self_index`, `filament_tower_interface_print_temp`, `first_layer_print_sequence`, `full_fan_speed_layer`, `hot_plate_temp`, `hot_plate_temp_initial_layer`, `idle_temperature`, `internal_bridge_fan_speed`, `ironing_fan_speed`, `nozzle_temperature`, `nozzle_temperature_initial_layer`, `nozzle_temperature_range_high`, `nozzle_temperature_range_low`, `other_layers_print_sequence`, `overhang_fan_speed`, `physical_extruder_map`, `print_extruder_id`, `printer_extruder_id`, `required_nozzle_HRC`, `supertack_plate_temp`, `supertack_plate_temp_initial_layer`, `support_material_interface_fan_speed`, `temperature_vitrification`, `textured_cool_plate_temp`, `textured_cool_plate_temp_initial_layer`, `textured_plate_temp`, `textured_plate_temp_initial_layer`

## IntsNullable

`filament_flush_temp`, `nozzle_flush_dataset`

## Percents

`filament_shrink`, `filament_shrinkage_compensation_z`, `retract_before_wipe`

## PercentsNullable

`filament_ironing_flow`, `filament_retract_before_wipe`

## Existing raw stream audit

1211 captured Native/Ares stream pairs inspected offline across seeded/official historical G4 full runs and the RED.
Native empty numeric/bool footer lines: {}
Ares empty/all-nil numeric/bool footer lines: {'other_layers_print_sequence': 1}
Observed differing nil lines: 1.

String/point/group empty-vector controls are listed in the current RED input audit and are excluded from numeric/bool nil classification.
