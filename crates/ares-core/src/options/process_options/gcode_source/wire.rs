use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProcessGCodeSourceOptions;

impl Serialize for ProcessGCodeSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(17))?;
        map.serialize_entry("accel_to_decel_enable", &self.accel_to_decel_enable)?;
        map.serialize_entry("accel_to_decel_factor", &self.accel_to_decel_factor)?;
        map.serialize_entry("enable_arc_fitting", &self.enable_arc_fitting)?;
        map.serialize_entry("enable_wrapping_detection", &self.enable_wrapping_detection)?;
        map.serialize_entry(
            "extrusion_rate_smoothing_external_perimeter_only",
            &self.extrusion_rate_smoothing_external_perimeter_only,
        )?;
        map.serialize_entry("gcode_add_line_number", &self.gcode_add_line_number)?;
        map.serialize_entry(
            "initial_layer_travel_acceleration",
            &self.initial_layer_travel_acceleration,
        )?;
        map.serialize_entry("initial_layer_travel_jerk", &self.initial_layer_travel_jerk)?;
        map.serialize_entry(
            "initial_layer_travel_speed",
            &self.initial_layer_travel_speed,
        )?;
        map.serialize_entry(
            "max_volumetric_extrusion_rate_slope",
            &self.max_volumetric_extrusion_rate_slope,
        )?;
        map.serialize_entry(
            "max_volumetric_extrusion_rate_slope_segment_length",
            &self.max_volumetric_extrusion_rate_slope_segment_length,
        )?;
        map.serialize_entry(
            "process_change_extrusion_role_gcode",
            &self.process_change_extrusion_role_gcode,
        )?;
        map.serialize_entry(
            "single_extruder_multi_material_priming",
            &self.single_extruder_multi_material_priming,
        )?;
        map.serialize_entry(
            "small_area_infill_flow_compensation_model",
            &self.small_area_infill_flow_compensation_model,
        )?;
        map.serialize_entry("travel_speed", &self.travel_speed)?;
        map.serialize_entry("travel_speed_z", &self.travel_speed_z)?;
        map.serialize_entry(
            "wipe_tower_no_sparse_layers",
            &self.wipe_tower_no_sparse_layers,
        )?;
        map.end()
    }
}
