use super::*;
use crate::{ExtrusionMove, OverhangSpeedBands, Point2};

#[test]
fn overhang_perimeter_uses_bridge_fallback_speed_by_default() {
    let layers = [single_overhang_layer(1)];
    let options = SpeedOptions::new(120.0, 60.0, 100.0).with_bridge_speed(25.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 25.0);
}

#[test]
fn configured_overhang_speed_applies_after_first_layer() {
    let layers = [single_overhang_layer(1)];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_bridge_speed(25.0)
        .with_overhang_perimeter_speed(35.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 35.0);
}

#[test]
fn first_layer_overhang_uses_first_layer_wall_speed() {
    let layers = [single_overhang_layer(0)];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_first_layer_speed(30.0)
        .with_overhang_perimeter_speed(35.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 30.0);
}

#[test]
fn small_perimeter_speed_does_not_override_overhang_perimeter() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        square_moves(PrintPathRole::OverhangPerimeter),
        0.4,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_overhang_perimeter_speed(35.0)
        .with_small_perimeter_speed(20.0)
        .with_small_perimeter_threshold(10.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 35.0);
}

#[test]
fn overhang_speed_bands_select_by_unsupported_span() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![
            overhang_move(ToolpathMoveKind::Travel, Point2::new(0.0, 0.0), None, 0.1),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(1.0, 0.0),
                Some(0.1),
                0.1,
            ),
            overhang_move(ToolpathMoveKind::Travel, Point2::new(2.0, 0.0), None, 0.2),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(3.0, 0.0),
                Some(0.2),
                0.2,
            ),
            overhang_move(ToolpathMoveKind::Travel, Point2::new(4.0, 0.0), None, 0.3),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(5.0, 0.0),
                Some(0.3),
                0.3,
            ),
            overhang_move(ToolpathMoveKind::Travel, Point2::new(6.0, 0.0), None, 0.4),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(7.0, 0.0),
                Some(0.4),
                0.4,
            ),
        ],
        0.4,
    )];
    let capped_options = SpeedOptions::new(120.0, 80.0, 100.0)
        .with_bridge_speed(25.0)
        .with_overhang_perimeter_speed(25.0)
        .with_overhang_speed_bands(OverhangSpeedBands::new(
            0.4,
            [Some(70.0), Some(50.0), Some(35.0), Some(20.0)],
            Some(20.0),
        ));
    let uncapped_options = SpeedOptions::new(120.0, 90.0, 100.0)
        .with_overhang_perimeter_speed(90.0)
        .with_overhang_speed_bands(OverhangSpeedBands::new(
            0.4,
            [Some(70.0), Some(50.0), Some(35.0), Some(20.0)],
            Some(20.0),
        ));

    let capped = generate_speed_moves(&layers, capped_options);
    let uncapped = generate_speed_moves(&layers, uncapped_options);

    assert_eq!(capped[0].moves()[1].speed_mm_s(), 25.0);
    assert_eq!(capped[0].moves()[3].speed_mm_s(), 25.0);
    assert_eq!(capped[0].moves()[5].speed_mm_s(), 25.0);
    assert_eq!(capped[0].moves()[7].speed_mm_s(), 20.0);
    assert_eq!(uncapped[0].moves()[1].speed_mm_s(), 70.0);
    assert_eq!(uncapped[0].moves()[3].speed_mm_s(), 50.0);
    assert_eq!(uncapped[0].moves()[5].speed_mm_s(), 35.0);
    assert_eq!(uncapped[0].moves()[7].speed_mm_s(), 20.0);
}

