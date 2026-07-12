use super::*;

#[test]
fn rectilinear_pattern_rotates_sparse_lines_on_odd_layers() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Rectilinear),
    )
    .unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(2.0, 0.5), Point2::new(0.0, 0.5)]
    );
}

#[test]
fn aligned_rectilinear_pattern_keeps_sparse_lines_aligned_between_layers() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::AlignedRectilinear),
    )
    .unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
}

#[test]
fn crosszag_pattern_keeps_sparse_lines_aligned_between_layers() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::CrossZag),
    )
    .unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
}

#[test]
fn line_zigzag_and_crosshatch_scaffolds_rotate_sparse_lines_on_odd_layers() {
    for pattern in [
        InfillPattern::Line,
        InfillPattern::ZigZag,
        InfillPattern::CrossHatch,
    ] {
        let layers = vec![
            LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
            LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        ];

        let infills = generate_infills(&print_layers(&layers), &layers, options(pattern)).unwrap();

        assert_eq!(
            infills[0].paths()[0].points(),
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)],
            "{pattern:?} layer 0"
        );
        assert_eq!(
            infills[1].paths()[0].points(),
            &[Point2::new(2.0, 0.5), Point2::new(0.0, 0.5)],
            "{pattern:?} layer 1"
        );
    }
}

#[test]
fn grid_pattern_keeps_perpendicular_passes_on_odd_layers() {
    let layers = vec![LayerContours::new(
        1,
        0.4,
        square_layer().contours().to_vec(),
    )];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Grid),
    )
    .unwrap();
    let mut segments = infills[0]
        .paths()
        .iter()
        .map(|path| {
            let points = path.points();
            (points[0], points[1])
        })
        .collect::<Vec<_>>();
    segments.sort_by(|left, right| {
        compare_points(left.0, right.0).then_with(|| compare_points(left.1, right.1))
    });

    assert_eq!(
        segments,
        vec![
            (Point2::new(0.0, 0.5), Point2::new(2.0, 0.5)),
            (Point2::new(0.0, 1.5), Point2::new(2.0, 1.5)),
            (Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)),
            (Point2::new(1.5, 0.0), Point2::new(1.5, 2.0)),
        ]
    );
}

#[test]
fn sparse_infill_rotate_template_overrides_odd_layer_rotation() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];
    let options = options(InfillPattern::Rectilinear)
        .with_sparse_infill_rotate_template_for_tests(vec![90.0, 0.0]);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
}
