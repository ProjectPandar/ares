use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProjectGCodeSourceOptions;

impl Serialize for ProjectGCodeSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(17))?;
        map.serialize_entry("bbl_calib_mark_logo", &self.bbl_calib_mark_logo)?;
        map.serialize_entry("deretraction_speed", &self.deretraction_speed)?;
        map.serialize_entry("extruder_ams_count", &self.extruder_ams_count)?;
        map.serialize_entry("filament_ids", &self.filament_ids)?;
        map.serialize_entry("filament_map", &self.filament_map)?;
        map.serialize_entry("filament_map_mode", &self.filament_map_mode)?;
        map.serialize_entry("has_scarf_joint_seam", &self.has_scarf_joint_seam)?;
        map.serialize_entry("nozzle_volume_type", &self.nozzle_volume_type)?;
        map.serialize_entry("retract_before_wipe", &self.retract_before_wipe)?;
        map.serialize_entry("retract_length_toolchange", &self.retract_length_toolchange)?;
        map.serialize_entry("retract_lift_above", &self.retract_lift_above)?;
        map.serialize_entry("retract_lift_below", &self.retract_lift_below)?;
        map.serialize_entry("retract_restart_extra", &self.retract_restart_extra)?;
        map.serialize_entry(
            "retract_restart_extra_toolchange",
            &self.retract_restart_extra_toolchange,
        )?;
        map.serialize_entry("retraction_length", &self.retraction_length)?;
        map.serialize_entry("retraction_speed", &self.retraction_speed)?;
        map.serialize_entry("z_hop", &self.z_hop)?;
        map.end()
    }
}
