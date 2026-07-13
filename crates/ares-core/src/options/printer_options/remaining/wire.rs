use serde::{Serialize, Serializer, ser::SerializeMap};

use super::PrinterRemainingOptions;

impl Serialize for PrinterRemainingOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(42))?;
        map.serialize_entry("adaptive_bed_mesh_margin", &self.adaptive_bed_mesh_margin)?;
        map.serialize_entry("bbl_use_printhost", &self.bbl_use_printhost)?;
        map.serialize_entry("bed_custom_model", &self.bed_custom_model)?;
        map.serialize_entry("bed_custom_texture", &self.bed_custom_texture)?;
        map.serialize_entry("bed_exclude_area", &self.bed_exclude_area)?;
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
        map.end()
    }
}
