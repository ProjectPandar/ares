use crate::{
    Contour, Point2, PrintPathRole, SliceOptions, pipeline::test_support::contours_pipeline,
};
use serde_json::json;

#[test]
fn pipeline_consumes_combine_brims_in_first_layer_brim_output() {
    let separate: SliceOptions = serde_json::from_value(json!({
        "brim_width": 0.4,
        "brim_type": "outer_only",
        "skirt_loops": 0,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0
    }))
    .unwrap();
    let combined: SliceOptions = serde_json::from_value(json!({
        "brim_width": 0.4,
        "brim_type": "outer_only",
        "combine_brims": true,
        "skirt_loops": 0,
        "sparse_infill_density": 0,
        "filament_max_volumetric_speed": 0.0
    }))
    .unwrap();
    let contours = vec![square(0.0, 0.0, 1.0, 1.0), square(3.0, 0.0, 4.0, 1.0)];

    let separate_pipeline = contours_pipeline(&separate, contours.clone());
    let combined_pipeline = contours_pipeline(&combined, contours);

    assert_eq!(separate_pipeline.layer_brims()[0].paths().len(), 2);
    assert_eq!(combined_pipeline.layer_brims()[0].paths().len(), 1);
    assert_eq!(separate_pipeline.diagnostics().total_brim_path_count(), 2);
    assert_eq!(combined_pipeline.diagnostics().total_brim_path_count(), 1);
    assert_eq!(
        combined_pipeline.layer_print_paths()[0].paths()[0].role(),
        PrintPathRole::Brim
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
