use super::*;
use serde_json::json;

#[test]
fn painted_brims_remain_empty_in_pipeline() {
    let painted_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_type": "painted",
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let painted_pipeline = rectangular_pipeline(&painted_options);
    assert!(painted_pipeline.layer_brims()[0].paths().is_empty());
}

#[test]
fn brim_ears_max_angle_zero_reaches_pipeline_and_gcode_as_no_brims() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_type": "brim_ears",
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "brim_ears_max_angle": 0,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let pipeline = rectangular_pipeline(&options);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(pipeline.layer_brims()[0].paths().is_empty());
    assert!(gcode.lines().any(|line| line == "; brim_count = 0"));
    assert!(!gcode.lines().any(|line| line.starts_with(";BRIM:")));
}
