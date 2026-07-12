use super::*;
use crate::{Contour, InfillOptions, InfillPattern, Layer, LayerContours, Point2, SliceError};

mod anchor;
mod bridge_angle;
mod bridge_density;
mod calibration;
mod combination;
mod concentric;
mod crosszag;
mod elephant_foot_density;
mod internal_bridge_angle;
mod internal_bridge_density;
mod internal_solid;
mod layer_angles;
mod lockedzag;
mod multiline;
mod overlap;
mod sparse_shell;
mod spiral_vase;
mod surface_density;
mod symmetric;
mod top_surface_width;

pub(super) fn layer(points: Vec<Point2>) -> LayerContours {
    LayerContours::new(7, 0.4, vec![Contour::new(points)])
}

fn layer_zero(points: Vec<Point2>) -> LayerContours {
    LayerContours::new(0, 0.4, vec![Contour::new(points)])
}

pub(super) fn square_layer() -> LayerContours {
    layer_zero(vec![
        Point2::new(0.0, 0.0),
        Point2::new(2.0, 0.0),
        Point2::new(2.0, 2.0),
        Point2::new(0.0, 2.0),
    ])
}

pub(super) fn print_layers(layers: &[LayerContours]) -> Vec<Layer> {
    layers
        .iter()
        .map(|layer| Layer::new(layer.layer_id(), 0.2, layer.print_z()))
        .collect()
}

pub(super) fn options(pattern: InfillPattern) -> InfillOptions {
    InfillOptions::new_for_tests(50.0, 0.0, 0.5)
        .with_minimum_sparse_infill_area_for_tests(0.0)
        .with_pattern_for_tests(pattern)
}

#[test]
fn density_zero_preserves_empty_infill_layers() {
    let layers = vec![layer(vec![
        Point2::new(0.0, 0.0),
        Point2::new(2.0, 0.0),
        Point2::new(2.0, 2.0),
        Point2::new(0.0, 2.0),
    ])];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(0.0, 0.0, 0.5),
    )
    .unwrap();

    assert_eq!(infills.len(), 1);
    assert_eq!(infills[0].layer_id(), 7);
    assert_eq!(infills[0].print_z(), 0.4);
    assert!(infills[0].paths().is_empty());
}

#[test]
fn generates_axis_aligned_sparse_lines_inside_square() {
    let layers = vec![square_layer()];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(50.0, 0.0, 0.5).with_minimum_sparse_infill_area_for_tests(0.0),
    )
    .unwrap();

    assert_eq!(infills[0].paths().len(), 2);
    assert_eq!(infills[0].paths()[0].role(), InfillRole::Sparse);
    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
    assert_eq!(
        infills[0].paths()[1].points(),
        &[Point2::new(1.5, 0.0), Point2::new(1.5, 2.0)]
    );
}

#[test]
fn rectilinear_pattern_keeps_single_sparse_pass() {
    let layers = vec![square_layer()];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Rectilinear),
    )
    .unwrap();

    assert_eq!(
        infills[0]
            .paths()
            .iter()
            .map(InfillPath::points)
            .collect::<Vec<_>>(),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)][..],
            &[Point2::new(1.5, 0.0), Point2::new(1.5, 2.0)][..],
        ]
    );
}

#[test]
fn zigzag_pattern_keeps_scanlines_and_alternates_segment_direction() {
    let layers = vec![square_layer()];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::ZigZag),
    )
    .unwrap();

    assert_eq!(
        infills[0]
            .paths()
            .iter()
            .map(InfillPath::points)
            .collect::<Vec<_>>(),
        vec![
            &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)][..],
            &[Point2::new(1.5, 2.0), Point2::new(1.5, 0.0)][..],
        ]
    );
}

#[test]
fn grid_pattern_adds_perpendicular_sparse_pass() {
    let layers = vec![square_layer()];

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
fn clips_sparse_lines_out_of_inner_holes() {
    let layers = vec![LayerContours::new(
        0,
        0.4,
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

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(50.0, 0.0, 0.5).with_minimum_sparse_infill_area_for_tests(0.0),
    )
    .unwrap();

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

#[test]
fn default_minimum_sparse_infill_area_suppresses_tiny_sparse_regions() {
    let layers = vec![LayerContours::new(
        7,
        0.4,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(2.0, 0.0),
            Point2::new(2.0, 2.0),
            Point2::new(0.0, 2.0),
        ])],
    )];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(50.0, 0.0, 0.5),
    )
    .unwrap();

    assert!(infills[0].paths().is_empty());
}

#[test]
fn minimum_sparse_infill_area_zero_preserves_tiny_sparse_regions() {
    let layers = vec![square_layer()];
    let options =
        InfillOptions::new_for_tests(50.0, 0.0, 0.5).with_minimum_sparse_infill_area_for_tests(0.0);

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 2);
}

#[test]
fn minimum_sparse_infill_area_keeps_regions_above_threshold() {
    let layers = vec![LayerContours::new(
        0,
        0.4,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ])],
    )];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(50.0, 0.0, 0.5),
    )
    .unwrap();

    assert_eq!(infills[0].paths().len(), 4);
}

#[test]
fn minimum_sparse_infill_area_preserves_hole_clipping_when_filled_area_is_above_threshold() {
    let layers = vec![LayerContours::new(
        0,
        0.4,
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
        .with_minimum_sparse_infill_area_for_tests(10.0);

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

#[test]
fn generates_diagonal_sparse_line_inside_diamond() {
    let layers = vec![layer_zero(vec![
        Point2::new(-1.0, 0.0),
        Point2::new(0.0, -1.0),
        Point2::new(1.0, 0.0),
        Point2::new(0.0, 1.0),
    ])];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(50.0, 45.0, 0.5)
            .with_minimum_sparse_infill_area_for_tests(0.0),
    )
    .unwrap();

    assert_eq!(infills[0].paths().len(), 2);
    assert_eq!(
        infills[0].paths()[0].points(),
        &[
            Point2::new(0.146447, -0.853553),
            Point2::new(-0.853553, 0.146447),
        ]
    );
    assert_eq!(
        infills[0].paths()[1].points(),
        &[
            Point2::new(0.853553, -0.146447),
            Point2::new(-0.146447, 0.853553),
        ]
    );
}

#[test]
fn rejects_malformed_infill_contours() {
    let layers = vec![layer(vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)])];

    let result = generate_infills(
        &print_layers(&layers),
        &layers,
        InfillOptions::new_for_tests(50.0, 0.0, 0.5),
    );

    assert!(matches!(result, Err(SliceError::InvalidInput(_))));
}
