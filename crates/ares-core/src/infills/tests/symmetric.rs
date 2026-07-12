use super::*;

fn asymmetric_layer() -> LayerContours {
    layer_zero(vec![
        Point2::new(0.0, 0.0),
        Point2::new(3.0, 0.0),
        Point2::new(3.0, 2.0),
        Point2::new(0.0, 2.0),
    ])
}

#[test]
fn symmetric_infill_y_axis_mirrors_zigzag_sparse_segments() {
    let layers = vec![asymmetric_layer()];
    let plain = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::ZigZag),
    )
    .unwrap();
    let mirrored = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::ZigZag).with_symmetric_infill_y_axis_for_tests(true),
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
    assert_ne!(
        mirrored[0].paths()[0].points(),
        plain[0].paths()[0].points()
    );
}

#[test]
fn symmetric_infill_y_axis_mirrors_crosszag_sparse_segments() {
    let layers = vec![asymmetric_layer()];
    let plain = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::CrossZag),
    )
    .unwrap();
    let mirrored = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::CrossZag).with_symmetric_infill_y_axis_for_tests(true),
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
    assert_ne!(
        mirrored[0].paths()[0].points(),
        plain[0].paths()[0].points()
    );
}

#[test]
fn symmetric_infill_y_axis_does_not_mirror_rectilinear_segments() {
    let layers = vec![asymmetric_layer()];
    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Rectilinear).with_symmetric_infill_y_axis_for_tests(true),
    )
    .unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
}

#[test]
fn symmetric_infill_y_axis_does_not_mirror_non_crosszag_solid_shell() {
    let layers = vec![asymmetric_layer()];
    let options = options(InfillPattern::CrossZag)
        .with_symmetric_infill_y_axis_for_tests(true)
        .with_shell_layers_for_tests(1, 0)
        .with_bottom_surface_pattern_for_tests(InfillPattern::Rectilinear);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths()[0].role(), InfillRole::Solid);
    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
}
