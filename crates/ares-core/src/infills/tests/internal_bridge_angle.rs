use super::*;

fn square_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(layer_id, print_z, square_layer().contours().to_vec())
}

fn rectangle_layer_with_id(
    layer_id: usize,
    print_z: f64,
    bounds: (f64, f64, f64, f64),
) -> LayerContours {
    let (min_x, min_y, max_x, max_y) = bounds;
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ])],
    )
}

fn dense_middle_options() -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_solid_infill_rotate_template_for_tests(vec![0.0])
}

fn sparse_middle_options() -> InfillOptions {
    InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_sparse_infill_rotate_template_for_tests(vec![0.0])
}

fn no_shell_dense_options() -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(0, 0)
        .with_solid_infill_rotate_template_for_tests(vec![0.0])
}

fn three_square_layers() -> Vec<LayerContours> {
    vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ]
}

fn middle_layer_points(infills: &[LayerInfills]) -> Vec<&[Point2]> {
    infills[1].paths().iter().map(InfillPath::points).collect()
}

#[test]
fn positive_internal_bridge_angle_rotates_eligible_internal_bridge_lines() {
    let layers = three_square_layers();
    let options = dense_middle_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_filter_for_tests("nofilter")
        .with_internal_bridge_angle_for_tests(90.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        middle_layer_points(&infills),
        vec![
            &[Point2::new(2.0, 0.5), Point2::new(0.0, 0.5)][..],
            &[Point2::new(2.0, 1.5), Point2::new(0.0, 1.5)][..],
        ]
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::InternalBridge)
    );
}

#[test]
fn zero_internal_bridge_angle_auto_detects_non_square_internal_bridge_direction() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 4.0, 2.0)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 4.0, 2.0)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 4.0, 2.0)),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_solid_infill_rotate_template_for_tests(vec![0.0])
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_filter_for_tests("nofilter")
        .with_internal_bridge_angle_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        middle_layer_points(&infills),
        vec![
            &[Point2::new(4.0, 0.4), Point2::new(0.0, 0.4)][..],
            &[Point2::new(4.0, 1.2), Point2::new(0.0, 1.2)][..],
            &[Point2::new(4.0, 2.0), Point2::new(0.0, 2.0)][..],
        ]
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::InternalBridge)
    );
}

#[test]
fn zero_internal_bridge_angle_preserves_current_direction_for_square_bounds() {
    let layers = three_square_layers();
    let options = dense_middle_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_filter_for_tests("nofilter")
        .with_internal_bridge_angle_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        middle_layer_points(&infills),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)][..],
            &[Point2::new(1.5, 0.0), Point2::new(1.5, 2.0)][..],
        ]
    );
}

#[test]
fn internal_bridge_angle_does_not_create_internal_bridge_at_default_density() {
    let layers = three_square_layers();
    let options = dense_middle_options().with_internal_bridge_angle_for_tests(90.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[1].paths().len(), 4);
    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn internal_bridge_angle_does_not_change_sparse_middle_or_no_shell_dense_layers() {
    let layers = three_square_layers();
    let sparse_options = sparse_middle_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_angle_for_tests(90.0);
    let sparse = generate_infills(&print_layers(&layers), &layers, sparse_options).unwrap();

    assert_eq!(
        middle_layer_points(&sparse),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)][..],
            &[Point2::new(1.5, 0.0), Point2::new(1.5, 2.0)][..],
        ]
    );
    assert!(
        sparse[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );

    let no_shell_layers = vec![square_layer_with_id(0, 0.2)];
    let no_shell_options = no_shell_dense_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_angle_for_tests(90.0);
    let no_shell = generate_infills(
        &print_layers(&no_shell_layers),
        &no_shell_layers,
        no_shell_options,
    )
    .unwrap();

    assert_eq!(
        no_shell[0].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
    assert!(
        no_shell[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}
