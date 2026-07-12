use super::super::super::{
    BedTemperatureFormula, ExtruderTypes, GCodeFlavor, NullableInts, NullableNozzleTypes,
    OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts, OrcaString, OrcaStrings,
    Point2dList, PowerLossRecoveryMode, PrinterGCodeSourceOptions, PrinterStructure,
    RetractLiftEnforces, WipeTowerType, ZHopTypes,
};

pub(super) fn assert_concrete_types(value: &PrinterGCodeSourceOptions) {
    let _: &OrcaBool = &value.auxiliary_fan;
    let _: &BedTemperatureFormula = &value.bed_temperature_formula;
    let _: &OrcaString = &value.before_layer_change_gcode;
    let _: &OrcaString = &value.change_extrusion_role_gcode;
    let _: &OrcaString = &value.change_filament_gcode;
    let _: &OrcaFloat = &value.cooling_tube_length;
    let _: &OrcaFloat = &value.cooling_tube_retraction;
    let _: &OrcaBool = &value.disable_m73;
    let _: &OrcaBool = &value.enable_filament_ramming;
    let _: &OrcaInt = &value.enable_long_retraction_when_cut;
    let _: &PowerLossRecoveryMode = &value.enable_power_loss_recovery;
    let _: &OrcaFloat = &value.extra_loading_move;
    let _: &ExtruderTypes = &value.extruder_type;
    let _: &OrcaFloat = &value.fan_kickstart;
    let _: &OrcaBool = &value.fan_speedup_overhangs;
    let _: &OrcaFloat = &value.fan_speedup_time;
    let _: &OrcaString = &value.file_start_gcode;
    let _: &GCodeFlavor = &value.gcode_flavor;
    let _: &OrcaBool = &value.high_current_on_filament_swap;
    let _: &OrcaString = &value.layer_change_gcode;
    let _: &OrcaBools = &value.long_retractions_when_cut;
    let _: &OrcaString = &value.machine_end_gcode;
    let _: &OrcaFloat = &value.machine_load_filament_time;
    let _: &OrcaString = &value.machine_pause_gcode;
    let _: &OrcaString = &value.machine_start_gcode;
    let _: &OrcaFloat = &value.machine_tool_change_time;
    let _: &OrcaFloat = &value.machine_unload_filament_time;
    let _: &OrcaBool = &value.manual_filament_change;
    let _: &OrcaInt = &value.master_extruder_id;
    let _: &NullableInts = &value.nozzle_flush_dataset;
    let _: &OrcaInt = &value.nozzle_hrc;
    let _: &NullableNozzleTypes = &value.nozzle_type;
    let _: &OrcaFloat = &value.parking_pos_retraction;
    let _: &OrcaInt = &value.part_cooling_fan_min_pwm;
    let _: &OrcaInts = &value.physical_extruder_map;
    let _: &OrcaInts = &value.printer_extruder_id;
    let _: &OrcaStrings = &value.printer_extruder_variant;
    let _: &PrinterStructure = &value.printer_structure;
    let _: &OrcaString = &value.printing_by_object_gcode;
    let _: &OrcaBool = &value.purge_in_prime_tower;
    let _: &RetractLiftEnforces = &value.retract_lift_enforce;
    let _: &OrcaFloats = &value.retraction_distances_when_cut;
    let _: &OrcaBool = &value.scan_first_layer;
    let _: &OrcaBool = &value.silent_mode;
    let _: &OrcaBool = &value.single_extruder_multi_material;
    let _: &OrcaBool = &value.support_air_filtration;
    let _: &OrcaBool = &value.support_chamber_temp_control;
    let _: &OrcaBool = &value.support_multi_bed_types;
    let _: &OrcaBool = &value.support_object_skip_flush;
    let _: &OrcaString = &value.template_custom_gcode;
    let _: &OrcaFloat = &value.time_cost;
    let _: &OrcaString = &value.time_lapse_gcode;
    let _: &OrcaBool = &value.tool_change_on_wipe_tower;
    let _: &OrcaFloats = &value.travel_slope;
    let _: &OrcaBool = &value.use_3mf;
    let _: &OrcaBool = &value.use_firmware_retraction;
    let _: &OrcaBool = &value.use_relative_e_distances;
    let _: &WipeTowerType = &value.wipe_tower_type;
    let _: &OrcaString = &value.wrapping_detection_gcode;
    let _: &OrcaInt = &value.wrapping_detection_layers;
    let _: &Point2dList = &value.wrapping_exclude_area;
    let _: &ZHopTypes = &value.z_hop_types;
}
