use crate::{
    Point2, PrintPathRole, SliceOptions, pipeline::test_support::rectangular_layers_pipeline,
};
use serde_json::json;

#[test]
fn infill_wall_overlap_reaches_middle_sparse_print_paths_and_gcode() {
    let zero_overlap: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "sparse_infill_rotate_template": "0",
        "infill_anchor": 0,
        "infill_wall_overlap": 0,
        "top_bottom_infill_wall_overlap": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();
    let default_overlap: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 2,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "sparse_infill_rotate_template": "0",
        "infill_anchor": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let zero_pipeline = rectangular_layers_pipeline(&zero_overlap, 3);
    let default_pipeline = rectangular_layers_pipeline(&default_overlap, 3);

    assert!(
        zero_pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::SparseInfill
                    && path.points() == [Point2::new(1.5, 0.6), Point2::new(1.5, 3.4)]
            })
    );
    assert!(
        default_pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::SparseInfill
                    && path.points() == [Point2::new(1.5, 0.54), Point2::new(1.5, 3.46)]
            })
    );

    let zero_gcode =
        String::from_utf8(crate::gcode::format_gcode(&zero_pipeline, &zero_overlap).unwrap())
            .unwrap();
    let default_gcode =
        String::from_utf8(crate::gcode::format_gcode(&default_pipeline, &default_overlap).unwrap())
            .unwrap();

    assert!(zero_gcode.contains(";PRINT_PATH:sparse_infill:1.5,0.6 -> 1.5,3.4"));
    assert!(default_gcode.contains(";PRINT_PATH:sparse_infill:1.5,0.54 -> 1.5,3.46"));
}

#[test]
fn zero_wall_loops_preserve_original_sparse_gcode_boundary() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 0,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "infill_direction": 0,
        "infill_anchor": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = rectangular_layers_pipeline(&options, 1);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";PRINT_PATH:sparse_infill:0.5,0 -> 0.5,4"));
}
