use crate::{
    Contour, Point2, PrintPathRole, SliceOptions, gcode::format_gcode,
    pipeline::test_support::contours_pipeline,
};
use serde_json::json;

#[test]
fn per_object_skirts_reach_print_paths_moves_and_gcode() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "perobject",
        "skirt_loops": 1,
        "skirt_height": 1,
        "skirt_distance": 1.0,
        "skirt_speed": 50.0,
        "skirt_line_width": 0.4,
        "wall_loops": 0,
        "sparse_infill_density": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = contours_pipeline(
        &options,
        vec![square(0.0, 0.0, 1.0, 1.0), square(3.0, 0.0, 4.0, 1.0)],
    );
    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(pipeline.layer_skirts()[0].paths().len(), 2);
    assert_eq!(
        pipeline.layer_print_paths()[0]
            .paths()
            .iter()
            .filter(|path| path.role() == PrintPathRole::Skirt)
            .count(),
        2
    );
    assert!(
        pipeline.layer_extrusion_moves()[0]
            .moves()
            .iter()
            .any(|movement| movement.role() == PrintPathRole::Skirt)
    );
    assert!(
        pipeline.layer_speed_moves()[0]
            .moves()
            .iter()
            .any(|movement| movement.role() == PrintPathRole::Skirt)
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SKIRT:-1,-1 -> 2,-1 -> 2,2 -> -1,2")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";SKIRT:2,-1 -> 5,-1 -> 5,2 -> 2,2")
    );
    assert!(
        gcode
            .lines()
            .any(|line| line == ";PRINT_PATH:skirt:2,-1 -> 5,-1 -> 5,2 -> 2,2")
    );
}

fn square(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}
