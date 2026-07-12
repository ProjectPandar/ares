use crate::{
    Contour, ExtrusionRole, Point2, PrintPathRole, SliceOptions, gcode::format_gcode,
    pipeline::test_support::contour_layers_pipeline,
};
use serde_json::json;

const THIN_WALL_MARKER: &str = ";PRINT_PATH:external_perimeter:0.4,0.35 -> 2.6,0.35";

#[test]
fn min_length_factor_removes_short_thin_wall_from_middle_layer_gcode() {
    let options = options(json!({ "detect_thin_wall": true, "min_length_factor": 6.0 }));
    let pipeline = thin_wall_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let middle_region = &pipeline.print().objects()[0].layers()[1].regions()[0];

    assert!(!layer_print_paths_have_centerline(&pipeline, 1));
    assert!(
        !middle_region
            .perimeters()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::ExternalPerimeter
                && path.points() == thin_wall_centerline())
    );
    assert_eq!(marker_count(&gcode), 2);
}

#[test]
fn min_length_factor_zero_preserves_detected_thin_wall_gcode() {
    let options = options(json!({ "detect_thin_wall": true, "min_length_factor": 0.0 }));
    let pipeline = thin_wall_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert!(layer_print_paths_have_centerline(&pipeline, 0));
    assert!(layer_print_paths_have_centerline(&pipeline, 1));
    assert!(layer_print_paths_have_centerline(&pipeline, 2));
    assert_eq!(marker_count(&gcode), 3);
}

fn layer_print_paths_have_centerline(pipeline: &crate::SlicingPipeline, layer_id: usize) -> bool {
    pipeline.layer_print_paths()[layer_id]
        .paths()
        .iter()
        .any(|path| {
            path.role() == PrintPathRole::ExternalPerimeter
                && path.points() == thin_wall_centerline()
        })
}

fn marker_count(gcode: &str) -> usize {
    gcode
        .lines()
        .filter(|line| *line == THIN_WALL_MARKER)
        .count()
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4,
        "wall_loops": 4,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0,
        "sparse_infill_density": 0,
        "minimum_sparse_infill_area": 0
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn thin_wall_pipeline(options: &SliceOptions) -> crate::SlicingPipeline {
    contour_layers_pipeline(
        options,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(3.0, 0.7),
            Point2::new(0.0, 0.7),
        ])],
        3,
    )
}

fn thin_wall_centerline() -> [Point2; 2] {
    [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
}
