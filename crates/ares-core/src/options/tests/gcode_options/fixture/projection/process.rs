use super::*;

pub(super) fn assert_fields(fixture: &Fixture) -> BTreeSet<&'static str> {
    assert_source_fields! {
        fixture, process;
        "enable_arc_fitting" => enable_arc_fitting,
        "enable_wrapping_detection" => enable_wrapping_detection,
        "gcode_add_line_number" => gcode_add_line_number,
        "max_volumetric_extrusion_rate_slope" => max_volumetric_extrusion_rate_slope,
        "max_volumetric_extrusion_rate_slope_segment_length" => max_volumetric_extrusion_rate_slope_segment_length,
        "extrusion_rate_smoothing_external_perimeter_only" => extrusion_rate_smoothing_external_perimeter_only,
        "single_extruder_multi_material_priming" => single_extruder_multi_material_priming,
        "wipe_tower_no_sparse_layers" => wipe_tower_no_sparse_layers,
        "process_change_extrusion_role_gcode" => process_change_extrusion_role_gcode,
        "travel_speed" => travel_speed,
        "travel_speed_z" => travel_speed_z,
        "accel_to_decel_enable" => accel_to_decel_enable,
        "accel_to_decel_factor" => accel_to_decel_factor,
        "initial_layer_travel_speed" => initial_layer_travel_speed,
        "initial_layer_travel_acceleration" => initial_layer_travel_acceleration,
        "initial_layer_travel_jerk" => initial_layer_travel_jerk,
        "small_area_infill_flow_compensation_model" => small_area_infill_flow_compensation_model,
    }
}
