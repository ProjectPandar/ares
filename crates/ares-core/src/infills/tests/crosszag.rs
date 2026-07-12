use super::*;

fn square_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(layer_id, print_z, square_layer().contours().to_vec())
}

fn path_points(infills: &[LayerInfills], layer_index: usize) -> Vec<&[Point2]> {
    infills[layer_index]
        .paths()
        .iter()
        .map(InfillPath::points)
        .collect()
}

#[test]
fn crosszag_layer_zero_matches_zigzag_single_line_shape() {
    let layers = vec![square_layer()];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::CrossZag),
    )
    .unwrap();

    assert_eq!(
        path_points(&infills, 0),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)][..],
            &[Point2::new(1.5, 2.0), Point2::new(1.5, 0.0)][..],
        ]
    );
}

#[test]
fn crosszag_infill_shift_step_offsets_layer_two_scanlines() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];
    let options = options(InfillPattern::CrossZag).with_infill_shift_step_for_tests(0.25);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        path_points(&infills, 2),
        vec![
            &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)][..],
            &[Point2::new(1.25, 2.0), Point2::new(1.25, 0.0)][..],
        ]
    );
}

#[test]
fn crosszag_infill_shift_step_uses_layer_id_not_vector_index() {
    let layers = vec![square_layer_with_id(2, 0.6)];
    let options = options(InfillPattern::CrossZag).with_infill_shift_step_for_tests(0.25);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        path_points(&infills, 0),
        vec![
            &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)][..],
            &[Point2::new(1.25, 2.0), Point2::new(1.25, 0.0)][..],
        ]
    );
}

#[test]
fn infill_shift_step_does_not_affect_non_crosszag_patterns() {
    for pattern in [
        InfillPattern::Rectilinear,
        InfillPattern::AlignedRectilinear,
        InfillPattern::Line,
        InfillPattern::Grid,
        InfillPattern::ZigZag,
        InfillPattern::CrossHatch,
    ] {
        let layers = vec![
            square_layer_with_id(0, 0.2),
            square_layer_with_id(1, 0.4),
            square_layer_with_id(2, 0.6),
        ];
        let baseline = generate_infills(&print_layers(&layers), &layers, options(pattern)).unwrap();
        let shifted = generate_infills(
            &print_layers(&layers),
            &layers,
            options(pattern).with_infill_shift_step_for_tests(0.25),
        )
        .unwrap();

        assert_eq!(shifted, baseline, "{pattern:?}");
    }
}
