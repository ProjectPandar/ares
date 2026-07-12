use super::*;
#[test]
fn density_100_generates_solid_infill_role() {
    let layers = vec![square_layer()];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(!infills[0].paths().is_empty());
    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn density_100_uses_internal_solid_grid_pattern() {
    let layers = vec![square_layer()];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_pattern_for_tests(InfillPattern::Rectilinear)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[0].paths().iter().any(|path| {
        let points = path.points();
        points[0].y() == points[1].y()
    }));
}

#[test]
fn density_100_uses_solid_rotate_template_not_sparse_template() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_sparse_infill_rotate_template_for_tests(vec![90.0, 90.0])
        .with_solid_infill_rotate_template_for_tests(vec![0.0, 0.0]);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
}

#[test]
fn dense_bottom_shell_uses_bottom_surface_pattern_on_odd_bottom_layer() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Rectilinear)
        .with_bottom_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(2, 0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
}

#[test]
fn dense_top_shell_uses_top_surface_pattern_independently_of_internal_grid() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid)
        .with_top_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(0, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[1].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
    assert_eq!(infills[1].paths().len(), 4);
}

#[test]
fn interior_dense_layer_still_uses_internal_solid_grid_pattern() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid)
        .with_bottom_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_top_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[1].paths().len(), 8);
    assert!(infills[1].paths().iter().any(|path| {
        let points = path.points();
        points[0].y() == points[1].y()
    }));
}

#[test]
fn sparse_density_interior_uses_sparse_pattern_between_solid_shells() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_pattern_for_tests(InfillPattern::Grid)
        .with_bottom_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_top_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
    assert_eq!(infills[1].paths().len(), 4);
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
    );
    assert!(infills[1].paths().iter().any(|path| {
        let points = path.points();
        points[0].y() == points[1].y()
    }));
    assert!(
        infills[2]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn extra_solid_infills_promotes_matching_sparse_layers_to_internal_solid() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_extra_solid_infills_for_tests("2")
        .with_shell_layers_for_tests(0, 0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(
        infills[0]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Sparse)
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
            .all(|path| path.role() == InfillRole::Sparse)
    );
}

#[test]
fn extra_solid_infills_does_not_override_bottom_or_top_shell_roles() {
    let layers = vec![
        LayerContours::new(0, 0.2, square_layer().contours().to_vec()),
        LayerContours::new(1, 0.4, square_layer().contours().to_vec()),
        LayerContours::new(2, 0.6, square_layer().contours().to_vec()),
    ];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_extra_solid_infills_for_tests("1#")
        .with_bottom_surface_pattern_for_tests(InfillPattern::Grid)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Rectilinear)
        .with_top_surface_pattern_for_tests(InfillPattern::AlignedRectilinear)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

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
    assert_eq!(
        infills[2].paths()[0].points(),
        &[Point2::new(0.25, 0.0), Point2::new(0.25, 2.0)]
    );
}

#[test]
fn narrow_internal_solid_rectangles_use_concentric_internal_segments_by_default() {
    let layers = narrow_internal_layers();
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[1]
            .paths()
            .iter()
            .map(InfillPath::points)
            .collect::<Vec<_>>(),
        vec![
            &[Point2::new(0.2, 0.2), Point2::new(3.8, 0.2)][..],
            &[Point2::new(3.8, 0.2), Point2::new(3.8, 0.6)][..],
            &[Point2::new(3.8, 0.6), Point2::new(0.2, 0.6)][..],
            &[Point2::new(0.2, 0.6), Point2::new(0.2, 0.2)][..],
        ]
    );
    assert!(
        infills[1]
            .paths()
            .iter()
            .all(|path| path.role() == InfillRole::Solid)
    );
}

#[test]
fn narrow_internal_solid_overlap_uses_adjusted_boundary() {
    let layers = narrow_internal_layers();
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid)
        .with_wall_boundary_for_tests(1, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(15.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[1]
            .paths()
            .iter()
            .map(InfillPath::points)
            .collect::<Vec<_>>(),
        vec![
            &[Point2::new(0.34, 0.34), Point2::new(3.66, 0.34)][..],
            &[Point2::new(3.66, 0.34), Point2::new(3.66, 0.46)][..],
            &[Point2::new(3.66, 0.46), Point2::new(0.34, 0.46)][..],
            &[Point2::new(0.34, 0.46), Point2::new(0.34, 0.34)][..],
        ]
    );
}

#[test]
fn disabled_narrow_internal_solid_detection_preserves_configured_pattern() {
    let layers = narrow_internal_layers();
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid)
        .with_detect_narrow_internal_solid_infill_for_tests(false)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[1].paths().len(), 12);
    assert!(infills[1].paths().iter().any(|path| {
        let points = path.points();
        points[0].y() == points[1].y()
    }));
}

#[test]
fn exact_narrow_threshold_is_included_for_internal_solid_detection() {
    let layers = narrow_internal_layers();
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Rectilinear)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[1].paths().len(), 4);
}

#[test]
fn narrow_detection_does_not_reroute_bottom_or_top_surface_roles() {
    let layers = narrow_internal_layers();
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_bottom_surface_pattern_for_tests(InfillPattern::Grid)
        .with_top_surface_pattern_for_tests(InfillPattern::Grid)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 12);
    assert_eq!(infills[2].paths().len(), 12);
}

#[test]
fn minimum_sparse_infill_area_suppresses_narrow_internal_solid_before_detection() {
    let layers = narrow_internal_layers();
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.4)
        .with_minimum_sparse_infill_area_for_tests(15.0)
        .with_internal_solid_infill_pattern_for_tests(InfillPattern::Grid)
        .with_shell_layers_for_tests(1, 1);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[1].paths().is_empty());
}

fn narrow_internal_layers() -> Vec<LayerContours> {
    vec![
        narrow_layer(0, 0.2),
        narrow_layer(1, 0.4),
        narrow_layer(2, 0.6),
    ]
}

fn narrow_layer(layer_id: usize, print_z: f64) -> LayerContours {
    LayerContours::new(
        layer_id,
        print_z,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 0.8),
            Point2::new(0.0, 0.8),
        ])],
    )
}
