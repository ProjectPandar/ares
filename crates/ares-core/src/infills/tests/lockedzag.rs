use super::*;

fn square_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(layer_id, print_z, square_layer().contours().to_vec())
}

fn four_mm_square_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ])],
    )
}

fn path_points(infills: &[LayerInfills], layer_index: usize) -> Vec<&[Point2]> {
    infills[layer_index]
        .paths()
        .iter()
        .map(InfillPath::points)
        .collect()
}

#[test]
fn lockedzag_layer_zero_matches_zigzag_sparse_shape() {
    let layers = vec![square_layer()];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::LockedZag),
    )
    .unwrap();

    assert_eq!(
        path_points(&infills, 0),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)][..],
            &[Point2::new(1.5, 2.0), Point2::new(1.5, 0.0)][..],
        ]
    );
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
}

#[test]
fn lockedzag_keeps_sparse_lines_aligned_between_layers() {
    let layers = vec![square_layer_with_id(0, 0.2), square_layer_with_id(1, 0.4)];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::LockedZag),
    )
    .unwrap();

    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
}

#[test]
fn lockedzag_infill_shift_step_offsets_layer_two_scanlines() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];
    let options = options(InfillPattern::LockedZag).with_infill_shift_step_for_tests(0.25);

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
fn lockedzag_fill_multiline_stays_single_line_branch() {
    let layers = vec![four_mm_square_layer_with_id(0, 0.2)];
    let baseline = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::LockedZag),
    )
    .unwrap();
    let multiline = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::LockedZag).with_fill_multiline_for_tests(3),
    )
    .unwrap();

    assert_eq!(multiline, baseline);
    assert_eq!(multiline[0].paths().len(), 4);
}

#[test]
fn symmetric_infill_y_axis_mirrors_lockedzag_sparse_segments() {
    let layers = vec![LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(3.0, 2.0),
            Point2::new(0.0, 2.0),
        ])],
    )];
    let plain = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::LockedZag),
    )
    .unwrap();
    let mirrored = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::LockedZag).with_symmetric_infill_y_axis_for_tests(true),
    )
    .unwrap();

    assert_eq!(
        plain[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
    assert_eq!(
        mirrored[0].paths()[0].points(),
        &[Point2::new(2.5, 0.0), Point2::new(2.5, 2.0)]
    );
}
