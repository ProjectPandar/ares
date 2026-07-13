use std::collections::BTreeSet;

use super::super::super::{
    AmsCounts, BedTemperatureFormula, CsvTable, ExtruderTypes, FloatOrPercent, GCodeFlavor,
    GCodeOptions, NozzleVolumeTypes, Nullable, NullableInts, NullableNozzleTypes, OrcaBool,
    OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents, OrcaString, OrcaStrings,
    Percent, Point2dList, PowerLossRecoveryMode, PrinterStructure, ProjectFilamentMapMode,
    RammingParameters, RetractLiftEnforces, SpaceTuple, VariantStride, WipeTowerType, ZHopTypes,
};

#[test]
fn gcode_options_types_are_concrete_for_all_149_fields() {
    fn assert_types(value: Option<&GCodeOptions>) {
        let mut count = 0;
        let mut fields = BTreeSet::new();
        typed!(value, count, fields, OrcaBool;
            accel_to_decel_enable, auxiliary_fan, bbl_calib_mark_logo, disable_m73,
            enable_arc_fitting, enable_filament_ramming, enable_wrapping_detection,
            extrusion_rate_smoothing_external_perimeter_only, fan_speedup_overhangs,
            gcode_add_line_number, has_scarf_joint_seam, high_current_on_filament_swap,
            manual_filament_change, purge_in_prime_tower, scan_first_layer, silent_mode,
            single_extruder_multi_material, single_extruder_multi_material_priming,
            support_air_filtration, support_chamber_temp_control, support_multi_bed_types,
            support_object_skip_flush, tool_change_on_wipe_tower, use_3mf,
            use_firmware_retraction, use_relative_e_distances, wipe_tower_no_sparse_layers
        );
        typed!(value, count, fields, OrcaBools;
            adaptive_pressure_advance, adaptive_pressure_advance_overhangs,
            enable_pressure_advance, filament_is_support, filament_multitool_ramming,
            filament_soluble, long_retractions_when_cut
        );
        typed!(value, count, fields, OrcaFloat;
            cooling_tube_length, cooling_tube_retraction, extra_loading_move, fan_kickstart,
            fan_speedup_time, machine_load_filament_time, machine_tool_change_time,
            machine_unload_filament_time, max_volumetric_extrusion_rate_slope,
            max_volumetric_extrusion_rate_slope_segment_length, parking_pos_retraction,
            time_cost, travel_speed, travel_speed_z
        );
        typed!(value, count, fields, OrcaFloats;
            adaptive_pressure_advance_bridges, deretraction_speed, filament_change_length,
            filament_cooling_final_speed, filament_cooling_initial_speed, filament_cost,
            filament_density, filament_diameter, filament_loading_speed,
            filament_loading_speed_start, filament_max_volumetric_speed,
            filament_minimal_purge_on_wipe_tower, filament_multitool_ramming_flow,
            filament_multitool_ramming_volume, filament_stamping_distance,
            filament_stamping_loading_speed, filament_toolchange_delay,
            filament_tower_interface_pre_extrusion_dist,
            filament_tower_interface_pre_extrusion_length,
            filament_tower_interface_purge_volume, filament_tower_ironing_area,
            filament_unloading_speed, filament_unloading_speed_start, pressure_advance,
            retract_length_toolchange, retract_lift_above, retract_lift_below,
            retract_restart_extra, retract_restart_extra_toolchange,
            retraction_distances_when_cut, retraction_length, retraction_speed, travel_slope,
            z_hop
        );
        typed!(value, count, fields, OrcaInt;
            enable_long_retraction_when_cut, master_extruder_id, nozzle_hrc,
            part_cooling_fan_min_pwm, wrapping_detection_layers
        );
        typed!(value, count, fields, OrcaInts;
            filament_adhesiveness_category, filament_cooling_moves, filament_map,
            filament_printable, filament_tower_interface_print_temp, physical_extruder_map,
            printer_extruder_id, required_nozzle_hrc, temperature_vitrification
        );
        typed!(value, count, fields, OrcaString;
            before_layer_change_gcode, change_extrusion_role_gcode, change_filament_gcode,
            file_start_gcode, layer_change_gcode, machine_end_gcode, machine_pause_gcode,
            machine_start_gcode, printing_by_object_gcode, process_change_extrusion_role_gcode,
            template_custom_gcode, time_lapse_gcode, wrapping_detection_gcode
        );
        typed!(value, count, fields, OrcaStrings;
            default_filament_colour, filament_change_extrusion_role_gcode, filament_colour,
            filament_end_gcode, filament_ids, filament_start_gcode, filament_type,
            filament_vendor, printer_extruder_variant, small_area_infill_flow_compensation_model
        );
        typed!(value, count, fields, Vec<Nullable<OrcaBool>>;
            filament_adaptive_volumetric_speed, long_retractions_when_ec
        );
        typed!(value, count, fields, Vec<Nullable<OrcaFloat>>;
            filament_cooling_before_tower, filament_flow_ratio,
            filament_flush_volumetric_speed, retraction_distances_when_ec
        );
        typed!(value, count, fields, Vec<Nullable<OrcaInt>>; filament_flush_temp);
        typed!(value, count, fields, AmsCounts; extruder_ams_count);
        typed!(value, count, fields, BedTemperatureFormula; bed_temperature_formula);
        typed!(value, count, fields, CsvTable; adaptive_pressure_advance_model);
        typed!(value, count, fields, ExtruderTypes; extruder_type);
        typed!(value, count, fields, FloatOrPercent;
            initial_layer_travel_acceleration, initial_layer_travel_jerk,
            initial_layer_travel_speed
        );
        typed!(value, count, fields, GCodeFlavor; gcode_flavor);
        typed!(value, count, fields, NozzleVolumeTypes; nozzle_volume_type);
        typed!(value, count, fields, NullableInts; nozzle_flush_dataset);
        typed!(value, count, fields, NullableNozzleTypes; nozzle_type);
        typed!(value, count, fields, OrcaPercents; retract_before_wipe);
        typed!(value, count, fields, Percent; accel_to_decel_factor);
        typed!(value, count, fields, Point2dList; wrapping_exclude_area);
        typed!(value, count, fields, PowerLossRecoveryMode; enable_power_loss_recovery);
        typed!(value, count, fields, PrinterStructure; printer_structure);
        typed!(value, count, fields, ProjectFilamentMapMode; filament_map_mode);
        typed!(value, count, fields, RammingParameters; filament_ramming_parameters);
        typed!(value, count, fields, RetractLiftEnforces; retract_lift_enforce);
        typed!(value, count, fields, SpaceTuple; volumetric_speed_coefficients);
        typed!(value, count, fields, VariantStride; filament_extruder_variant);
        typed!(value, count, fields, WipeTowerType; wipe_tower_type);
        typed!(value, count, fields, ZHopTypes; z_hop_types);

        assert_eq!(count, 149);
        assert_eq!(
            fields,
            GCodeOptions::FIELD_METADATA
                .iter()
                .map(|(field, _, _)| *field)
                .collect::<BTreeSet<_>>()
        );
    }

    assert_types(None);
}

macro_rules! typed {
    ($value:ident, $count:ident, $fields:ident, $ty:ty; $($field:ident),+ $(,)?) => {
        $(
            if let Some(value) = $value {
                let _: &$ty = &value.$field;
            }
            $count += 1;
            assert!($fields.insert(stringify!($field)), stringify!($field));
        )+
    };
}

use typed;
