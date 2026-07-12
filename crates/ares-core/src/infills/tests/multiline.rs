use super::*;

fn four_mm_square_layer() -> LayerContours {
    layer_zero(vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(4.0, 4.0),
        Point2::new(0.0, 4.0),
    ])
}

fn path_segments(infills: &LayerInfills) -> Vec<(Point2, Point2)> {
    infills
        .paths()
        .iter()
        .map(|path| (path.points()[0], path.points()[1]))
        .collect()
}

fn sorted_path_segments(infills: &LayerInfills) -> Vec<(Point2, Point2)> {
    let mut segments = path_segments(infills);
    segments.sort_by(|left, right| {
        compare_points(left.0, right.0).then_with(|| compare_points(left.1, right.1))
    });
    segments
}

#[test]
fn rectilinear_fill_multiline_uses_orca_source_spacing_before_expansion() {
    let layers = vec![four_mm_square_layer()];
    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Rectilinear).with_fill_multiline_for_tests(3),
    )
    .unwrap();

    assert_eq!(
        path_segments(&infills[0]),
        vec![
            (Point2::new(1.0, 0.0), Point2::new(1.0, 4.0)),
            (Point2::new(1.5, 0.0), Point2::new(1.5, 4.0)),
            (Point2::new(2.0, 0.0), Point2::new(2.0, 4.0)),
        ]
    );
}

#[test]
fn rectilinear_fill_multiline_drops_neighbors_outside_scanline_bounds() {
    let layers = vec![four_mm_square_layer()];
    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Rectilinear).with_fill_multiline_for_tests(7),
    )
    .unwrap();

    assert_eq!(
        path_segments(&infills[0]),
        vec![
            (Point2::new(2.0, 0.0), Point2::new(2.0, 4.0)),
            (Point2::new(2.5, 0.0), Point2::new(2.5, 4.0)),
            (Point2::new(3.0, 0.0), Point2::new(3.0, 4.0)),
            (Point2::new(3.5, 0.0), Point2::new(3.5, 4.0)),
        ]
    );
}

#[test]
fn grid_fill_multiline_expands_both_perpendicular_pass_families() {
    let layers = vec![four_mm_square_layer()];
    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Grid).with_fill_multiline_for_tests(3),
    )
    .unwrap();

    assert_eq!(
        sorted_path_segments(&infills[0]),
        vec![
            (Point2::new(0.0, 1.0), Point2::new(4.0, 1.0)),
            (Point2::new(0.0, 1.5), Point2::new(4.0, 1.5)),
            (Point2::new(0.0, 2.0), Point2::new(4.0, 2.0)),
            (Point2::new(1.0, 0.0), Point2::new(1.0, 4.0)),
            (Point2::new(1.5, 0.0), Point2::new(1.5, 4.0)),
            (Point2::new(2.0, 0.0), Point2::new(2.0, 4.0)),
        ]
    );
}

fn three_square_layers() -> Vec<LayerContours> {
    vec![
        LayerContours::new(0, 0.2, four_mm_square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, four_mm_square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, four_mm_square_layer().contours().to_vec()),
    ]
}

#[test]
fn fill_multiline_does_not_expand_solid_bottom_or_top_surface_roles() {
    let layers = three_square_layers();
    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(100.0, 0.0, 0.5)
            .with_minimum_sparse_infill_area_for_tests(0.0)
            .with_fill_multiline_for_tests(3)
            .with_shell_layers_for_tests(1, 1)
            .with_bottom_surface_pattern_for_tests(InfillPattern::Rectilinear)
            .with_internal_solid_infill_pattern_for_tests(InfillPattern::Rectilinear)
            .with_top_surface_pattern_for_tests(InfillPattern::Rectilinear),
    )
    .unwrap();

    assert_eq!(path_segments(&infills[0]).len(), 8);
    assert_eq!(path_segments(&infills[1]).len(), 8);
    assert_eq!(path_segments(&infills[2]).len(), 8);
    assert!(path_segments(&infills[0]).contains(&(Point2::new(0.25, 0.0), Point2::new(0.25, 4.0))));
    assert!(path_segments(&infills[1]).contains(&(Point2::new(4.0, 1.25), Point2::new(0.0, 1.25))));
    assert!(path_segments(&infills[2]).contains(&(Point2::new(3.75, 0.0), Point2::new(3.75, 4.0))));
}

#[test]
fn fill_multiline_does_not_expand_internal_bridge_role() {
    let layers = vec![
        LayerContours::new(0, 0.2, four_mm_square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, four_mm_square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, four_mm_square_layer().contours().to_vec()),
    ];
    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(100.0, 0.0, 0.5)
            .with_minimum_sparse_infill_area_for_tests(0.0)
            .with_fill_multiline_for_tests(3)
            .with_shell_layers_for_tests(1, 1)
            .with_internal_solid_infill_pattern_for_tests(InfillPattern::Rectilinear)
            .with_internal_bridge_density_for_tests(50.0),
    )
    .unwrap();

    assert_eq!(path_segments(&infills[1]).len(), 4);
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::InternalBridge)
    );
}