#[test]
fn final_overhang_speed_bucket_uses_configured_severe_speed() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![
            overhang_move(ToolpathMoveKind::Travel, Point2::new(0.0, 0.0), None, 0.5),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(1.0, 0.0),
                Some(0.1),
                0.5,
            ),
        ],
        0.4,
    )];
    let curled_enabled = SpeedOptions::new(120.0, 80.0, 100.0)
        .with_overhang_perimeter_speed(80.0)
        .with_overhang_speed_bands(OverhangSpeedBands::new(
            0.4,
            [Some(70.0), Some(50.0), Some(35.0), Some(20.0)],
            Some(20.0),
        ));
    let curled_disabled = SpeedOptions::new(120.0, 80.0, 100.0)
        .with_overhang_perimeter_speed(80.0)
        .with_overhang_speed_bands(OverhangSpeedBands::new(
            0.4,
            [Some(70.0), Some(50.0), Some(35.0), Some(20.0)],
            Some(25.0),
        ));

    let enabled = generate_speed_moves(&layers, curled_enabled);
    let disabled = generate_speed_moves(&layers, curled_disabled);

    assert_eq!(enabled[0].moves()[1].speed_mm_s(), 20.0);
    assert_eq!(disabled[0].moves()[1].speed_mm_s(), 25.0);
}

#[test]
fn zero_and_subthreshold_overhang_speed_bands_keep_base_speed() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.4,
        vec![
            overhang_move(ToolpathMoveKind::Travel, Point2::new(0.0, 0.0), None, 0.1),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(1.0, 0.0),
                Some(0.1),
                0.1,
            ),
            overhang_move(ToolpathMoveKind::Travel, Point2::new(2.0, 0.0), None, 0.2),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(3.0, 0.0),
                Some(0.2),
                0.2,
            ),
        ],
        0.2,
    )];
    let options = SpeedOptions::new(120.0, 80.0, 100.0)
        .with_overhang_perimeter_speed(25.0)
        .with_overhang_speed_bands(OverhangSpeedBands::new(
            0.4,
            [Some(0.0), Some(0.4), Some(35.0), Some(20.0)],
            Some(20.0),
        ));

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 25.0);
    assert_eq!(output[0].moves()[3].speed_mm_s(), 25.0);
}

#[test]
fn first_layer_overhang_speed_bands_are_ignored() {
    let layers = [LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            overhang_move(ToolpathMoveKind::Travel, Point2::new(0.0, 0.0), None, 0.4),
            overhang_move(
                ToolpathMoveKind::Print,
                Point2::new(1.0, 0.0),
                Some(0.1),
                0.4,
            ),
        ],
        0.1,
    )];
    let options = SpeedOptions::new(120.0, 80.0, 100.0)
        .with_first_layer_speed(30.0)
        .with_overhang_perimeter_speed(80.0)
        .with_overhang_speed_bands(OverhangSpeedBands::new(
            0.4,
            [Some(70.0), Some(50.0), Some(35.0), Some(20.0)],
            Some(20.0),
        ));

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 30.0);
}

#[test]
fn slow_down_layers_interpolate_overhang_from_first_layer_wall_speed() {
    let layers = [single_overhang_layer(1)];
    let options = SpeedOptions::new(120.0, 90.0, 150.0)
        .with_first_layer_speed(30.0)
        .with_first_layer_infill_speed(60.0)
        .with_overhang_perimeter_speed(90.0)
        .with_slow_down_layers(4);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[1].speed_mm_s(), 45.0);
}

fn overhang_move(
    kind: ToolpathMoveKind,
    point: Point2,
    e_position: Option<f64>,
    unsupported_span_mm: f64,
) -> ExtrusionMove {
    ExtrusionMove::new(kind, PrintPathRole::OverhangPerimeter, point, e_position)
        .with_unsupported_span_mm(Some(unsupported_span_mm))
}

fn single_overhang_layer(layer_id: usize) -> LayerExtrusionMoves {
    LayerExtrusionMoves::new(
        layer_id,
        0.2 * (layer_id + 1) as f64,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::OverhangPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::OverhangPerimeter,
                Point2::new(1.0, 0.0),
                Some(0.1),
            ),
        ],
        0.1,
    )
}

fn square_moves(role: PrintPathRole) -> Vec<ExtrusionMove> {
    vec![
        ExtrusionMove::new(ToolpathMoveKind::Travel, role, Point2::new(0.0, 0.0), None),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            role,
            Point2::new(1.0, 0.0),
            Some(0.1),
        ),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            role,
            Point2::new(1.0, 1.0),
            Some(0.2),
        ),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            role,
            Point2::new(0.0, 1.0),
            Some(0.3),
        ),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            role,
            Point2::new(0.0, 0.0),
            Some(0.4),
        ),
    ]
}
