use serde::{Serialize, Serializer, ser::SerializeMap};

use super::ProcessPrintSourceOptions;

impl Serialize for ProcessPrintSourceOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(59))?;
        map.serialize_entry("combine_brims", &self.combine_brims)?;
        map.serialize_entry("draft_shield", &self.draft_shield)?;
        map.serialize_entry("enable_prime_tower", &self.enable_prime_tower)?;
        map.serialize_entry(
            "enable_tower_interface_cooldown_during_tower",
            &self.enable_tower_interface_cooldown_during_tower,
        )?;
        map.serialize_entry(
            "enable_tower_interface_features",
            &self.enable_tower_interface_features,
        )?;
        map.serialize_entry("exclude_object", &self.exclude_object)?;
        map.serialize_entry("filename_format", &self.filename_format)?;
        map.serialize_entry("gcode_comments", &self.gcode_comments)?;
        map.serialize_entry("gcode_label_objects", &self.gcode_label_objects)?;
        map.serialize_entry(
            "independent_support_layer_height",
            &self.independent_support_layer_height,
        )?;
        map.serialize_entry(
            "initial_layer_infill_speed",
            &self.initial_layer_infill_speed,
        )?;
        map.serialize_entry("initial_layer_line_width", &self.initial_layer_line_width)?;
        map.serialize_entry(
            "initial_layer_print_height",
            &self.initial_layer_print_height,
        )?;
        map.serialize_entry("initial_layer_speed", &self.initial_layer_speed)?;
        map.serialize_entry(
            "max_travel_detour_distance",
            &self.max_travel_detour_distance,
        )?;
        map.serialize_entry("min_skirt_length", &self.min_skirt_length)?;
        map.serialize_entry("notes", &self.notes)?;
        map.serialize_entry("ooze_prevention", &self.ooze_prevention)?;
        map.serialize_entry("post_process", &self.post_process)?;
        map.serialize_entry("preheat_steps", &self.preheat_steps)?;
        map.serialize_entry("preheat_time", &self.preheat_time)?;
        map.serialize_entry("prime_tower_brim_width", &self.prime_tower_brim_width)?;
        map.serialize_entry(
            "prime_tower_enable_framework",
            &self.prime_tower_enable_framework,
        )?;
        map.serialize_entry("prime_tower_flat_ironing", &self.prime_tower_flat_ironing)?;
        map.serialize_entry("prime_tower_infill_gap", &self.prime_tower_infill_gap)?;
        map.serialize_entry("prime_tower_skip_points", &self.prime_tower_skip_points)?;
        map.serialize_entry("prime_tower_width", &self.prime_tower_width)?;
        map.serialize_entry("prime_volume", &self.prime_volume)?;
        map.serialize_entry("print_order", &self.print_order)?;
        map.serialize_entry("print_sequence", &self.print_sequence)?;
        map.serialize_entry("reduce_crossing_wall", &self.reduce_crossing_wall)?;
        map.serialize_entry("reduce_infill_retraction", &self.reduce_infill_retraction)?;
        map.serialize_entry("resolution", &self.resolution)?;
        map.serialize_entry("single_loop_draft_shield", &self.single_loop_draft_shield)?;
        map.serialize_entry("skirt_distance", &self.skirt_distance)?;
        map.serialize_entry("skirt_height", &self.skirt_height)?;
        map.serialize_entry("skirt_loops", &self.skirt_loops)?;
        map.serialize_entry("skirt_speed", &self.skirt_speed)?;
        map.serialize_entry("skirt_type", &self.skirt_type)?;
        map.serialize_entry("slow_down_layers", &self.slow_down_layers)?;
        map.serialize_entry(
            "spiral_finishing_flow_ratio",
            &self.spiral_finishing_flow_ratio,
        )?;
        map.serialize_entry("spiral_mode", &self.spiral_mode)?;
        map.serialize_entry(
            "spiral_mode_max_xy_smoothing",
            &self.spiral_mode_max_xy_smoothing,
        )?;
        map.serialize_entry("spiral_mode_smooth", &self.spiral_mode_smooth)?;
        map.serialize_entry(
            "spiral_starting_flow_ratio",
            &self.spiral_starting_flow_ratio,
        )?;
        map.serialize_entry("standby_temperature_delta", &self.standby_temperature_delta)?;
        map.serialize_entry("timelapse_type", &self.timelapse_type)?;
        map.serialize_entry("wipe_tower_bridging", &self.wipe_tower_bridging)?;
        map.serialize_entry("wipe_tower_cone_angle", &self.wipe_tower_cone_angle)?;
        map.serialize_entry("wipe_tower_extra_flow", &self.wipe_tower_extra_flow)?;
        map.serialize_entry(
            "wipe_tower_extra_rib_length",
            &self.wipe_tower_extra_rib_length,
        )?;
        map.serialize_entry("wipe_tower_extra_spacing", &self.wipe_tower_extra_spacing)?;
        map.serialize_entry("wipe_tower_filament", &self.wipe_tower_filament)?;
        map.serialize_entry("wipe_tower_fillet_wall", &self.wipe_tower_fillet_wall)?;
        map.serialize_entry(
            "wipe_tower_max_purge_speed",
            &self.wipe_tower_max_purge_speed,
        )?;
        map.serialize_entry("wipe_tower_rib_width", &self.wipe_tower_rib_width)?;
        map.serialize_entry("wipe_tower_rotation_angle", &self.wipe_tower_rotation_angle)?;
        map.serialize_entry("wipe_tower_wall_type", &self.wipe_tower_wall_type)?;
        map.serialize_entry("wiping_volumes_extruders", &self.wiping_volumes_extruders)?;
        map.end()
    }
}
