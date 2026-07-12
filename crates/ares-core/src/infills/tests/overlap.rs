use super::*;

fn rectangle_layer(layer_id: usize) -> LayerContours {
    LayerContours::new(
        layer_id,
        0.2 * (layer_id + 1) as f64,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ])],
    )
}

#[test]
fn middle_sparse_infill_respects_wall_overlap_boundary() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1), rectangle_layer(2)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_sparse_infill_rotate_template_for_tests(vec![0.0])
        .with_shell_layers_for_tests(0, 0)
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(15.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[1].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(1.5, 0.54), Point2::new(1.5, 3.46)]
    }));
}

#[test]
fn zero_overlap_clips_middle_sparse_infill_to_wall_inner_boundary() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1), rectangle_layer(2)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_sparse_infill_rotate_template_for_tests(vec![0.0])
        .with_shell_layers_for_tests(0, 0)
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(0.0)
        .with_top_bottom_infill_wall_overlap_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[1].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(1.5, 0.6), Point2::new(1.5, 3.4)]
    }));
}

#[test]
fn first_sparse_infill_uses_top_bottom_overlap() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1), rectangle_layer(2)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(0, 0)
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(15.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[0].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(1.5, 0.5), Point2::new(1.5, 3.5)]
    }));
}

#[test]
fn top_bottom_solid_infill_uses_top_bottom_overlap() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1), rectangle_layer(2)];
    let options = InfillOptions::new_for_tests(100.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_shell_layers_for_tests(1, 1)
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(0.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[0].paths().iter().any(|path| {
        path.role() == InfillRole::Solid
            && path.points() == [Point2::new(0.6, 0.5), Point2::new(0.6, 3.5)]
    }));
    assert!(infills[2].paths().iter().any(|path| {
        path.role() == InfillRole::Solid
            && path.points() == [Point2::new(0.6, 0.5), Point2::new(0.6, 3.5)]
    }));
}

#[test]
fn zero_wall_loops_preserve_original_infill_boundary() {
    let layers = vec![rectangle_layer(0)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_wall_boundary_for_tests(0, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(15.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[0].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(0.5, 0.0), Point2::new(0.5, 4.0)]
    }));
}

#[test]
fn collapsed_wall_boundary_contributes_no_infill() {
    let layers = vec![LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ])],
    )];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_wall_boundary_for_tests(4, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(0.0)
        .with_top_bottom_infill_wall_overlap_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[0].paths().is_empty());
}

#[test]
fn only_one_wall_first_layer_uses_single_loop_boundary() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_sparse_infill_rotate_template_for_tests(vec![0.0])
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_only_one_wall_first_layer_for_tests()
        .with_infill_wall_overlap_for_tests(0.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[0].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(0.5, 0.1), Point2::new(0.5, 3.9)]
    }));
}

#[test]
fn only_one_wall_top_uses_single_loop_boundary() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1), rectangle_layer(2)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_sparse_infill_rotate_template_for_tests(vec![0.0])
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_only_one_wall_top_for_tests()
        .with_infill_wall_overlap_for_tests(0.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[2].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(0.5, 0.1), Point2::new(0.5, 3.9)]
    }));
}

#[test]
fn alternate_extra_wall_expands_middle_odd_layer_loop_count() {
    let layers = vec![rectangle_layer(0), rectangle_layer(1), rectangle_layer(2)];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_sparse_infill_rotate_template_for_tests(vec![0.0])
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_alternate_extra_wall_for_tests()
        .with_infill_wall_overlap_for_tests(15.0)
        .with_top_bottom_infill_wall_overlap_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert!(infills[1].paths().iter().any(|path| {
        path.role() == InfillRole::Sparse
            && path.points() == [Point2::new(1.5, 0.94), Point2::new(1.5, 3.06)]
    }));
}

#[test]
fn multi_contour_layers_keep_existing_hole_clipping() {
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
                Point2::new(3.0, 1.0),
                Point2::new(3.0, 3.0),
                Point2::new(1.0, 3.0),
            ]),
        ],
    )];
    let options = InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_solid_line_width_for_tests(0.4)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_wall_boundary_for_tests(2, 0.4, 0.4)
        .with_infill_wall_overlap_for_tests(15.0)
        .with_top_bottom_infill_wall_overlap_for_tests(25.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[0]
            .paths()
            .iter()
            .map(InfillPath::points)
            .collect::<Vec<_>>(),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 4.0)][..],
            &[Point2::new(1.5, 0.0), Point2::new(1.5, 1.0)][..],
            &[Point2::new(1.5, 3.0), Point2::new(1.5, 4.0)][..],
            &[Point2::new(2.5, 0.0), Point2::new(2.5, 1.0)][..],
            &[Point2::new(2.5, 3.0), Point2::new(2.5, 4.0)][..],
            &[Point2::new(3.5, 0.0), Point2::new(3.5, 4.0)][..],
        ]
    );
}
