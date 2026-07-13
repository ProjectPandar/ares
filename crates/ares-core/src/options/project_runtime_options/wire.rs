use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProjectRuntimeOptions;

impl Serialize for ProjectRuntimeOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Self {
            gcode,
            print,
            preset,
        } = self;
        let mut map = serializer.serialize_map(Some(44))?;
        map.serialize_entry("bbl_calib_mark_logo", &gcode.bbl_calib_mark_logo)?;
        map.serialize_entry("curr_bed_type", &print.curr_bed_type)?;
        map.serialize_entry("default_filament_profile", &preset.default_filament_profile)?;
        map.serialize_entry("deretraction_speed", &gcode.deretraction_speed)?;
        map.serialize_entry("extruder_ams_count", &gcode.extruder_ams_count)?;
        map.serialize_entry("extruder_colour", &print.extruder_colour)?;
        map.serialize_entry("extruder_offset", &print.extruder_offset)?;
        map.serialize_entry("filament_colour_type", &preset.filament_colour_type)?;
        map.serialize_entry("filament_ids", &gcode.filament_ids)?;
        map.serialize_entry("filament_map", &gcode.filament_map)?;
        map.serialize_entry("filament_map_mode", &gcode.filament_map_mode)?;
        map.serialize_entry("filament_multi_colour", &preset.filament_multi_colour)?;
        map.serialize_entry("filament_self_index", &preset.filament_self_index)?;
        map.serialize_entry("filament_settings_id", &preset.filament_settings_id)?;
        map.serialize_entry(
            "first_layer_print_sequence",
            &print.first_layer_print_sequence,
        )?;
        map.serialize_entry("flush_multiplier", &print.flush_multiplier)?;
        map.serialize_entry("flush_volumes_matrix", &print.flush_volumes_matrix)?;
        map.serialize_entry("flush_volumes_vector", &print.flush_volumes_vector)?;
        map.serialize_entry("has_scarf_joint_seam", &gcode.has_scarf_joint_seam)?;
        map.serialize_entry("max_layer_height", &print.max_layer_height)?;
        map.serialize_entry("min_layer_height", &print.min_layer_height)?;
        map.serialize_entry("nozzle_diameter", &print.nozzle_diameter)?;
        map.serialize_entry("nozzle_volume_type", &gcode.nozzle_volume_type)?;
        map.serialize_entry(
            "other_layers_print_sequence",
            &print.other_layers_print_sequence,
        )?;
        map.serialize_entry(
            "other_layers_print_sequence_nums",
            &print.other_layers_print_sequence_nums,
        )?;
        map.serialize_entry(
            "print_compatible_printers",
            &preset.print_compatible_printers,
        )?;
        map.serialize_entry("print_settings_id", &preset.print_settings_id)?;
        map.serialize_entry("printer_settings_id", &preset.printer_settings_id)?;
        map.serialize_entry("retract_before_wipe", &gcode.retract_before_wipe)?;
        map.serialize_entry(
            "retract_length_toolchange",
            &gcode.retract_length_toolchange,
        )?;
        map.serialize_entry("retract_lift_above", &gcode.retract_lift_above)?;
        map.serialize_entry("retract_lift_below", &gcode.retract_lift_below)?;
        map.serialize_entry("retract_restart_extra", &gcode.retract_restart_extra)?;
        map.serialize_entry(
            "retract_restart_extra_toolchange",
            &gcode.retract_restart_extra_toolchange,
        )?;
        map.serialize_entry(
            "retract_when_changing_layer",
            &print.retract_when_changing_layer,
        )?;
        map.serialize_entry("retraction_length", &gcode.retraction_length)?;
        map.serialize_entry(
            "retraction_minimum_travel",
            &print.retraction_minimum_travel,
        )?;
        map.serialize_entry("retraction_speed", &gcode.retraction_speed)?;
        map.serialize_entry("start_end_points", &print.start_end_points)?;
        map.serialize_entry("wipe", &print.wipe)?;
        map.serialize_entry("wipe_distance", &print.wipe_distance)?;
        map.serialize_entry("wipe_tower_x", &print.wipe_tower_x)?;
        map.serialize_entry("wipe_tower_y", &print.wipe_tower_y)?;
        map.serialize_entry("z_hop", &gcode.z_hop)?;
        map.end()
    }
}
