use serde::{Serialize, Serializer, ser::SerializeMap};

use super::PrinterGCodeSourceOptions;

impl Serialize for PrinterGCodeSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(62))?;
        map.serialize_entry("auxiliary_fan", &self.auxiliary_fan)?;
        map.serialize_entry("bed_temperature_formula", &self.bed_temperature_formula)?;
        map.serialize_entry("before_layer_change_gcode", &self.before_layer_change_gcode)?;
        map.serialize_entry(
            "change_extrusion_role_gcode",
            &self.change_extrusion_role_gcode,
        )?;
        map.serialize_entry("change_filament_gcode", &self.change_filament_gcode)?;
        map.serialize_entry("cooling_tube_length", &self.cooling_tube_length)?;
        map.serialize_entry("cooling_tube_retraction", &self.cooling_tube_retraction)?;
        map.serialize_entry("disable_m73", &self.disable_m73)?;
        map.serialize_entry("enable_filament_ramming", &self.enable_filament_ramming)?;
        map.serialize_entry(
            "enable_long_retraction_when_cut",
            &self.enable_long_retraction_when_cut,
        )?;
        map.serialize_entry(
            "enable_power_loss_recovery",
            &self.enable_power_loss_recovery,
        )?;
        map.serialize_entry("extra_loading_move", &self.extra_loading_move)?;
        map.serialize_entry("extruder_type", &self.extruder_type)?;
        map.serialize_entry("fan_kickstart", &self.fan_kickstart)?;
        map.serialize_entry("fan_speedup_overhangs", &self.fan_speedup_overhangs)?;
        map.serialize_entry("fan_speedup_time", &self.fan_speedup_time)?;
        map.serialize_entry("file_start_gcode", &self.file_start_gcode)?;
        map.serialize_entry("gcode_flavor", &self.gcode_flavor)?;
        map.serialize_entry(
            "high_current_on_filament_swap",
            &self.high_current_on_filament_swap,
        )?;
        map.serialize_entry("layer_change_gcode", &self.layer_change_gcode)?;
        map.serialize_entry("long_retractions_when_cut", &self.long_retractions_when_cut)?;
        map.serialize_entry("machine_end_gcode", &self.machine_end_gcode)?;
        map.serialize_entry(
            "machine_load_filament_time",
            &self.machine_load_filament_time,
        )?;
        map.serialize_entry("machine_pause_gcode", &self.machine_pause_gcode)?;
        map.serialize_entry("machine_start_gcode", &self.machine_start_gcode)?;
        map.serialize_entry("machine_tool_change_time", &self.machine_tool_change_time)?;
        map.serialize_entry(
            "machine_unload_filament_time",
            &self.machine_unload_filament_time,
        )?;
        map.serialize_entry("manual_filament_change", &self.manual_filament_change)?;
        map.serialize_entry("master_extruder_id", &self.master_extruder_id)?;
        map.serialize_entry("nozzle_flush_dataset", &self.nozzle_flush_dataset)?;
        map.serialize_entry("nozzle_hrc", &self.nozzle_hrc)?;
        map.serialize_entry("nozzle_type", &self.nozzle_type)?;
        map.serialize_entry("parking_pos_retraction", &self.parking_pos_retraction)?;
        map.serialize_entry("part_cooling_fan_min_pwm", &self.part_cooling_fan_min_pwm)?;
        map.serialize_entry("physical_extruder_map", &self.physical_extruder_map)?;
        map.serialize_entry("printer_extruder_id", &self.printer_extruder_id)?;
        map.serialize_entry("printer_extruder_variant", &self.printer_extruder_variant)?;
        map.serialize_entry("printer_structure", &self.printer_structure)?;
        map.serialize_entry("printing_by_object_gcode", &self.printing_by_object_gcode)?;
        map.serialize_entry("purge_in_prime_tower", &self.purge_in_prime_tower)?;
        map.serialize_entry("retract_lift_enforce", &self.retract_lift_enforce)?;
        map.serialize_entry(
            "retraction_distances_when_cut",
            &self.retraction_distances_when_cut,
        )?;
        map.serialize_entry("scan_first_layer", &self.scan_first_layer)?;
        map.serialize_entry("silent_mode", &self.silent_mode)?;
        map.serialize_entry(
            "single_extruder_multi_material",
            &self.single_extruder_multi_material,
        )?;
        map.serialize_entry("support_air_filtration", &self.support_air_filtration)?;
        map.serialize_entry(
            "support_chamber_temp_control",
            &self.support_chamber_temp_control,
        )?;
        map.serialize_entry("support_multi_bed_types", &self.support_multi_bed_types)?;
        map.serialize_entry("support_object_skip_flush", &self.support_object_skip_flush)?;
        map.serialize_entry("template_custom_gcode", &self.template_custom_gcode)?;
        map.serialize_entry("time_cost", &self.time_cost)?;
        map.serialize_entry("time_lapse_gcode", &self.time_lapse_gcode)?;
        map.serialize_entry("tool_change_on_wipe_tower", &self.tool_change_on_wipe_tower)?;
        map.serialize_entry("travel_slope", &self.travel_slope)?;
        map.serialize_entry("use_3mf", &self.use_3mf)?;
        map.serialize_entry("use_firmware_retraction", &self.use_firmware_retraction)?;
        map.serialize_entry("use_relative_e_distances", &self.use_relative_e_distances)?;
        map.serialize_entry("wipe_tower_type", &self.wipe_tower_type)?;
        map.serialize_entry("wrapping_detection_gcode", &self.wrapping_detection_gcode)?;
        map.serialize_entry("wrapping_detection_layers", &self.wrapping_detection_layers)?;
        map.serialize_entry("wrapping_exclude_area", &self.wrapping_exclude_area)?;
        map.serialize_entry("z_hop_types", &self.z_hop_types)?;
        map.end()
    }
}
