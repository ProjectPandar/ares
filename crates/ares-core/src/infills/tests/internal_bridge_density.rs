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

fn two_contour_layer_with_id(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![
            Contour::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(0.6, 0.0),
                Point2::new(0.6, 0.6),
                Point2::new(0.0, 0.6),
            ]),
            Contour::new(vec![
                Point2::new(3.0, 0.0),
                Point2::new(7.0, 0.0),
                Point2::new(7.0, 4.0),
                Point2::new(3.0, 4.0),
            ]),
        ],
    )
}

fn dense_middle_options() -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
}

fn sparse_middle_options() -> InfillOptions {
    InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
}

fn no_shell_dense_options() -> InfillOptions {
    InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(0, 0)
}

fn assert_middle_paths_have_role(infills: &[LayerInfills], role: InfillRole) {
    assert!(!infills[1].paths().is_empty());
    assert!(
        infills[1].paths().iter().all(|path| path.role() == role),
        "{:?}",
        infills[1].paths()
    );
}

#[test]
fn default_dense_middle_layer_stays_solid_infill() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];

    let infills =
        generate_infills(&print_layers(&layers), &layers, dense_middle_options()).unwrap();

    assert_eq!(infills[0].paths().len(), 4);
    assert_eq!(infills[1].paths().len(), 4);
    assert_eq!(infills[2].paths().len(), 4);
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
    assert!(
        infills[2]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn lower_internal_bridge_density_reduces_only_dense_middle_layer_lines() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 4.0, 4.0)),
    ];
    let options = dense_middle_options().with_internal_bridge_density_for_tests(50.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 8);
    assert_eq!(infills[1].paths().len(), 4);
    assert_eq!(infills[2].paths().len(), 8);
    assert_middle_paths_have_role(&infills, InfillRole::InternalBridge);
}

#[test]
fn disabled_filter_keeps_small_dense_middle_layer_solid() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 0.9, 0.9)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 0.9, 0.9)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 0.9, 0.9)),
    ];
    let options = dense_middle_options().with_internal_bridge_density_for_tests(50.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_middle_paths_have_role(&infills, InfillRole::Solid);
}

#[test]
fn disabled_filter_keeps_large_dense_middle_layer_internal_bridge() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 4.0, 4.0)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 4.0, 4.0)),
    ];
    let options = dense_middle_options().with_internal_bridge_density_for_tests(50.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_middle_paths_have_role(&infills, InfillRole::InternalBridge);
}

#[test]
fn limited_filter_converts_small_layer_that_disabled_filters() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 1.5, 1.5)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 1.5, 1.5)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 1.5, 1.5)),
    ];
    let options = dense_middle_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_filter_for_tests("limited");

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_middle_paths_have_role(&infills, InfillRole::InternalBridge);
}

#[test]
fn limited_filter_still_filters_tiny_layer() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 0.8, 0.8)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 0.8, 0.8)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 0.8, 0.8)),
    ];
    let options = dense_middle_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_filter_for_tests("limited");

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_middle_paths_have_role(&infills, InfillRole::Solid);
}

#[test]
fn nofilter_converts_tiny_layer() {
    let layers = vec![
        rectangle_layer_with_id(0, 0.2, (0.0, 0.0, 0.8, 0.8)),
        rectangle_layer_with_id(1, 0.4, (0.0, 0.0, 0.8, 0.8)),
        rectangle_layer_with_id(2, 0.6, (0.0, 0.0, 0.8, 0.8)),
    ];
    let options = dense_middle_options()
        .with_internal_bridge_density_for_tests(50.0)
        .with_internal_bridge_filter_for_tests("nofilter");

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_middle_paths_have_role(&infills, InfillRole::InternalBridge);
}

#[test]
fn mixed_contours_use_largest_span_for_whole_layer_decision() {
    let layers = vec![
        two_contour_layer_with_id(0, 0.2),
        two_contour_layer_with_id(1, 0.4),
        two_contour_layer_with_id(2, 0.6),
    ];
    let options = dense_middle_options().with_internal_bridge_density_for_tests(50.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_middle_paths_have_role(&infills, InfillRole::InternalBridge);
}

#[test]
fn internal_bridge_density_does_not_change_sparse_middle_spacing() {
    let layers = vec![
        square_layer_with_id(0, 0.2),
        square_layer_with_id(1, 0.4),
        square_layer_with_id(2, 0.6),
    ];
    let default = generate_infills(
        &print_layers(&layers),
        &layers,
        sparse_middle_options().with_internal_bridge_density_for_tests(100.0),
    )
    .unwrap();
    let lower = generate_infills(
        &print_layers(&layers),
        &layers,
        sparse_middle_options().with_internal_bridge_density_for_tests(50.0),
    )
    .unwrap();

    assert_eq!(default[1].paths().len(), 2);
    assert_eq!(lower[1].paths().len(), 2);
    assert!(
        lower[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
}

#[test]
fn internal_bridge_density_does_not_reclassify_no_shell_dense_layer() {
    let layers = vec![square_layer_with_id(0, 0.2)];
    let default = generate_infills(
        &print_layers(&layers),
        &layers,
        no_shell_dense_options().with_internal_bridge_density_for_tests(100.0),
    )
    .unwrap();
    let lower = generate_infills(
        &print_layers(&layers),
        &layers,
        no_shell_dense_options().with_internal_bridge_density_for_tests(50.0),
    )
    .unwrap();

    assert_eq!(default[0].paths().len(), 4);
    assert_eq!(lower[0].paths().len(), 4);
    assert!(
        lower[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}
