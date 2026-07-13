use serde::{Serialize, Serializer, ser::SerializeMap};

use super::PrinterOptions;

impl Serialize for PrinterOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(132))?;
        map.serialize_entry(
            "adaptive_bed_mesh_margin",
            &self.remaining.adaptive_bed_mesh_margin,
        )?;
        map.serialize_entry("auxiliary_fan", &self.gcode.auxiliary_fan)?;
        map.serialize_entry("bbl_use_printhost", &self.remaining.bbl_use_printhost)?;
        map.serialize_entry("bed_custom_model", &self.remaining.bed_custom_model)?;
        map.serialize_entry("bed_custom_texture", &self.remaining.bed_custom_texture)?;
        map.serialize_entry("bed_exclude_area", &self.remaining.bed_exclude_area)?;
        map.serialize_entry("bed_mesh_max", &self.remaining.bed_mesh_max)?;
        map.serialize_entry("bed_mesh_min", &self.remaining.bed_mesh_min)?;
        map.serialize_entry(
            "bed_mesh_probe_distance",
            &self.remaining.bed_mesh_probe_distance,
        )?;
        map.serialize_entry(
            "bed_temperature_formula",
            &self.gcode.bed_temperature_formula,
        )?;
        map.serialize_entry(
            "before_layer_change_gcode",
            &self.gcode.before_layer_change_gcode,
        )?;
        map.serialize_entry("best_object_pos", &self.remaining.best_object_pos)?;
        map.serialize_entry(
            "change_extrusion_role_gcode",
            &self.gcode.change_extrusion_role_gcode,
        )?;
        map.serialize_entry("change_filament_gcode", &self.gcode.change_filament_gcode)?;
        map.serialize_entry("cooling_tube_length", &self.gcode.cooling_tube_length)?;
        map.serialize_entry(
            "cooling_tube_retraction",
            &self.gcode.cooling_tube_retraction,
        )?;
        map.serialize_entry("default_bed_type", &self.remaining.default_bed_type)?;
        map.serialize_entry(
            "default_nozzle_volume_type",
            &self.remaining.default_nozzle_volume_type,
        )?;
        map.serialize_entry(
            "default_print_profile",
            &self.remaining.default_print_profile,
        )?;
        map.serialize_entry("disable_m73", &self.gcode.disable_m73)?;
        map.serialize_entry(
            "emit_machine_limits_to_gcode",
            &self.machine.emit_machine_limits_to_gcode,
        )?;
        map.serialize_entry(
            "enable_filament_ramming",
            &self.gcode.enable_filament_ramming,
        )?;
        map.serialize_entry(
            "enable_long_retraction_when_cut",
            &self.gcode.enable_long_retraction_when_cut,
        )?;
        map.serialize_entry(
            "enable_power_loss_recovery",
            &self.gcode.enable_power_loss_recovery,
        )?;
        map.serialize_entry("extra_loading_move", &self.gcode.extra_loading_move)?;
        map.serialize_entry(
            "extruder_clearance_height_to_lid",
            &self.remaining.extruder_clearance_height_to_lid,
        )?;
        map.serialize_entry(
            "extruder_clearance_height_to_rod",
            &self.remaining.extruder_clearance_height_to_rod,
        )?;
        map.serialize_entry(
            "extruder_clearance_radius",
            &self.remaining.extruder_clearance_radius,
        )?;
        map.serialize_entry(
            "extruder_printable_area",
            &self.remaining.extruder_printable_area,
        )?;
        map.serialize_entry(
            "extruder_printable_height",
            &self.remaining.extruder_printable_height,
        )?;
        map.serialize_entry("extruder_type", &self.gcode.extruder_type)?;
        map.serialize_entry(
            "extruder_variant_list",
            &self.remaining.extruder_variant_list,
        )?;
        map.serialize_entry("fan_kickstart", &self.gcode.fan_kickstart)?;
        map.serialize_entry("fan_speedup_overhangs", &self.gcode.fan_speedup_overhangs)?;
        map.serialize_entry("fan_speedup_time", &self.gcode.fan_speedup_time)?;
        map.serialize_entry("file_start_gcode", &self.gcode.file_start_gcode)?;
        map.serialize_entry(
            "flashforge_serial_number",
            &self.remaining.flashforge_serial_number,
        )?;
        map.serialize_entry("gcode_flavor", &self.gcode.gcode_flavor)?;
        map.serialize_entry("grab_length", &self.remaining.grab_length)?;
        map.serialize_entry(
            "head_wrap_detect_zone",
            &self.remaining.head_wrap_detect_zone,
        )?;
        map.serialize_entry(
            "high_current_on_filament_swap",
            &self.gcode.high_current_on_filament_swap,
        )?;
        map.serialize_entry("host_type", &self.remaining.host_type)?;
        map.serialize_entry("input_shaping_damp_x", &self.machine.input_shaping_damp_x)?;
        map.serialize_entry("input_shaping_damp_y", &self.machine.input_shaping_damp_y)?;
        map.serialize_entry("input_shaping_emit", &self.machine.input_shaping_emit)?;
        map.serialize_entry("input_shaping_freq_x", &self.machine.input_shaping_freq_x)?;
        map.serialize_entry("input_shaping_freq_y", &self.machine.input_shaping_freq_y)?;
        map.serialize_entry("input_shaping_type", &self.machine.input_shaping_type)?;
        map.serialize_entry("layer_change_gcode", &self.gcode.layer_change_gcode)?;
        map.serialize_entry(
            "long_retractions_when_cut",
            &self.gcode.long_retractions_when_cut,
        )?;
        map.serialize_entry("machine_end_gcode", &self.gcode.machine_end_gcode)?;
        map.serialize_entry(
            "machine_load_filament_time",
            &self.gcode.machine_load_filament_time,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_e",
            &self.machine.machine_max_acceleration_e,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_extruding",
            &self.machine.machine_max_acceleration_extruding,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_retracting",
            &self.machine.machine_max_acceleration_retracting,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_travel",
            &self.machine.machine_max_acceleration_travel,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_x",
            &self.machine.machine_max_acceleration_x,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_y",
            &self.machine.machine_max_acceleration_y,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_z",
            &self.machine.machine_max_acceleration_z,
        )?;
        map.serialize_entry("machine_max_jerk_e", &self.machine.machine_max_jerk_e)?;
        map.serialize_entry("machine_max_jerk_x", &self.machine.machine_max_jerk_x)?;
        map.serialize_entry("machine_max_jerk_y", &self.machine.machine_max_jerk_y)?;
        map.serialize_entry("machine_max_jerk_z", &self.machine.machine_max_jerk_z)?;
        map.serialize_entry(
            "machine_max_junction_deviation",
            &self.machine.machine_max_junction_deviation,
        )?;
        map.serialize_entry("machine_max_speed_e", &self.machine.machine_max_speed_e)?;
        map.serialize_entry("machine_max_speed_x", &self.machine.machine_max_speed_x)?;
        map.serialize_entry("machine_max_speed_y", &self.machine.machine_max_speed_y)?;
        map.serialize_entry("machine_max_speed_z", &self.machine.machine_max_speed_z)?;
        map.serialize_entry(
            "machine_min_extruding_rate",
            &self.machine.machine_min_extruding_rate,
        )?;
        map.serialize_entry(
            "machine_min_travel_rate",
            &self.machine.machine_min_travel_rate,
        )?;
        map.serialize_entry("machine_pause_gcode", &self.gcode.machine_pause_gcode)?;
        map.serialize_entry("machine_start_gcode", &self.gcode.machine_start_gcode)?;
        map.serialize_entry(
            "machine_tool_change_time",
            &self.gcode.machine_tool_change_time,
        )?;
        map.serialize_entry(
            "machine_unload_filament_time",
            &self.gcode.machine_unload_filament_time,
        )?;
        map.serialize_entry("manual_filament_change", &self.gcode.manual_filament_change)?;
        map.serialize_entry("master_extruder_id", &self.gcode.master_extruder_id)?;
        map.serialize_entry(
            "max_resonance_avoidance_speed",
            &self.machine.max_resonance_avoidance_speed,
        )?;
        map.serialize_entry(
            "min_resonance_avoidance_speed",
            &self.machine.min_resonance_avoidance_speed,
        )?;
        map.serialize_entry("nozzle_flush_dataset", &self.gcode.nozzle_flush_dataset)?;
        map.serialize_entry("nozzle_height", &self.remaining.nozzle_height)?;
        map.serialize_entry("nozzle_hrc", &self.gcode.nozzle_hrc)?;
        map.serialize_entry("nozzle_type", &self.gcode.nozzle_type)?;
        map.serialize_entry("nozzle_volume", &self.remaining.nozzle_volume)?;
        map.serialize_entry(
            "parallel_printheads_bed_exclude_areas",
            &self.remaining.parallel_printheads_bed_exclude_areas,
        )?;
        map.serialize_entry(
            "parallel_printheads_count",
            &self.remaining.parallel_printheads_count,
        )?;
        map.serialize_entry("parking_pos_retraction", &self.gcode.parking_pos_retraction)?;
        map.serialize_entry(
            "part_cooling_fan_min_pwm",
            &self.gcode.part_cooling_fan_min_pwm,
        )?;
        map.serialize_entry(
            "pellet_modded_printer",
            &self.remaining.pellet_modded_printer,
        )?;
        map.serialize_entry("physical_extruder_map", &self.gcode.physical_extruder_map)?;
        map.serialize_entry(
            "preferred_orientation",
            &self.remaining.preferred_orientation,
        )?;
        map.serialize_entry("printable_area", &self.remaining.printable_area)?;
        map.serialize_entry("printable_height", &self.remaining.printable_height)?;
        map.serialize_entry("printer_agent", &self.remaining.printer_agent)?;
        map.serialize_entry("printer_extruder_id", &self.gcode.printer_extruder_id)?;
        map.serialize_entry(
            "printer_extruder_variant",
            &self.gcode.printer_extruder_variant,
        )?;
        map.serialize_entry("printer_model", &self.remaining.printer_model)?;
        map.serialize_entry("printer_notes", &self.remaining.printer_notes)?;
        map.serialize_entry("printer_structure", &self.gcode.printer_structure)?;
        map.serialize_entry("printer_technology", &self.remaining.printer_technology)?;
        map.serialize_entry("printer_variant", &self.remaining.printer_variant)?;
        map.serialize_entry(
            "printhost_authorization_type",
            &self.remaining.printhost_authorization_type,
        )?;
        map.serialize_entry(
            "printhost_ssl_ignore_revoke",
            &self.remaining.printhost_ssl_ignore_revoke,
        )?;
        map.serialize_entry(
            "printing_by_object_gcode",
            &self.gcode.printing_by_object_gcode,
        )?;
        map.serialize_entry("purge_in_prime_tower", &self.gcode.purge_in_prime_tower)?;
        map.serialize_entry("resonance_avoidance", &self.machine.resonance_avoidance)?;
        map.serialize_entry("retract_lift_enforce", &self.gcode.retract_lift_enforce)?;
        map.serialize_entry(
            "retraction_distances_when_cut",
            &self.gcode.retraction_distances_when_cut,
        )?;
        map.serialize_entry("scan_first_layer", &self.gcode.scan_first_layer)?;
        map.serialize_entry("silent_mode", &self.gcode.silent_mode)?;
        map.serialize_entry(
            "single_extruder_multi_material",
            &self.gcode.single_extruder_multi_material,
        )?;
        map.serialize_entry("support_air_filtration", &self.gcode.support_air_filtration)?;
        map.serialize_entry(
            "support_chamber_temp_control",
            &self.gcode.support_chamber_temp_control,
        )?;
        map.serialize_entry(
            "support_multi_bed_types",
            &self.gcode.support_multi_bed_types,
        )?;
        map.serialize_entry(
            "support_object_skip_flush",
            &self.gcode.support_object_skip_flush,
        )?;
        map.serialize_entry(
            "support_parallel_printheads",
            &self.remaining.support_parallel_printheads,
        )?;
        map.serialize_entry("template_custom_gcode", &self.gcode.template_custom_gcode)?;
        map.serialize_entry("thumbnails", &self.remaining.thumbnails)?;
        map.serialize_entry("thumbnails_format", &self.remaining.thumbnails_format)?;
        map.serialize_entry("time_cost", &self.gcode.time_cost)?;
        map.serialize_entry("time_lapse_gcode", &self.gcode.time_lapse_gcode)?;
        map.serialize_entry(
            "tool_change_on_wipe_tower",
            &self.gcode.tool_change_on_wipe_tower,
        )?;
        map.serialize_entry("travel_slope", &self.gcode.travel_slope)?;
        map.serialize_entry(
            "upward_compatible_machine",
            &self.remaining.upward_compatible_machine,
        )?;
        map.serialize_entry("use_3mf", &self.gcode.use_3mf)?;
        map.serialize_entry(
            "use_firmware_retraction",
            &self.gcode.use_firmware_retraction,
        )?;
        map.serialize_entry(
            "use_relative_e_distances",
            &self.gcode.use_relative_e_distances,
        )?;
        map.serialize_entry("wipe_tower_type", &self.gcode.wipe_tower_type)?;
        map.serialize_entry(
            "wrapping_detection_gcode",
            &self.gcode.wrapping_detection_gcode,
        )?;
        map.serialize_entry(
            "wrapping_detection_layers",
            &self.gcode.wrapping_detection_layers,
        )?;
        map.serialize_entry("wrapping_exclude_area", &self.gcode.wrapping_exclude_area)?;
        map.serialize_entry("z_hop_types", &self.gcode.z_hop_types)?;
        map.serialize_entry("z_offset", &self.remaining.z_offset)?;
        map.end()
    }
}
