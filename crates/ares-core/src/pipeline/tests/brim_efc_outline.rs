use super::*;
use serde_json::json;

#[test]
fn brim_efc_outline_changes_pipeline_paths_and_gcode() {
    let raw: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();
    let efc: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "brim_width": 0.4,
        "brim_object_gap": 0,
        "brim_use_efc_outline": true,
        "elefant_foot_compensation": 0.2,
        "elefant_foot_compensation_layers": 1,
        "raft_layers": 0,
        "line_width": 0.4,
        "skirt_loops": 0,
        "wall_loops": 0,
        "sparse_infill_density": 0
    }))
    .unwrap();

    let raw_pipeline = rectangular_pipeline(&raw);
    let efc_pipeline = rectangular_pipeline(&efc);

    assert_eq!(raw_pipeline.diagnostics().total_brim_path_count(), 1);
    assert_eq!(efc_pipeline.diagnostics().total_brim_path_count(), 1);
    assert_eq!(
        raw_pipeline.layer_brims()[0].paths()[0].points()[0],
        Point2::new(-0.4, -0.4)
    );
    assert_eq!(
        efc_pipeline.layer_brims()[0].paths()[0].points()[0],
        Point2::new(-0.2, -0.2)
    );

    let raw_gcode =
        String::from_utf8(crate::gcode::format_gcode(&raw_pipeline, &raw).unwrap()).unwrap();
    let efc_gcode =
        String::from_utf8(crate::gcode::format_gcode(&efc_pipeline, &efc).unwrap()).unwrap();

    assert!(raw_gcode.contains(";BRIM:-0.4,-0.4 -> 4.4,-0.4"));
    assert!(efc_gcode.contains(";BRIM:-0.2,-0.2 -> 4.2,-0.2"));
}
