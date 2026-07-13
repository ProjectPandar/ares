use super::*;

#[test]
fn gcode_options_projection_preserves_each_process_field() {
    assert_process_projection!(enable_arc_fitting, OrcaBool(true));
    assert_process_projection!(enable_wrapping_detection, OrcaBool(true));
    assert_process_projection!(gcode_add_line_number, OrcaBool(true));
    assert_process_projection!(max_volumetric_extrusion_rate_slope, OrcaFloat(9201.01));
    assert_process_projection!(
        max_volumetric_extrusion_rate_slope_segment_length,
        OrcaFloat(9202.02)
    );
    assert_process_projection!(
        extrusion_rate_smoothing_external_perimeter_only,
        OrcaBool(true)
    );
    assert_process_projection!(single_extruder_multi_material_priming, OrcaBool(true));
    assert_process_projection!(wipe_tower_no_sparse_layers, OrcaBool(true));
    assert_process_projection!(
        process_change_extrusion_role_gcode,
        OrcaString("process-role".into())
    );
    assert_process_projection!(travel_speed, OrcaFloat(9203.03));
    assert_process_projection!(travel_speed_z, OrcaFloat(9204.04));
    assert_process_projection!(accel_to_decel_enable, OrcaBool(false));
    assert_process_projection!(accel_to_decel_factor, Percent(9205.05));
    assert_process_projection!(initial_layer_travel_speed, FloatOrPercent::Float(9206.06));
    assert_process_projection!(
        initial_layer_travel_acceleration,
        FloatOrPercent::Float(9207.07)
    );
    assert_process_projection!(initial_layer_travel_jerk, FloatOrPercent::Float(9208.08));
    assert_process_projection!(
        small_area_infill_flow_compensation_model,
        OrcaStrings(vec!["process-model-a".into(), "process-model-b".into()])
    );
}
