use super::*;

#[test]
fn gcode_options_projection_preserves_each_printer_field() {
    assert_printer_projection!(before_layer_change_gcode, OrcaString("printer-before".into()));
    assert_printer_projection!(printing_by_object_gcode, OrcaString("printer-object".into()));
    assert_printer_projection!(machine_end_gcode, OrcaString("printer-end".into()));
    assert_printer_projection!(fan_kickstart, OrcaFloat(9101.01));
    assert_printer_projection!(fan_speedup_overhangs, OrcaBool(false));
    assert_printer_projection!(fan_speedup_time, OrcaFloat(9102.02));
    assert_printer_projection!(part_cooling_fan_min_pwm, OrcaInt(9103));
    assert_printer_projection!(support_object_skip_flush, OrcaBool(true));
    assert_printer_projection!(
        bed_temperature_formula,
        BedTemperatureFormula::FirstFilament
    );
    assert_printer_projection!(
        physical_extruder_map,
        OrcaInts(vec![OrcaInt(9104), OrcaInt(9105)])
    );
    assert_printer_projection!(nozzle_flush_dataset, NullableInts(vec![Nullable::Nil]));
    assert_printer_projection!(scan_first_layer, OrcaBool(true));
    assert_printer_projection!(enable_power_loss_recovery, PowerLossRecoveryMode::Enable);
    assert_printer_projection!(wrapping_detection_layers, OrcaInt(9106));
    assert_printer_projection!(
        wrapping_exclude_area,
        Point2dList(vec![Point2d::new(9107.0, 9108.0)])
    );
    assert_printer_projection!(gcode_flavor, GCodeFlavor::Klipper);
    assert_printer_projection!(time_cost, OrcaFloat(9109.09));
    assert_printer_projection!(layer_change_gcode, OrcaString("printer-layer".into()));
    assert_printer_projection!(time_lapse_gcode, OrcaString("printer-timelapse".into()));
    assert_printer_projection!(
        wrapping_detection_gcode,
        OrcaString("printer-wrapping".into())
    );
    assert_printer_projection!(enable_long_retraction_when_cut, OrcaInt(9110));
    assert_printer_projection!(
        retraction_distances_when_cut,
        OrcaFloats(vec![OrcaFloat(9111.11), OrcaFloat(9112.12)])
    );
    assert_printer_projection!(
        long_retractions_when_cut,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)])
    );
    assert_printer_projection!(z_hop_types, ZHopTypes(vec![ZHopType::Normal]));
    assert_printer_projection!(travel_slope, OrcaFloats(vec![OrcaFloat(9113.13)]));
    assert_printer_projection!(
        retract_lift_enforce,
        RetractLiftEnforces(vec![RetractLiftEnforce::TopOnly])
    );
    assert_printer_projection!(file_start_gcode, OrcaString("printer-file".into()));
    assert_printer_projection!(machine_start_gcode, OrcaString("printer-start".into()));
    assert_printer_projection!(single_extruder_multi_material, OrcaBool(false));
    assert_printer_projection!(manual_filament_change, OrcaBool(true));
    assert_printer_projection!(change_filament_gcode, OrcaString("printer-change".into()));
    assert_printer_projection!(
        change_extrusion_role_gcode,
        OrcaString("printer-role".into())
    );
    assert_printer_projection!(silent_mode, OrcaBool(true));
    assert_printer_projection!(machine_pause_gcode, OrcaString("printer-pause".into()));
    assert_printer_projection!(template_custom_gcode, OrcaString("printer-template".into()));
    assert_printer_projection!(
        nozzle_type,
        NullableNozzleTypes(vec![Nullable::Value(NozzleType::Brass), Nullable::Nil])
    );
    assert_printer_projection!(nozzle_hrc, OrcaInt(9114));
    assert_printer_projection!(auxiliary_fan, OrcaBool(true));
    assert_printer_projection!(support_air_filtration, OrcaBool(false));
    assert_printer_projection!(printer_structure, PrinterStructure::I3);
    assert_printer_projection!(support_chamber_temp_control, OrcaBool(false));
    assert_printer_projection!(extruder_type, ExtruderTypes(vec![ExtruderType::Bowden]));
    assert_printer_projection!(
        printer_extruder_id,
        OrcaInts(vec![OrcaInt(9115), OrcaInt(9116)])
    );
    assert_printer_projection!(master_extruder_id, OrcaInt(9117));
    assert_printer_projection!(
        printer_extruder_variant,
        OrcaStrings(vec!["printer-variant-a".into(), "printer-variant-b".into()])
    );
    assert_printer_projection!(use_firmware_retraction, OrcaBool(true));
    assert_printer_projection!(use_relative_e_distances, OrcaBool(false));
    assert_printer_projection!(disable_m73, OrcaBool(true));
    assert_printer_projection!(cooling_tube_retraction, OrcaFloat(9118.18));
    assert_printer_projection!(cooling_tube_length, OrcaFloat(9119.19));
    assert_printer_projection!(high_current_on_filament_swap, OrcaBool(true));
    assert_printer_projection!(parking_pos_retraction, OrcaFloat(9120.20));
    assert_printer_projection!(extra_loading_move, OrcaFloat(9121.21));
    assert_printer_projection!(machine_load_filament_time, OrcaFloat(9122.22));
    assert_printer_projection!(machine_tool_change_time, OrcaFloat(9123.23));
    assert_printer_projection!(machine_unload_filament_time, OrcaFloat(9124.24));
    assert_printer_projection!(wipe_tower_type, WipeTowerType::Type1);
    assert_printer_projection!(purge_in_prime_tower, OrcaBool(false));
    assert_printer_projection!(enable_filament_ramming, OrcaBool(false));
    assert_printer_projection!(tool_change_on_wipe_tower, OrcaBool(true));
    assert_printer_projection!(support_multi_bed_types, OrcaBool(true));
    assert_printer_projection!(use_3mf, OrcaBool(true));
}
