mod serialization;

use crate::SliceError;

use super::SliceOptions;

use serialization::{
    optional_bool_vector_export, optional_filament_cooling_before_tower_export,
    optional_float_vector_export, optional_int_vector_export, optional_int_vector_export_in_range,
    optional_non_negative_scalar_float_export, optional_scalar_bool_export,
    optional_scalar_float_export, optional_scalar_float_export_with_bounds,
    optional_scalar_int_export_in_range, optional_small_area_flow_model_export,
    optional_string_vector_export, optional_wipe_tower_coordinate_export,
    optional_wipe_tower_type_export, optional_wipe_tower_wall_type_export,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FilamentConfigExports {
    pub(crate) filament_adhesiveness_category: Option<String>,
    pub(crate) cooling_tube_retraction: Option<String>,
    pub(crate) cooling_tube_length: Option<String>,
    pub(crate) high_current_on_filament_swap: Option<String>,
    pub(crate) parking_pos_retraction: Option<String>,
    pub(crate) extra_loading_move: Option<String>,
    pub(crate) machine_load_filament_time: Option<String>,
    pub(crate) machine_tool_change_time: Option<String>,
    pub(crate) machine_unload_filament_time: Option<String>,
    pub(crate) filament_loading_speed: Option<String>,
    pub(crate) filament_loading_speed_start: Option<String>,
    pub(crate) filament_unloading_speed: Option<String>,
    pub(crate) filament_unloading_speed_start: Option<String>,
    pub(crate) filament_toolchange_delay: Option<String>,
    pub(crate) filament_cooling_moves: Option<String>,
    pub(crate) filament_cooling_initial_speed: Option<String>,
    pub(crate) filament_minimal_purge_on_wipe_tower: Option<String>,
    pub(crate) filament_cooling_before_tower: Option<String>,
    pub(crate) filament_tower_interface_pre_extrusion_dist: Option<String>,
    pub(crate) filament_tower_interface_pre_extrusion_length: Option<String>,
    pub(crate) filament_tower_ironing_area: Option<String>,
    pub(crate) filament_tower_interface_purge_volume: Option<String>,
    pub(crate) filament_tower_interface_print_temp: Option<String>,
    pub(crate) filament_cooling_final_speed: Option<String>,
    pub(crate) filament_ramming_parameters: Option<String>,
    pub(crate) filament_multitool_ramming: Option<String>,
    pub(crate) filament_multitool_ramming_volume: Option<String>,
    pub(crate) filament_multitool_ramming_flow: Option<String>,
    pub(crate) filament_stamping_loading_speed: Option<String>,
    pub(crate) filament_stamping_distance: Option<String>,
    pub(crate) wipe_tower_type: Option<String>,
    pub(crate) purge_in_prime_tower: Option<String>,
    pub(crate) enable_filament_ramming: Option<String>,
    pub(crate) tool_change_on_wipe_tower: Option<String>,
    pub(crate) wipe_tower_no_sparse_layers: Option<String>,
    pub(crate) support_multi_bed_types: Option<String>,
    pub(crate) wipe_tower_x: Option<String>,
    pub(crate) wipe_tower_y: Option<String>,
    pub(crate) prime_tower_width: Option<String>,
    pub(crate) wipe_tower_rotation_angle: Option<String>,
    pub(crate) prime_tower_brim_width: Option<String>,
    pub(crate) wipe_tower_bridging: Option<String>,
    pub(crate) wipe_tower_extra_flow: Option<String>,
    pub(crate) wipe_tower_cone_angle: Option<String>,
    pub(crate) wipe_tower_extra_spacing: Option<String>,
    pub(crate) wipe_tower_max_purge_speed: Option<String>,
    pub(crate) wipe_tower_wall_type: Option<String>,
    pub(crate) wipe_tower_extra_rib_length: Option<String>,
    pub(crate) wipe_tower_rib_width: Option<String>,
    pub(crate) wipe_tower_fillet_wall: Option<String>,
    pub(crate) wipe_tower_filament: Option<String>,
    pub(crate) small_area_infill_flow_compensation_model: Option<String>,
    pub(crate) filament_colour: Option<String>,
    pub(crate) extruder_colour: Option<String>,
    pub(crate) filament_multi_colour: Option<String>,
    pub(crate) filament_colour_new: Option<String>,
    pub(crate) filament_colour_type: Option<String>,
    pub(crate) default_filament_colour: Option<String>,
    pub(crate) filament_ids: Option<String>,
    pub(crate) filament_soluble: Option<String>,
    pub(crate) filament_is_support: Option<String>,
    pub(crate) filament_printable: Option<String>,
    pub(crate) filament_change_length: Option<String>,
    pub(crate) required_nozzle_hrc: Option<String>,
    pub(crate) filament_map: Option<String>,
}

impl SliceOptions {
    pub fn filament_colour_config_export(&self) -> Result<Option<String>, SliceError> {
        optional_string_vector_export(self.values().get("filament_colour"), "filament_colour")
    }

    pub fn extruder_colour_config_export(&self) -> Result<Option<String>, SliceError> {
        if self.values().get("extruder_colour").is_none() {
            return Ok(None);
        }
        self.filament_colour_config_export()
    }

    pub fn default_filament_colour_config_export(&self) -> Result<Option<String>, SliceError> {
        optional_string_vector_export(
            self.values().get("default_filament_colour"),
            "default_filament_colour",
        )
    }

    pub(crate) fn filament_config_exports(&self) -> Result<FilamentConfigExports, SliceError> {
        Ok(FilamentConfigExports {
            filament_adhesiveness_category: optional_int_vector_export_in_range(
                self.values().get("filament_adhesiveness_category"),
                "filament_adhesiveness_category",
                0,
                i32::MAX,
            )?,
            cooling_tube_retraction: optional_non_negative_scalar_float_export(
                self.values().get("cooling_tube_retraction"),
                "cooling_tube_retraction",
            )?,
            cooling_tube_length: optional_non_negative_scalar_float_export(
                self.values().get("cooling_tube_length"),
                "cooling_tube_length",
            )?,
            high_current_on_filament_swap: optional_scalar_bool_export(
                self.values().get("high_current_on_filament_swap"),
                "high_current_on_filament_swap",
            )?,
            parking_pos_retraction: optional_non_negative_scalar_float_export(
                self.values().get("parking_pos_retraction"),
                "parking_pos_retraction",
            )?,
            extra_loading_move: optional_scalar_float_export(
                self.values().get("extra_loading_move"),
                "extra_loading_move",
            )?,
            machine_load_filament_time: optional_non_negative_scalar_float_export(
                self.values().get("machine_load_filament_time"),
                "machine_load_filament_time",
            )?,
            machine_tool_change_time: optional_non_negative_scalar_float_export(
                self.values().get("machine_tool_change_time"),
                "machine_tool_change_time",
            )?,
            machine_unload_filament_time: optional_non_negative_scalar_float_export(
                self.values().get("machine_unload_filament_time"),
                "machine_unload_filament_time",
            )?,
            filament_loading_speed: optional_float_vector_export(
                self.values().get("filament_loading_speed"),
                "filament_loading_speed",
            )?,
            filament_loading_speed_start: optional_float_vector_export(
                self.values().get("filament_loading_speed_start"),
                "filament_loading_speed_start",
            )?,
            filament_unloading_speed: optional_float_vector_export(
                self.values().get("filament_unloading_speed"),
                "filament_unloading_speed",
            )?,
            filament_unloading_speed_start: optional_float_vector_export(
                self.values().get("filament_unloading_speed_start"),
                "filament_unloading_speed_start",
            )?,
            filament_toolchange_delay: optional_float_vector_export(
                self.values().get("filament_toolchange_delay"),
                "filament_toolchange_delay",
            )?,
            filament_cooling_moves: optional_int_vector_export_in_range(
                self.values().get("filament_cooling_moves"),
                "filament_cooling_moves",
                0,
                20,
            )?,
            filament_cooling_initial_speed: optional_float_vector_export(
                self.values().get("filament_cooling_initial_speed"),
                "filament_cooling_initial_speed",
            )?,
            filament_minimal_purge_on_wipe_tower: optional_float_vector_export(
                self.values().get("filament_minimal_purge_on_wipe_tower"),
                "filament_minimal_purge_on_wipe_tower",
            )?,
            filament_cooling_before_tower: optional_filament_cooling_before_tower_export(
                self.values().get("filament_cooling_before_tower"),
            )?,
            filament_tower_interface_pre_extrusion_dist: optional_float_vector_export(
                self.values()
                    .get("filament_tower_interface_pre_extrusion_dist"),
                "filament_tower_interface_pre_extrusion_dist",
            )?,
            filament_tower_interface_pre_extrusion_length: optional_float_vector_export(
                self.values()
                    .get("filament_tower_interface_pre_extrusion_length"),
                "filament_tower_interface_pre_extrusion_length",
            )?,
            filament_tower_ironing_area: optional_float_vector_export(
                self.values().get("filament_tower_ironing_area"),
                "filament_tower_ironing_area",
            )?,
            filament_tower_interface_purge_volume: optional_float_vector_export(
                self.values().get("filament_tower_interface_purge_volume"),
                "filament_tower_interface_purge_volume",
            )?,
            filament_tower_interface_print_temp: optional_int_vector_export_in_range(
                self.values().get("filament_tower_interface_print_temp"),
                "filament_tower_interface_print_temp",
                -1,
                i32::MAX,
            )?,
            filament_cooling_final_speed: optional_float_vector_export(
                self.values().get("filament_cooling_final_speed"),
                "filament_cooling_final_speed",
            )?,
            filament_ramming_parameters: optional_string_vector_export(
                self.values().get("filament_ramming_parameters"),
                "filament_ramming_parameters",
            )?,
            filament_multitool_ramming: optional_bool_vector_export(
                self.values().get("filament_multitool_ramming"),
                "filament_multitool_ramming",
            )?,
            filament_multitool_ramming_volume: optional_float_vector_export(
                self.values().get("filament_multitool_ramming_volume"),
                "filament_multitool_ramming_volume",
            )?,
            filament_multitool_ramming_flow: optional_float_vector_export(
                self.values().get("filament_multitool_ramming_flow"),
                "filament_multitool_ramming_flow",
            )?,
            filament_stamping_loading_speed: optional_float_vector_export(
                self.values().get("filament_stamping_loading_speed"),
                "filament_stamping_loading_speed",
            )?,
            filament_stamping_distance: optional_float_vector_export(
                self.values().get("filament_stamping_distance"),
                "filament_stamping_distance",
            )?,
            wipe_tower_type: optional_wipe_tower_type_export(self.values().get("wipe_tower_type"))?,
            purge_in_prime_tower: optional_scalar_bool_export(
                self.values().get("purge_in_prime_tower"),
                "purge_in_prime_tower",
            )?,
            enable_filament_ramming: optional_scalar_bool_export(
                self.values().get("enable_filament_ramming"),
                "enable_filament_ramming",
            )?,
            tool_change_on_wipe_tower: optional_scalar_bool_export(
                self.values().get("tool_change_on_wipe_tower"),
                "tool_change_on_wipe_tower",
            )?,
            wipe_tower_no_sparse_layers: optional_scalar_bool_export(
                self.values().get("wipe_tower_no_sparse_layers"),
                "wipe_tower_no_sparse_layers",
            )?,
            support_multi_bed_types: optional_scalar_bool_export(
                self.values().get("support_multi_bed_types"),
                "support_multi_bed_types",
            )?,
            wipe_tower_x: optional_wipe_tower_coordinate_export(
                self.values().get("wipe_tower_x"),
                "wipe_tower_x",
            )?,
            wipe_tower_y: optional_wipe_tower_coordinate_export(
                self.values().get("wipe_tower_y"),
                "wipe_tower_y",
            )?,
            prime_tower_width: optional_scalar_float_export_with_bounds(
                self.values().get("prime_tower_width"),
                "prime_tower_width",
                Some(2.0),
                None,
            )?,
            wipe_tower_rotation_angle: optional_scalar_float_export(
                self.values().get("wipe_tower_rotation_angle"),
                "wipe_tower_rotation_angle",
            )?,
            prime_tower_brim_width: optional_scalar_float_export_with_bounds(
                self.values().get("prime_tower_brim_width"),
                "prime_tower_brim_width",
                Some(-1.0),
                None,
            )?,
            wipe_tower_bridging: optional_scalar_float_export(
                self.values().get("wipe_tower_bridging"),
                "wipe_tower_bridging",
            )?,
            wipe_tower_extra_flow: optional_scalar_float_export_with_bounds(
                self.values().get("wipe_tower_extra_flow"),
                "wipe_tower_extra_flow",
                Some(100.0),
                Some(300.0),
            )?,
            wipe_tower_cone_angle: optional_scalar_float_export_with_bounds(
                self.values().get("wipe_tower_cone_angle"),
                "wipe_tower_cone_angle",
                Some(0.0),
                Some(90.0),
            )?,
            wipe_tower_extra_spacing: optional_scalar_float_export_with_bounds(
                self.values().get("wipe_tower_extra_spacing"),
                "wipe_tower_extra_spacing",
                Some(100.0),
                Some(300.0),
            )?,
            wipe_tower_max_purge_speed: optional_scalar_float_export_with_bounds(
                self.values().get("wipe_tower_max_purge_speed"),
                "wipe_tower_max_purge_speed",
                Some(10.0),
                None,
            )?,
            wipe_tower_wall_type: optional_wipe_tower_wall_type_export(
                self.values().get("wipe_tower_wall_type"),
            )?,
            wipe_tower_extra_rib_length: optional_scalar_float_export_with_bounds(
                self.values().get("wipe_tower_extra_rib_length"),
                "wipe_tower_extra_rib_length",
                None,
                Some(300.0),
            )?,
            wipe_tower_rib_width: optional_scalar_float_export_with_bounds(
                self.values().get("wipe_tower_rib_width"),
                "wipe_tower_rib_width",
                Some(0.0),
                Some(300.0),
            )?,
            wipe_tower_fillet_wall: optional_scalar_bool_export(
                self.values().get("wipe_tower_fillet_wall"),
                "wipe_tower_fillet_wall",
            )?,
            wipe_tower_filament: optional_scalar_int_export_in_range(
                self.values().get("wipe_tower_filament"),
                "wipe_tower_filament",
                0,
                i32::MAX,
            )?,
            small_area_infill_flow_compensation_model: optional_small_area_flow_model_export(
                self.values()
                    .get("small_area_infill_flow_compensation_model"),
            )?,
            filament_colour: self.filament_colour_config_export()?,
            extruder_colour: self.extruder_colour_config_export()?,
            filament_multi_colour: optional_string_vector_export(
                self.values().get("filament_multi_colour"),
                "filament_multi_colour",
            )?,
            filament_colour_new: optional_float_vector_export(
                self.values().get("filament_colour_new"),
                "filament_colour_new",
            )?,
            filament_colour_type: optional_string_vector_export(
                self.values().get("filament_colour_type"),
                "filament_colour_type",
            )?,
            default_filament_colour: self.default_filament_colour_config_export()?,
            filament_ids: optional_string_vector_export(
                self.values().get("filament_ids"),
                "filament_ids",
            )?,
            filament_soluble: optional_bool_vector_export(
                self.values().get("filament_soluble"),
                "filament_soluble",
            )?,
            filament_is_support: optional_bool_vector_export(
                self.values().get("filament_is_support"),
                "filament_is_support",
            )?,
            filament_printable: optional_int_vector_export(
                self.values().get("filament_printable"),
                "filament_printable",
            )?,
            filament_change_length: optional_float_vector_export(
                self.values().get("filament_change_length"),
                "filament_change_length",
            )?,
            required_nozzle_hrc: optional_int_vector_export_in_range(
                self.values().get("required_nozzle_HRC"),
                "required_nozzle_HRC",
                0,
                500,
            )?,
            filament_map: optional_int_vector_export(
                self.values().get("filament_map"),
                "filament_map",
            )?,
        })
    }
}
