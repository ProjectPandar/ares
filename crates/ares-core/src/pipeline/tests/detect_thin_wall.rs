use crate::{
    ExtrusionRole, Point2, PrintPathRole, SliceOptions, gcode::format_gcode,
    pipeline::test_support::narrow_rectangular_gap_fill_pipeline,
};
use serde_json::json;

#[test]
fn detect_thin_wall_replaces_rectangular_wall_gap_fill_with_external_perimeter() {
    let disabled = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 45
    }));
    let enabled = options(json!({
        "wall_loops": 4,
        "gap_infill_speed": 45,
        "detect_thin_wall": true
    }));

    let disabled_pipeline = narrow_rectangular_gap_fill_pipeline(&disabled);
    let enabled_pipeline = narrow_rectangular_gap_fill_pipeline(&enabled);
    let enabled_gcode =
        String::from_utf8(format_gcode(&enabled_pipeline, &enabled).unwrap()).unwrap();
    let enabled_region = &enabled_pipeline.print().objects()[0].layers()[0].regions()[0];

    assert_eq!(disabled_pipeline.layer_gap_fills()[0].paths().len(), 1);
    assert_eq!(enabled_pipeline.layer_gap_fills()[0].paths().len(), 0);
    assert!(
        enabled_pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| {
                path.role() == PrintPathRole::ExternalPerimeter
                    && path.points() == [Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
            })
    );
    assert!(
        !enabled_pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::GapFill)
    );
    assert!(
        !enabled_region
            .extras()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::GapFill)
    );
    assert!(
        enabled_region
            .perimeters()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::ExternalPerimeter)
    );
    assert!(enabled_gcode.contains(";PRINT_PATH:external_perimeter:"));
    assert!(!enabled_gcode.contains(";PRINT_PATH:gap_fill:"));
    assert!(!enabled_gcode.contains(";SPEED:print:gap_fill:"));
}

#[test]
fn detect_thin_wall_keeps_solid_surface_gap_fill_behavior() {
    let options = options(json!({
        "wall_loops": 0,
        "gap_fill_target": "topbottom",
        "sparse_infill_density": 100,
        "minimum_sparse_infill_area": 0,
        "bottom_shell_layers": 1,
        "top_shell_layers": 1,
        "gap_infill_speed": 45,
        "detect_thin_wall": true
    }));
    let pipeline = narrow_rectangular_gap_fill_pipeline(&options);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let region = &pipeline.print().objects()[0].layers()[0].regions()[0];

    assert_eq!(pipeline.layer_gap_fills()[0].paths().len(), 1);
    assert!(
        region
            .extras()
            .paths()
            .iter()
            .any(|path| path.role() == ExtrusionRole::GapFill)
    );
    assert!(gcode.contains(";PRINT_PATH:gap_fill:"));
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4,
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
