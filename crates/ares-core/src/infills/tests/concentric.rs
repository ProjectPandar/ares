use super::*;

#[test]
fn concentric_internal_solid_rectangle_generates_outside_in_loops() {
    let layers = vec![LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ])],
    )];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_detect_narrow_internal_solid_infill_for_tests(false)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Concentric);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[0]
            .paths()
            .iter()
            .map(InfillPath::points)
            .collect::<Vec<_>>(),
        vec![
            &[Point2::new(0.25, 0.25), Point2::new(3.75, 0.25)][..],
            &[Point2::new(3.75, 0.25), Point2::new(3.75, 3.75)][..],
            &[Point2::new(3.75, 3.75), Point2::new(0.25, 3.75)][..],
            &[Point2::new(0.25, 3.75), Point2::new(0.25, 0.25)][..],
            &[Point2::new(0.75, 0.75), Point2::new(3.25, 0.75)][..],
            &[Point2::new(3.25, 0.75), Point2::new(3.25, 3.25)][..],
            &[Point2::new(3.25, 3.25), Point2::new(0.75, 3.25)][..],
            &[Point2::new(0.75, 3.25), Point2::new(0.75, 0.75)][..],
            &[Point2::new(1.25, 1.25), Point2::new(2.75, 1.25)][..],
            &[Point2::new(2.75, 1.25), Point2::new(2.75, 2.75)][..],
            &[Point2::new(2.75, 2.75), Point2::new(1.25, 2.75)][..],
            &[Point2::new(1.25, 2.75), Point2::new(1.25, 1.25)][..],
            &[Point2::new(1.75, 1.75), Point2::new(2.25, 1.75)][..],
            &[Point2::new(2.25, 1.75), Point2::new(2.25, 2.25)][..],
            &[Point2::new(2.25, 2.25), Point2::new(1.75, 2.25)][..],
            &[Point2::new(1.75, 2.25), Point2::new(1.75, 1.75)][..],
        ]
    );
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn concentric_bottom_and_top_surface_patterns_use_surface_roles() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Rectilinear)
        .with_bottom_surface_pattern_for_tests(InfillPattern::Concentric)
        .with_top_surface_pattern_for_tests(InfillPattern::Concentric)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.25, 0.25), Point2::new(1.75, 0.25)]
    );
    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.25, 0.25), Point2::new(1.75, 0.25)]
    );
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn concentric_non_rectangular_contour_returns_rectangle_only_error() {
    let layers = vec![LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(3.0, 2.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ])],
    )];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Concentric);

    let err = generate_infills(&print_layers(&layers), &layers, options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("concentric"), "{err}");
    assert!(err.to_string().contains("rectangle"), "{err}");
}

#[test]
fn concentric_multi_contour_layer_returns_rectangle_only_error() {
    let layers = vec![LayerContours::new(
        0,
        0.2,
        vec![
            Contour::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(4.0, 0.0),
                Point2::new(4.0, 4.0),
                Point2::new(0.0, 4.0),
            ]),
            Contour::new(vec![
                Point2::new(1.0, 1.0),
                Point2::new(2.0, 1.0),
                Point2::new(2.0, 2.0),
                Point2::new(1.0, 2.0),
            ]),
        ],
    )];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Concentric);

    let err = generate_infills(&print_layers(&layers), &layers, options).unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("concentric"), "{err}");
    assert!(err.to_string().contains("rectangle"), "{err}");
}
