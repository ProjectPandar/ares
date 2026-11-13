use serde::{Serialize, Serializer, ser::SerializeMap};

use super::PrinterRemainingOptions;

impl Serialize for PrinterRemainingOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(66))?;
        map.serialize_entry("adaptive_bed_mesh_margin", &self.adaptive_bed_mesh_margin)?;
        map.serialize_entry("bbl_use_printhost", &self.bbl_use_printhost)?;
        map.serialize_entry("bed_custom_model", &self.bed_custom_model)?;
        map.serialize_entry("bed_custom_texture", &self.bed_custom_texture)?;
        map.serialize_entry("bed_exclude_area", &self.bed_exclude_area)?;
        map.serialize_entry("bed_exclude_volume_mode", &self.bed_exclude_volume_mode)?;
        map.serialize_entry("bed_exclude_volumes", &self.bed_exclude_volumes)?;
        map.serialize_entry("bed_mesh_max", &self.bed_mesh_max)?;
        map.serialize_entry("bed_mesh_min", &self.bed_mesh_min)?;
        map.serialize_entry("bed_mesh_probe_distance", &self.bed_mesh_probe_distance)?;
        map.serialize_entry("best_object_pos", &self.best_object_pos)?;
        map.serialize_entry("default_bed_type", &self.default_bed_type)?;
        map.serialize_entry(
            "default_nozzle_volume_type",
            &self.default_nozzle_volume_type,
        )?;
        map.serialize_entry("default_print_profile", &self.default_print_profile)?;
        map.serialize_entry(
            "extruder_bed_exclude_volumes",
            &self.extruder_bed_exclude_volumes,
        )?;
        map.serialize_entry(
            "extruder_clearance_height_to_lid",
            &self.extruder_clearance_height_to_lid,
        )?;
        map.serialize_entry(
            "extruder_clearance_height_to_rod",
            &self.extruder_clearance_height_to_rod,
        )?;
        map.serialize_entry("extruder_clearance_radius", &self.extruder_clearance_radius)?;
        map.serialize_entry("extruder_printable_area", &self.extruder_printable_area)?;
        map.serialize_entry("extruder_printable_height", &self.extruder_printable_height)?;
        map.serialize_entry("extruder_variant_list", &self.extruder_variant_list)?;
        map.serialize_entry("flashforge_serial_number", &self.flashforge_serial_number)?;
        map.serialize_entry("grab_length", &self.grab_length)?;
        map.serialize_entry("head_wrap_detect_zone", &self.head_wrap_detect_zone)?;
        map.serialize_entry("host_type", &self.host_type)?;
        map.serialize_entry("nozzle_height", &self.nozzle_height)?;
        map.serialize_entry("nozzle_volume", &self.nozzle_volume)?;
        map.serialize_entry(
            "parallel_printheads_bed_exclude_areas",
            &self.parallel_printheads_bed_exclude_areas,
        )?;
        map.serialize_entry("parallel_printheads_count", &self.parallel_printheads_count)?;
        map.serialize_entry("pellet_modded_printer", &self.pellet_modded_printer)?;
        map.serialize_entry("preferred_orientation", &self.preferred_orientation)?;
        map.serialize_entry("printable_area", &self.printable_area)?;
        map.serialize_entry("printable_height", &self.printable_height)?;
        map.serialize_entry("printer_agent", &self.printer_agent)?;
        map.serialize_entry("printer_model", &self.printer_model)?;
        map.serialize_entry("printer_notes", &self.printer_notes)?;
        map.serialize_entry("printer_technology", &self.printer_technology)?;
        map.serialize_entry("printer_variant", &self.printer_variant)?;
        map.serialize_entry(
            "printhost_authorization_type",
            &self.printhost_authorization_type,
        )?;
        map.serialize_entry(
            "printhost_ssl_ignore_revoke",
            &self.printhost_ssl_ignore_revoke,
        )?;
        map.serialize_entry(
            "support_parallel_printheads",
            &self.support_parallel_printheads,
        )?;
        map.serialize_entry("thumbnails", &self.thumbnails)?;
        map.serialize_entry("thumbnails_format", &self.thumbnails_format)?;
        map.serialize_entry("upward_compatible_machine", &self.upward_compatible_machine)?;
        map.serialize_entry("z_offset", &self.z_offset)?;
        map.serialize_entry("gcode_skip_config_block", &self.gcode_skip_config_block)?;
        map.serialize_entry("build_plate_tilt_x", &self.build_plate_tilt_x)?;
        map.serialize_entry("build_plate_tilt_y", &self.build_plate_tilt_y)?;
        map.serialize_entry("belt_printer", &self.belt_printer)?;
        map.serialize_entry("belt_printer_infinite_y", &self.belt_printer_infinite_y)?;
        map.serialize_entry("belt_slice_rotation_angle", &self.belt_slice_rotation_angle)?;
        map.serialize_entry("belt_slice_rotation", &self.belt_slice_rotation)?;
        map.serialize_entry("belt_support_floor_offset", &self.belt_support_floor_offset)?;
        map.serialize_entry("gcode_remap_x", &self.gcode_remap_x)?;
        map.serialize_entry("gcode_remap_y", &self.gcode_remap_y)?;
        map.serialize_entry("gcode_remap_z", &self.gcode_remap_z)?;
        map.serialize_entry("belt_frame_tilt_decouple", &self.belt_frame_tilt_decouple)?;
        map.serialize_entry("belt_frame_tilt_angle", &self.belt_frame_tilt_angle)?;
        map.serialize_entry(
            "printer_plugin_config_overrides",
            &self.printer_plugin_config_overrides,
        )?;
        map.serialize_entry(
            "extruder_clearance_dist_to_rod",
            &self.extruder_clearance_dist_to_rod,
        )?;
        map.serialize_entry("fan_direction", &self.fan_direction)?;
        map.serialize_entry("cooling_filter_enabled", &self.cooling_filter_enabled)?;
        map.serialize_entry(
            "wait_for_temp_on_wipe_tower",
            &self.wait_for_temp_on_wipe_tower,
        )?;
        map.serialize_entry("enable_belt_purge_tower", &self.enable_belt_purge_tower)?;
        map.serialize_entry(
            "imex_firmware_managed_zones",
            &self.imex_firmware_managed_zones,
        )?;
        map.serialize_entry("imex_viz_theme", &self.imex_viz_theme)?;
        map.end()
    }
}
