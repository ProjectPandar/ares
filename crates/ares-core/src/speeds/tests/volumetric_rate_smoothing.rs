use super::*;
use crate::{ExtrusionMove, Point2};

#[test]
fn volumetric_rate_slope_limits_positive_adjacent_print_flow_jump() {
    let layer = LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(10.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(20.0, 0.0),
                Some(1.1),
            ),
        ],
        1.1,
    );
    let base_options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(0.0);
    let smoothed_options = base_options.with_max_volumetric_extrusion_rate_slope(1.0);

    let base = generate_speed_moves(std::slice::from_ref(&layer), base_options);
    let smoothed = generate_speed_moves(&[layer], smoothed_options);

    assert_eq!(base[0].moves()[2].speed_mm_s(), 100.0);
    assert!(smoothed[0].moves()[2].speed_mm_s() < base[0].moves()[2].speed_mm_s());
    let expected = (std::f64::consts::PI + 0.1) / (std::f64::consts::PI / 10.0);
    assert!((smoothed[0].moves()[2].speed_mm_s() - expected).abs() <= 0.000000001);
}

#[test]
fn volumetric_rate_slope_keeps_first_print_speed_without_baseline() {
    let layer = LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(10.0, 0.0),
                Some(1.0),
            ),
        ],
        1.0,
    );
    let options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(0.0)
        .with_max_volumetric_extrusion_rate_slope(1.0);

    let output = generate_speed_moves(&[layer], options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 100.0);
}

#[test]
fn external_only_smoothing_skips_sparse_infill_and_keeps_external_baseline() {
    let layer = LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(10.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(20.0, 0.0),
                Some(1.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(30.0, 0.0),
                Some(2.1),
            ),
        ],
        2.1,
    );
    let options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_speed(100.0)
        .with_first_layer_infill_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(0.0)
        .with_max_volumetric_extrusion_rate_slope(1.0)
        .with_extrusion_rate_smoothing_external_perimeter_only(true);

    let output = generate_speed_moves(&[layer], options);

    assert_eq!(output[0].moves()[2].speed_mm_s(), 100.0);
    let expected = (std::f64::consts::PI + 0.1) / (std::f64::consts::PI / 10.0);
    assert!((output[0].moves()[3].speed_mm_s() - expected).abs() <= 0.000000001);
}
