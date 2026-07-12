use super::*;
use serde_json::json;

fn layer_block(gcode: &str, layer_id: usize) -> &str {
    let marker = format!(";LAYER:{layer_id}\n");
    let start = gcode.find(&marker).expect("layer marker");
    let rest = &gcode[start..];
    let next = rest.find("\n;LAYER_CHANGE\n;LAYER:").unwrap_or(rest.len());
    &rest[..next]
}

#[test]
fn sparse_infill_grid_pattern_adds_perpendicular_gcode_moves() {
    let rectilinear_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();
    let grid_options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "grid",
        "infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let rectilinear_pipeline = rectangular_pipeline(&rectilinear_options);
    let grid_pipeline = rectangular_pipeline(&grid_options);

    assert_eq!(rectilinear_pipeline.layer_infills()[0].paths().len(), 4);
    assert_eq!(grid_pipeline.layer_infills()[0].paths().len(), 8);
    assert!(grid_pipeline.layer_infills()[0].paths().iter().any(|path| {
        let points = path.points();
        points[0].y() == points[1].y()
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&grid_pipeline, &grid_options).unwrap())
            .unwrap();

    assert!(gcode.contains(";PRINT_PATH:sparse_infill:"));
}

#[test]
fn sparse_infill_zigzag_pattern_reaches_paths_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "zigzag",
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

    assert_eq!(pipeline.layer_infills()[0].paths().len(), 4);
    assert!(pipeline.layer_infills()[0].paths().iter().any(|path| {
        let points = path.points();
        points == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]
    }));
    assert!(pipeline.layer_print_paths()[0].paths().iter().any(|path| {
        path.role() == crate::PrintPathRole::SparseInfill
            && path.points() == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";EXTRUSION:print:sparse_infill:"));
}

#[test]
fn sparse_infill_crosszag_pattern_reaches_paths_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "crosszag",
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

    assert_eq!(pipeline.layer_infills()[0].paths().len(), 4);
    assert!(pipeline.layer_infills()[0].paths().iter().any(|path| {
        let points = path.points();
        points == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]
    }));
    assert!(pipeline.layer_print_paths()[0].paths().iter().any(|path| {
        path.role() == crate::PrintPathRole::SparseInfill
            && path.points() == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";EXTRUSION:print:sparse_infill:"));
}

#[test]
fn sparse_infill_lockedzag_pattern_reaches_paths_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "lockedzag",
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

    assert!(pipeline.layer_infills()[0].paths().iter().any(|path| {
        path.role() == crate::InfillRole::Sparse
            && path.points() == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]
    }));
    assert!(pipeline.layer_print_paths()[0].paths().iter().any(|path| {
        path.role() == crate::PrintPathRole::SparseInfill
            && path.points() == [Point2::new(1.5, 4.0), Point2::new(1.5, 0.0)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:1.5,4 -> 1.5,0"));
    assert!(gcode.contains(";EXTRUSION:print:sparse_infill:"));
}

#[test]
fn sparse_infill_crosszag_shift_step_reaches_layer_two_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "crosszag",
        "infill_direction": 0,
        "infill_shift_step": 0.25,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 3);

    assert!(
        pipeline.layer_infills()[2]
            .paths()
            .iter()
            .any(|path| { path.points() == [Point2::new(0.25, 0.0), Point2::new(0.25, 4.0)] })
    );

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer_two = layer_block(&gcode, 2);

    assert!(layer_two.contains(";INFILL:sparse:0.25,0 -> 0.25,4"));
    assert!(layer_two.contains(";PRINT_PATH:sparse_infill:0.25,0 -> 0.25,4"));
}

#[test]
fn sparse_infill_rotate_template_reaches_paths_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "sparse_infill_rotate_template": "90,0",
        "infill_direction": 0,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "infill_anchor_max": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = crate::pipeline::test_support::rectangular_layers_pipeline(&options, 2);

    assert!(
        pipeline.layer_infills()[1]
            .paths()
            .iter()
            .any(|path| { path.points() == [Point2::new(0.5, 0.0), Point2::new(0.5, 4.0)] })
    );
    assert!(pipeline.layer_print_paths()[1].paths().iter().any(|path| {
        path.role() == crate::PrintPathRole::SparseInfill
            && path.points() == [Point2::new(0.5, 0.0), Point2::new(0.5, 4.0)]
    }));

    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(gcode.contains(";INFILL:sparse:0.5,0 -> 0.5,4"));
    assert!(gcode.contains(";PRINT_PATH:sparse_infill:0.5,0 -> 0.5,4"));
}

#[test]
fn minimum_sparse_infill_area_suppresses_sparse_infill_paths_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "minimum_sparse_infill_area": 16,
        "wall_loops": 0,
        "skirt_loops": 0,
        "brim_width": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();

    let pipeline = rectangular_pipeline(&options);
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(pipeline.layer_infills()[0].paths().is_empty());
    assert!(
        !pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == crate::PrintPathRole::SparseInfill)
    );
    assert!(
        !gcode
            .lines()
            .any(|line| line.starts_with(";INFILL:sparse:"))
    );
    assert!(
        !gcode
            .lines()
            .any(|line| line.starts_with(";PRINT_PATH:sparse_infill:"))
    );
}
