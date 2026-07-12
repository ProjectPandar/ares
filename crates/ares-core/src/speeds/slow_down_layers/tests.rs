use super::super::*;
use crate::{ExtrusionMove, Point2};

#[test]
fn slow_down_layers_interpolates_non_first_layer_print_speeds() {
    let layers = [
        LayerExtrusionMoves::new(0, 0.2, Vec::new(), 0.0),
        LayerExtrusionMoves::new(
            1,
            0.4,
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
                    Point2::new(1.0, 0.0),
                    Some(0.1),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::SparseInfill,
                    Point2::new(2.0, 0.0),
                    Some(0.2),
                ),
            ],
            0.2,
        ),
        LayerExtrusionMoves::new(
            4,
            1.0,
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
                    Point2::new(1.0, 0.0),
                    Some(0.3),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::SparseInfill,
                    Point2::new(2.0, 0.0),
                    Some(0.4),
                ),
            ],
            0.2,
        ),
    ];
    let options = SpeedOptions::new(120.0, 90.0, 150.0)
        .with_first_layer_speed(30.0)
        .with_first_layer_infill_speed(60.0)
        .with_slow_down_layers(4);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[1].moves()[0].speed_mm_s(), 120.0);
    assert_eq!(output[1].moves()[1].speed_mm_s(), 45.0);
    assert_eq!(output[1].moves()[2].speed_mm_s(), 82.5);
    assert_eq!(output[2].moves()[1].speed_mm_s(), 90.0);
    assert_eq!(output[2].moves()[2].speed_mm_s(), 150.0);
}

#[test]
fn dont_slow_down_outer_wall_preserves_external_perimeter_speed() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(1.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::InternalPerimeter,
                Point2::new(2.0, 0.0),
                Some(0.2),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(3.0, 0.0),
                Some(0.3),
            ),
        ],
        0.3,
    )];
    let base = SpeedOptions::new(120.0, 90.0, 150.0)
        .with_internal_perimeter_speed(90.0)
        .with_first_layer_speed(30.0)
        .with_first_layer_infill_speed(60.0)
        .with_slow_down_layers(4);

    let enabled = generate_speed_moves(&layers, base.with_dont_slow_down_outer_wall(true));
    let disabled = generate_speed_moves(&layers, base);

    assert_eq!(disabled[0].moves()[0].speed_mm_s(), 45.0);
    assert_eq!(enabled[0].moves()[0].speed_mm_s(), 90.0);
    assert_eq!(enabled[0].moves()[1].speed_mm_s(), 45.0);
    assert_eq!(enabled[0].moves()[2].speed_mm_s(), 82.5);
}

#[test]
fn dont_slow_down_outer_wall_keeps_first_layer_and_disabled_slow_layers_unchanged() {
    let first_layer = [LayerExtrusionMoves::new(
        0,
        0.2,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(1.0, 0.0),
            Some(0.1),
        )],
        0.1,
    )];
    let enabled_first_layer = SpeedOptions::new(120.0, 90.0, 100.0)
        .with_first_layer_speed(30.0)
        .with_slow_down_layers(4)
        .with_dont_slow_down_outer_wall(true);

    let first_layer_output = generate_speed_moves(&first_layer, enabled_first_layer);

    assert_eq!(first_layer_output[0].moves()[0].speed_mm_s(), 30.0);

    let layer_one = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(1.0, 0.0),
            Some(0.1),
        )],
        0.1,
    )];
    for slow_layers in [0, 1] {
        let disabled_slow_layers = SpeedOptions::new(120.0, 90.0, 100.0)
            .with_first_layer_speed(30.0)
            .with_slow_down_layers(slow_layers)
            .with_dont_slow_down_outer_wall(true);

        let output = generate_speed_moves(&layer_one, disabled_slow_layers);

        assert_eq!(output[0].moves()[0].speed_mm_s(), 90.0);
    }
}

#[test]
fn slow_down_layers_do_not_change_first_layer_skirt_or_low_normal_speeds() {
    let layers = [
        LayerExtrusionMoves::new(
            0,
            0.2,
            vec![ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(1.0, 0.0),
                Some(0.1),
            )],
            0.1,
        ),
        LayerExtrusionMoves::new(
            1,
            0.4,
            vec![
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::Skirt,
                    Point2::new(1.0, 0.0),
                    Some(0.2),
                ),
                ExtrusionMove::new(
                    ToolpathMoveKind::Print,
                    PrintPathRole::ExternalPerimeter,
                    Point2::new(2.0, 0.0),
                    Some(0.3),
                ),
            ],
            0.2,
        ),
    ];
    let options = SpeedOptions::new(120.0, 20.0, 100.0)
        .with_first_layer_speed(30.0)
        .with_skirt_speed(55.0)
        .with_slow_down_layers(4);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 30.0);
    assert_eq!(output[1].moves()[0].speed_mm_s(), 55.0);
    assert_eq!(output[1].moves()[1].speed_mm_s(), 20.0);
}

#[test]
fn slow_down_layers_disabled_for_zero_and_one() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(1.0, 0.0),
            Some(0.1),
        )],
        0.1,
    )];
    for slow_layers in [0, 1] {
        let options = SpeedOptions::new(120.0, 90.0, 100.0)
            .with_first_layer_speed(30.0)
            .with_slow_down_layers(slow_layers);

        let output = generate_speed_moves(&layers, options);

        assert_eq!(output[0].moves()[0].speed_mm_s(), 90.0);
    }
}

#[test]
fn volumetric_cap_applies_after_slow_down_layers() {
    let filament_area = std::f64::consts::PI * (1.75_f64 / 2.0).powi(2);
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
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
                Point2::new(1.0, 0.0),
                Some(1.0),
            ),
        ],
        1.0,
    )];
    let slow_only = SpeedOptions::new(120.0, 120.0, 100.0)
        .with_first_layer_speed(60.0)
        .with_slow_down_layers(4)
        .with_filament_max_volumetric_speed(0.0);
    let capped_after_slowdown = slow_only.with_filament_max_volumetric_speed(70.0 * filament_area);

    let slow_output = generate_speed_moves(&layers, slow_only);
    let capped_output = generate_speed_moves(&layers, capped_after_slowdown);

    assert_eq!(slow_output[0].moves()[1].speed_mm_s(), 75.0);
    assert_eq!(capped_output[0].moves()[1].speed_mm_s(), 70.0);
}

#[test]
fn volumetric_cap_still_applies_after_outer_wall_slowdown_skip() {
    let filament_area = std::f64::consts::PI * (1.75_f64 / 2.0).powi(2);
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
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
                Point2::new(1.0, 0.0),
                Some(1.0),
            ),
        ],
        1.0,
    )];
    let options = SpeedOptions::new(120.0, 120.0, 100.0)
        .with_first_layer_speed(60.0)
        .with_slow_down_layers(4)
        .with_dont_slow_down_outer_wall(true)
        .with_filament_max_volumetric_speed(90.0 * filament_area);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 90.0);
}
