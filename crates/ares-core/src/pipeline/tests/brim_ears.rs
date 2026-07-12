use super::*;
use serde_json::json;

#[test]
fn brim_ears_reach_print_paths_and_gcode() {
    let ear_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_type": "brim_ears",
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();
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

    let ear_pipeline = rectangular_pipeline(&ear_options);
    let painted_pipeline = rectangular_pipeline(&painted_options);

    assert_eq!(ear_pipeline.layer_brims()[0].paths().len(), 4);
    assert!(painted_pipeline.layer_brims()[0].paths().is_empty());
    assert_eq!(
        ear_pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .filter(|path| path.role() == PrintPathRole::Brim)
            .count(),
        4
    );

    let gcode = String::from_utf8(crate::gcode::format_gcode(&ear_pipeline, &ear_options).unwrap())
        .unwrap();

    assert!(gcode.contains(";BRIM:-0.4,-0.4 -> 0.4,-0.4 -> 0.4,0.4 -> -0.4,0.4"));
    assert!(gcode.contains(";PRINT_PATH:brim:"));
    assert!(gcode.contains(";EXTRUSION:print:brim:"));
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

#[test]
fn brim_ears_detection_length_reaches_pipeline_and_gcode() {
    let raw_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_type": "brim_ears",
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "brim_ears_detection_length": 0,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let simplified_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_type": "brim_ears",
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "brim_ears_detection_length": 0.5,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let raw_pipeline = crate::pipeline::test_support::kinked_brim_pipeline(&raw_options);
    let simplified_pipeline =
        crate::pipeline::test_support::kinked_brim_pipeline(&simplified_options);

    assert!(
        raw_pipeline.layer_brims()[0].paths().len()
            > simplified_pipeline.layer_brims()[0].paths().len()
    );
    assert!(brim_print_path_count(&raw_pipeline) > brim_print_path_count(&simplified_pipeline));

    let raw_gcode =
        String::from_utf8(crate::gcode::format_gcode(&raw_pipeline, &raw_options).unwrap())
            .unwrap();
    let simplified_gcode = String::from_utf8(
        crate::gcode::format_gcode(&simplified_pipeline, &simplified_options).unwrap(),
    )
    .unwrap();

    assert!(raw_gcode.matches(";BRIM:").count() > simplified_gcode.matches(";BRIM:").count());
}

fn brim_print_path_count(pipeline: &SlicingPipeline) -> usize {
    pipeline.layer_print_paths()[0]
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::Brim)
        .count()
}
