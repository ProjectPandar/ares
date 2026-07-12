use super::*;
use serde_json::json;

fn fill_multiline_options(pattern: &str, fill_multiline: u32) -> SliceOptions {
    serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": pattern,
        "fill_multiline": fill_multiline,
        "infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap()
}

#[test]
fn fill_multiline_changes_sparse_paths_and_gcode_coordinates() {
    let options = fill_multiline_options("rectilinear", 3);
    let pipeline = rectangular_pipeline(&options);

    assert_eq!(pipeline.layer_infills()[0].paths().len(), 3);
    assert!(
        pipeline.layer_infills()[0]
            .paths()
            .iter()
            .any(|path| { path.points() == [Point2::new(1.0, 0.0), Point2::new(1.0, 4.0)] })
    );
    assert!(pipeline.layer_print_paths()[0].paths().iter().any(|path| {
        path.role() == PrintPathRole::SparseInfill
            && path.points() == [Point2::new(2.0, 0.0), Point2::new(2.0, 4.0)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:1,0 -> 1,4"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:2,0 -> 2,4"));
    assert!(!gcode.contains(";INFILL:sparse:0.5,0 -> 0.5,4"));
    assert!(!gcode.contains(";INFILL:sparse:2.5,0 -> 2.5,4"));
    assert!(!gcode.contains(";INFILL:sparse:3.5,0 -> 3.5,4"));
}

#[test]
fn zigzag_ignores_fill_multiline_in_this_slice() {
    let options = fill_multiline_options("zigzag", 3);
    let pipeline = rectangular_pipeline(&options);

    assert_eq!(pipeline.layer_infills()[0].paths().len(), 4);
    assert!(
        pipeline.layer_infills()[0]
            .paths()
            .iter()
            .any(|path| { path.points() == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)] })
    );

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:1.5,4 -> 1.5,0"));
}
