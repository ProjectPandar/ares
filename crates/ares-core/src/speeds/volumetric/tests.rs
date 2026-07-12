use crate::{
    ExtrusionMove, LayerExtrusionMoves, Point2, PrintPathRole, SpeedOptions, ToolpathMoveKind,
    generate_speed_moves,
};

#[test]
fn caps_print_speed_from_extrusion_volume_per_mm() {
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
        .with_filament_max_volumetric_speed(1.0);

    let output = generate_speed_moves(&[layer], options);

    let capped = output[0].moves()[1].speed_mm_s();
    assert!((capped - 3.183098861837907).abs() <= 0.000000001);
    assert!((output[0].moves()[1].feedrate_mm_min() - 190.9859317102744).abs() <= 0.0000001);
}

#[test]
fn zero_max_volumetric_speed_does_not_cap_print_or_travel() {
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
        .with_filament_max_volumetric_speed(0.0);

    let output = generate_speed_moves(&[layer], options);

    assert_eq!(output[0].moves()[0].speed_mm_s(), 120.0);
    assert_eq!(output[0].moves()[1].speed_mm_s(), 100.0);
}

#[test]
fn cap_state_uses_previous_print_endpoint_after_consecutive_prints() {
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
            ExtrusionMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(20.0, 0.0),
                Some(1.5),
            ),
        ],
        1.5,
    );
    let options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(1.0);

    let output = generate_speed_moves(&[layer], options);

    let second_print_speed = output[0].moves()[2].speed_mm_s();
    assert!((second_print_speed - 6.366197723675814).abs() <= 0.000000001);
}

#[test]
fn higher_extrusion_delta_from_flow_ratios_lowers_capped_speed() {
    let low_flow = LayerExtrusionMoves::new(
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
                Some(0.5),
            ),
        ],
        0.5,
    );
    let high_flow = LayerExtrusionMoves::new(
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
        .with_filament_max_volumetric_speed(1.0);

    let low_speed = generate_speed_moves(&[low_flow], options).remove(0).moves()[1].speed_mm_s();
    let high_speed = generate_speed_moves(&[high_flow], options)
        .remove(0)
        .moves()[1]
        .speed_mm_s();

    assert!((low_speed - high_speed * 2.0).abs() <= 0.000000001);
}

#[test]
fn adaptive_volumetric_speed_caps_from_move_geometry() {
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
            )
            .with_adaptive_volumetric_geometry(0.2, 0.4),
        ],
        1.0,
    );
    let base_options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(10.0);
    let adaptive_options = base_options
        .with_filament_adaptive_volumetric_speed(true)
        .with_volumetric_speed_coefficients(Some([0.0, 0.0, 0.0, 0.0, 0.0, 1.0]));

    let base_speed = generate_speed_moves(std::slice::from_ref(&layer), base_options)
        .remove(0)
        .moves()[1]
        .speed_mm_s();
    let adaptive_speed = generate_speed_moves(&[layer], adaptive_options)
        .remove(0)
        .moves()[1]
        .speed_mm_s();

    assert!(adaptive_speed < base_speed);
    assert!((adaptive_speed - 3.183098861837907).abs() <= 0.000000001);
}

#[test]
fn adaptive_volumetric_speed_caps_ignore_unusable_or_missing_runtime_values() {
    let move_with_geometry = ExtrusionMove::new(
        ToolpathMoveKind::Print,
        PrintPathRole::ExternalPerimeter,
        Point2::new(10.0, 0.0),
        Some(1.0),
    )
    .with_adaptive_volumetric_geometry(0.2, 0.4);
    let move_without_geometry = ExtrusionMove::new(
        ToolpathMoveKind::Print,
        PrintPathRole::ExternalPerimeter,
        Point2::new(10.0, 0.0),
        Some(1.0),
    );
    let base_options = SpeedOptions::new(120.0, 100.0, 100.0)
        .with_first_layer_speed(100.0)
        .with_filament_diameter(2.0)
        .with_filament_max_volumetric_speed(1.0);
    let baseline = speed_for_single_print(move_with_geometry, base_options);
    let adaptive_false = base_options
        .with_filament_adaptive_volumetric_speed(false)
        .with_volumetric_speed_coefficients(Some([0.0, 0.0, 0.0, 0.0, 0.0, 0.1]));
    let adaptive_omitted_equivalent =
        base_options.with_volumetric_speed_coefficients(Some([0.0, 0.0, 0.0, 0.0, 0.0, 0.1]));

    assert_eq!(
        speed_for_single_print(move_with_geometry, adaptive_false),
        baseline
    );
    assert_eq!(
        speed_for_single_print(move_with_geometry, adaptive_omitted_equivalent),
        baseline
    );

    for options in [
        base_options
            .with_filament_adaptive_volumetric_speed(true)
            .with_volumetric_speed_coefficients(None),
        base_options
            .with_filament_adaptive_volumetric_speed(true)
            .with_volumetric_speed_coefficients(Some([0.0, 0.0, 0.0, 0.0, 0.0, 0.0])),
        base_options
            .with_filament_adaptive_volumetric_speed(true)
            .with_volumetric_speed_coefficients(Some([0.0, 0.0, 0.0, 0.0, 0.0, -1.0])),
        base_options
            .with_filament_adaptive_volumetric_speed(true)
            .with_volumetric_speed_coefficients(Some([f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 1.0])),
        base_options
            .with_filament_adaptive_volumetric_speed(true)
            .with_volumetric_speed_coefficients(Some([f64::NAN, 0.0, 0.0, 0.0, 0.0, 1.0])),
    ] {
        assert_eq!(
            speed_for_single_print(move_with_geometry, options),
            baseline
        );
    }

    let adaptive_without_geometry = base_options
        .with_filament_adaptive_volumetric_speed(true)
        .with_volumetric_speed_coefficients(Some([0.0, 0.0, 0.0, 0.0, 0.0, 0.1]));
    assert_eq!(
        speed_for_single_print(move_without_geometry, adaptive_without_geometry),
        baseline
    );
}

fn speed_for_single_print(print_move: ExtrusionMove, options: SpeedOptions) -> f64 {
    let layer = LayerExtrusionMoves::new(
        0,
        0.2,
        vec![
            ExtrusionMove::new(
                ToolpathMoveKind::Travel,
                print_move.role(),
                Point2::new(0.0, 0.0),
                None,
            ),
            print_move,
        ],
        1.0,
    );
    generate_speed_moves(&[layer], options).remove(0).moves()[1].speed_mm_s()
}
