use super::*;

fn three_square_layers() -> Vec<LayerContours> {
    (0..3)
        .map(|id| {
            LayerContours::new(
                id,
                0.2 * (id + 1) as f64,
                vec![Contour::new(vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(2.0, 0.0),
                    Point2::new(2.0, 2.0),
                    Point2::new(0.0, 2.0),
                ])],
            )
        })
        .collect()
}

fn calibration_options_with_top_pattern(pattern: InfillPattern, enabled: bool) -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_bottom_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_top_surface_pattern_for_tests(pattern)
        .with_calib_flowrate_topinfill_special_order_for_tests(enabled)
}

fn layer_segments(infills: &[LayerInfills], layer_index: usize) -> Vec<Vec<Point2>> {
    infills[layer_index]
        .paths()
        .iter()
        .map(|path| path.points().to_vec())
        .collect()
}

fn reversed_segments(segments: &[Vec<Point2>]) -> Vec<Vec<Point2>> {
    segments
        .iter()
        .map(|segment| vec![segment[1], segment[0]])
        .collect()
}

#[test]
fn calibration_reverses_only_aligned_top_surface_segments() {
    let layers = three_square_layers();
    let disabled = generate_infills(
        &print_layers(&layers),
        &layers,
        calibration_options_with_top_pattern(InfillPattern::AlignedRectilinear, false),
    )
    .unwrap();
    let enabled = generate_infills(
        &print_layers(&layers),
        &layers,
        calibration_options_with_top_pattern(InfillPattern::AlignedRectilinear, true),
    )
    .unwrap();

    assert_eq!(
        layer_segments(&enabled, 2),
        reversed_segments(&layer_segments(&disabled, 2))
    );
    assert_eq!(layer_segments(&enabled, 0), layer_segments(&disabled, 0));
    assert_eq!(layer_segments(&enabled, 1), layer_segments(&disabled, 1));
}

#[test]
fn calibration_keeps_zigzag_alternation_then_xors_top_surface_reversal() {
    let layers = three_square_layers();
    let disabled = generate_infills(
        &print_layers(&layers),
        &layers,
        calibration_options_with_top_pattern(InfillPattern::ZigZag, false),
    )
    .unwrap();
    let enabled = generate_infills(
        &print_layers(&layers),
        &layers,
        calibration_options_with_top_pattern(InfillPattern::ZigZag, true),
    )
    .unwrap();

    let disabled_top = layer_segments(&disabled, 2);
    assert_eq!(
        disabled_top,
        vec![
            vec![Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)],
            vec![Point2::new(0.75, 2.0), Point2::new(0.75, 0.0)],
            vec![Point2::new(1.25, 0.0), Point2::new(1.25, 2.0)],
            vec![Point2::new(1.75, 2.0), Point2::new(1.75, 0.0)],
        ]
    );
    assert_eq!(
        layer_segments(&enabled, 2),
        reversed_segments(&disabled_top)
    );
    assert_eq!(layer_segments(&enabled, 0), layer_segments(&disabled, 0));
    assert_eq!(layer_segments(&enabled, 1), layer_segments(&disabled, 1));
}
