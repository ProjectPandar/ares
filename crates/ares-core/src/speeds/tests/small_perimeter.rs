use super::*;

#[test]
fn small_perimeter_speed_applies_to_external_span_under_orca_length_threshold() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        external_square_moves(),
        0.4,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_small_perimeter_speed(20.0)
        .with_small_perimeter_threshold(3.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 120.0);
    for move_ in &output[0].moves()[1..] {
        assert_eq!(move_.speed_mm_s(), 20.0);
        assert_eq!(move_.feedrate_mm_min(), 1200.0);
    }
}

#[test]
fn small_perimeter_threshold_uses_orca_circumference_conversion() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        external_square_moves(),
        0.4,
    )];
    let below = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_small_perimeter_speed(20.0)
        .with_small_perimeter_threshold(2.5);
    let above = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_small_perimeter_speed(20.0)
        .with_small_perimeter_threshold(2.6);

    let below_output = generate_speed_moves(&layers, below);
    let above_output = generate_speed_moves(&layers, above);

    assert_eq!(below_output[0].moves()[1].speed_mm_s(), 60.0);
    assert_eq!(above_output[0].moves()[1].speed_mm_s(), 20.0);
}

#[test]
fn small_perimeter_speed_does_not_apply_to_non_external_or_print_before_travel() {
    let layers = [LayerExtrusionMoves::new(
        1,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
                Some(0.1),
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::InternalPerimeter,
                Point2::new(0.0, 0.0),
                None,
            ),
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::InternalPerimeter,
                Point2::new(1.0, 0.0),
                Some(0.2),
            ),
        ],
        0.2,
    )];
    let options = SpeedOptions::new(120.0, 60.0, 100.0)
        .with_internal_perimeter_speed(45.0)
        .with_small_perimeter_speed(20.0)
        .with_small_perimeter_threshold(10.0);

    let output = generate_speed_moves(&layers, options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 60.0);
    assert_eq!(output[0].moves()[2].speed_mm_s(), 45.0);
}

fn external_square_moves() -> Vec<ExtrusionMove> {
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
            Point2::new(4.0, 0.0),
            Some(0.1),
        ),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(4.0, 4.0),
            Some(0.2),
        ),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(0.0, 4.0),
            Some(0.3),
        ),
        ExtrusionMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(0.0, 0.0),
            Some(0.4),
        ),
    ]
}
