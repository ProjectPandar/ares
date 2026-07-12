use crate::{Point2, PrintPathRole, SliceOptions, pipeline::test_support::rectangular_pipeline};
use serde_json::json;

#[test]
fn symmetric_infill_y_axis_reaches_zigzag_paths_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "zigzag",
        "symmetric_infill_y_axis": true,
        "infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = rectangular_pipeline(&options);

    assert!(
        pipeline.layer_infills()[0]
            .paths()
            .iter()
            .any(|path| { path.points() == [Point2::new(3.5, 0.0), Point2::new(3.5, 4.0)] })
    );
    assert!(pipeline.layer_print_paths()[0].paths().iter().any(|path| {
        path.role() == PrintPathRole::SparseInfill
            && path.points() == [Point2::new(3.5, 0.0), Point2::new(3.5, 4.0)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:3.5,0 -> 3.5,4"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:3.5,0 -> 3.5,4"));
}
