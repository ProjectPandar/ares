use crate::{
    Contour, Point2, PrintPathRole, SliceOptions, gcode::format_gcode,
    pipeline::test_support::contour_layers_pipeline_from_layers_for_tests,
};
use serde_json::json;

#[test]
fn combined_skirt_height_uses_aggregate_bounds_for_all_skirt_layers() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "combined",
        "skirt_height": 2,
        "skirt_loops": 1,
        "skirt_distance": 1.0,
        "skirt_line_width": 0.4,
        "wall_loops": 0,
        "sparse_infill_density": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            vec![square(0.0, 0.0, 4.0, 4.0)],
            vec![square(1.0, 1.0, 2.0, 2.0)],
        ],
    );

    assert_eq!(pipeline.layer_skirts()[0].paths().len(), 1);
    assert_eq!(pipeline.layer_skirts()[1].paths().len(), 1);
    assert_eq!(
        pipeline.layer_skirts()[0].paths()[0].points(),
        pipeline.layer_skirts()[1].paths()[0].points()
    );
    assert_eq!(
        pipeline.layer_skirts()[1].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(5.0, -1.0),
            Point2::new(5.0, 5.0),
            Point2::new(-1.0, 5.0),
        ]
    );
    assert_eq!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .filter(|path| path.role() == PrintPathRole::Skirt)
            .count(),
        1
    );

    let gcode = String::from_utf8(format_gcode(&pipeline, &options).unwrap()).unwrap();
    let layer_one = gcode
        .lines()
        .skip_while(|line| *line != ";LAYER:1")
        .collect::<Vec<_>>();

    assert!(layer_one.contains(&";SKIRT:-1,-1 -> 5,-1 -> 5,5 -> -1,5"));
    assert!(layer_one.contains(&";PRINT_PATH:skirt:-1,-1 -> 5,-1 -> 5,5 -> -1,5"));
}

#[test]
fn skirt_height_one_keeps_second_layer_skirt_free() {
    let options: SliceOptions = serde_json::from_value(json!({
        "skirt_type": "combined",
        "skirt_height": 1,
        "skirt_loops": 1,
        "skirt_distance": 1.0,
        "skirt_line_width": 0.4,
        "wall_loops": 0,
        "sparse_infill_density": 0,
        "brim_width": 0.0
    }))
    .unwrap();

    let pipeline = contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            vec![square(0.0, 0.0, 4.0, 4.0)],
            vec![square(1.0, 1.0, 2.0, 2.0)],
        ],
    );

    assert_eq!(pipeline.layer_skirts()[0].paths().len(), 1);
    assert!(pipeline.layer_skirts()[1].paths().is_empty());
    assert!(
        pipeline.layer_print_paths()[1]
            .paths()
            .iter()
            .all(|path| path.role() != PrintPathRole::Skirt)
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
