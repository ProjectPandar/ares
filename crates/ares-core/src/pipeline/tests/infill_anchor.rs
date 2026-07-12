use super::*;

#[test]
fn infill_anchor_reaches_print_paths_and_gcode_comments() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "infill_anchor": 0.25,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = rectangular_pipeline(&options);

    assert!(pipeline.layer_print_paths()[0].paths().iter().any(|path| {
        path.role() == PrintPathRole::SparseInfill
            && path.points() == [Point2::new(0.5, -0.25), Point2::new(0.5, 4.25)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";PRINT_PATH:sparse_infill:0.5,-0.25 -> 0.5,4.25"));
}
