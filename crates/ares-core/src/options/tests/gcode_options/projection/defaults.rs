use super::*;

macro_rules! assert_default_fields {
    ($projected:ident, $source:ident; $($field:ident),+ $(,)?) => {
        $(assert_eq!($projected.$field, $source.$field, stringify!($field));)+
    };
}

#[test]
fn gcode_options_projection_preserves_all_default_source_fields() {
    let printer = PrinterGCodeSourceOptions::default();
    let process = ProcessGCodeSourceOptions::default();
    let filament = FilamentGCodeSourceOptions::default();
    let project = ProjectGCodeSourceOptions::default();
    let projected = GCodeOptions::from_sources(&printer, &process, &filament, &project);

    assert_default_fields!(projected, printer;
        before_layer_change_gcode, printing_by_object_gcode, machine_end_gcode, fan_kickstart,
        fan_speedup_overhangs, fan_speedup_time, part_cooling_fan_min_pwm,
        support_object_skip_flush, bed_temperature_formula, physical_extruder_map,
        nozzle_flush_dataset, scan_first_layer, enable_power_loss_recovery,
        wrapping_detection_layers, wrapping_exclude_area, gcode_flavor, time_cost,
        layer_change_gcode, time_lapse_gcode, wrapping_detection_gcode,
        enable_long_retraction_when_cut, retraction_distances_when_cut,
        long_retractions_when_cut, z_hop_types, travel_slope, retract_lift_enforce,
        file_start_gcode, machine_start_gcode, single_extruder_multi_material,
        manual_filament_change, change_filament_gcode, change_extrusion_role_gcode,
        silent_mode, machine_pause_gcode, template_custom_gcode, nozzle_type, nozzle_hrc,
        auxiliary_fan, support_air_filtration, printer_structure, support_chamber_temp_control,
        extruder_type, printer_extruder_id, master_extruder_id, printer_extruder_variant,
        use_firmware_retraction, use_relative_e_distances, disable_m73,
        cooling_tube_retraction, cooling_tube_length, high_current_on_filament_swap,
        parking_pos_retraction, extra_loading_move, machine_load_filament_time,
        machine_tool_change_time, machine_unload_filament_time, wipe_tower_type,
        purge_in_prime_tower, enable_filament_ramming, tool_change_on_wipe_tower,
        support_multi_bed_types, use_3mf
    );
    assert_default_fields!(projected, process;
        enable_arc_fitting, enable_wrapping_detection, gcode_add_line_number,
        max_volumetric_extrusion_rate_slope, max_volumetric_extrusion_rate_slope_segment_length,
        extrusion_rate_smoothing_external_perimeter_only,
        single_extruder_multi_material_priming, wipe_tower_no_sparse_layers,
        process_change_extrusion_role_gcode, travel_speed, travel_speed_z,
        accel_to_decel_enable, accel_to_decel_factor, initial_layer_travel_speed,
        initial_layer_travel_acceleration, initial_layer_travel_jerk,
        small_area_infill_flow_compensation_model
    );
    assert_default_fields!(projected, filament;
        filament_end_gcode, filament_flow_ratio, enable_pressure_advance, pressure_advance,
        adaptive_pressure_advance, adaptive_pressure_advance_overhangs,
        adaptive_pressure_advance_model, adaptive_pressure_advance_bridges, filament_diameter,
        filament_adaptive_volumetric_speed, volumetric_speed_coefficients,
        filament_adhesiveness_category, filament_density, filament_type, filament_soluble,
        filament_colour, filament_vendor, filament_is_support, filament_printable,
        filament_change_length, filament_cost, default_filament_colour,
        temperature_vitrification, filament_max_volumetric_speed, required_nozzle_hrc,
        filament_extruder_variant, filament_flush_volumetric_speed, filament_flush_temp,
        retraction_distances_when_ec, long_retractions_when_ec, filament_start_gcode,
        filament_change_extrusion_role_gcode, filament_loading_speed,
        filament_loading_speed_start, filament_unloading_speed,
        filament_unloading_speed_start, filament_toolchange_delay, filament_cooling_moves,
        filament_cooling_initial_speed, filament_minimal_purge_on_wipe_tower,
        filament_cooling_before_tower, filament_tower_interface_pre_extrusion_dist,
        filament_tower_interface_pre_extrusion_length, filament_tower_ironing_area,
        filament_tower_interface_purge_volume, filament_tower_interface_print_temp,
        filament_cooling_final_speed, filament_ramming_parameters,
        filament_multitool_ramming, filament_multitool_ramming_volume,
        filament_multitool_ramming_flow, filament_stamping_loading_speed,
        filament_stamping_distance
    );
    assert_default_fields!(projected, project;
        deretraction_speed, filament_ids, filament_map_mode, filament_map,
        retract_before_wipe, retraction_length, retract_length_toolchange, z_hop,
        retract_lift_above, retract_lift_below, retract_restart_extra,
        retract_restart_extra_toolchange, retraction_speed, nozzle_volume_type,
        extruder_ams_count, bbl_calib_mark_logo, has_scarf_joint_seam
    );
}
