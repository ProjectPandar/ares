use crate::{
    Contour, Point2, SliceOptions, gcode::format_gcode,
    pipeline::test_support::contour_layers_pipeline,
};
use serde_json::json;

#[test]
fn arachne_wall_simplification_changes_emitted_gcode_geometry() {
    let options: SliceOptions = serde_json::from_value(json!({
        "wall_generator": "arachne",
        "wall_maximum_resolution": 0.5,
        "wall_maximum_deviation": 0.05,
        "wall_loops": 1,
        "line_width": 0.4,
        "wall_sequence": "outer wall/inner wall",
        "seam_position": "aligned",
        "sparse_infill_density": 0
    }))
    .unwrap();

    let pipeline = contour_layers_pipeline(&options, vec![notched_contour()], 1);
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let emitted_geometry = emitted_geometry_lines(&gcode);

    assert!(emitted_geometry.contains(";PERIMETER:external:0,0 -> 0.6,0 -> 0.6,1 -> 0,1"));
    assert!(
        emitted_geometry.contains(";PRINT_PATH:external_perimeter:0,0 -> 0.6,0 -> 0.6,1 -> 0,1")
    );
    assert!(!emitted_geometry.contains("0.3,0.02"));
}

fn notched_contour() -> Contour {
    Contour::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(0.3, 0.02),
        Point2::new(0.6, 0.0),
        Point2::new(0.6, 1.0),
        Point2::new(0.0, 1.0),
    ])
}

fn emitted_geometry_lines(gcode: &str) -> String {
    gcode
        .lines()
        .filter(|line| line.starts_with(";PERIMETER:") || line.starts_with(";PRINT_PATH:"))
        .collect::<Vec<_>>()
        .join("\n")
}
