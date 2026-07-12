use super::*;
use crate::{Contour, LayerContours, Point2, SliceError, SliceOptions};
use serde_json::json;

mod alternate_extra_wall;
mod extra_perimeters_on_overhangs;
mod fuzzy_skin;
mod fuzzy_skin_coherent;
mod min_feature_bead_width;
mod min_length_factor;
mod only_one_wall_first_layer;
mod only_one_wall_top;
mod overhang;
mod overhang_reverse;
mod precise_outer_wall;
mod seam_gap;
mod seam_position;
mod staggered_inner_seams;
mod thin_walls;
mod wall_generator;
mod wall_maximum_resolution_deviation;
mod wall_sequence;
mod wall_transition_parameters;

#[test]
fn generates_external_perimeter_for_square_contour() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ])],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            1,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters.len(), 1);
    assert_eq!(perimeters[0].layer_id(), 0);
    assert_eq!(perimeters[0].print_z(), 0.2);
    assert_eq!(perimeters[0].paths().len(), 1);
    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::External);
    assert_eq!(perimeters[0].paths()[0].points().len(), 4);
    assert_eq!(perimeters[0].paths()[0].points()[0], Point2::new(0.0, 0.0));
}

#[test]
fn clockwise_wall_direction_reverses_external_contour_order() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 1.0, 1.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            1,
            0.4,
            0.4,
            WallDirection::Clockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(
        perimeters[0].paths()[0].points(),
        &[
            Point2::new(0.0, 1.0),
            Point2::new(1.0, 1.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.0, 0.0),
        ]
    );
}

#[test]
fn clockwise_wall_direction_reverses_internal_rectangles() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            2,
            0.4,
            0.4,
            WallDirection::Clockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(
        perimeters[0].paths()[0].points(),
        &[
            Point2::new(0.4, 3.6),
            Point2::new(3.6, 3.6),
            Point2::new(3.6, 0.4),
            Point2::new(0.4, 0.4),
        ]
    );
}

#[test]
fn preserves_empty_layers_and_multiple_contours() {
    let layers = [
        LayerContours::new(0, 0.2, Vec::new()),
        LayerContours::new(
            1,
            0.4,
            vec![
                Contour::new(vec![
                    Point2::new(0.0, 0.0),
                    Point2::new(1.0, 0.0),
                    Point2::new(1.0, 1.0),
                ]),
                Contour::new(vec![
                    Point2::new(2.0, 0.0),
                    Point2::new(3.0, 0.0),
                    Point2::new(3.0, 1.0),
                ]),
            ],
        ),
    ];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            1,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert!(perimeters[0].paths().is_empty());
    assert_eq!(perimeters[1].paths().len(), 2);
    assert_eq!(perimeters[1].paths()[0].points()[0], Point2::new(0.0, 0.0));
    assert_eq!(perimeters[1].paths()[1].points()[0], Point2::new(2.0, 0.0));
}

#[test]
fn rejects_malformed_contours() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
        ])],
    )];

    assert!(matches!(
        generate_perimeters(
            &layers,
            PerimeterOptions::new(
                1,
                0.4,
                0.4,
                WallDirection::CounterClockwise,
                WallSequence::InnerOuter,
            )
        ),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn wall_loops_generate_external_and_internal_rectangles() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths().len(), 3);
    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.7570796326794897,
            0.7570796326794897,
            3.2429203673205103,
            3.2429203673205103
        )
    );
    assert_eq!(perimeters[0].paths()[1].role(), PerimeterRole::Internal);
    assert_eq!(
        perimeters[0].paths()[1].points(),
        rectangle_points(0.4, 0.4, 3.6, 3.6)
    );
    assert_eq!(perimeters[0].paths()[2].role(), PerimeterRole::External);
    assert_eq!(
        perimeters[0].paths()[2].points(),
        rectangle_points(0.0, 0.0, 4.0, 4.0)
    );
}

#[test]
fn internal_wall_width_controls_rectangular_internal_spacing() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 6.0, 6.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.2,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths().len(), 3);
    assert_eq!(
        perimeters[0].paths()[0].points(),
        rectangle_points(
            0.4570796326794897,
            0.4570796326794897,
            5.54292036732051,
            5.54292036732051
        )
    );
    assert_eq!(
        perimeters[0].paths()[1].points(),
        rectangle_points(0.3, 0.3, 5.7, 5.7)
    );
}

#[test]
fn wall_loops_skip_collapsed_internal_rectangles() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 0.7, 0.7)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            4,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths().len(), 1);
    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::External);
}

#[test]
fn zero_wall_loops_preserves_layers_without_paths() {
    let layers = [LayerContours::new(
        5,
        1.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            0,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].layer_id(), 5);
    assert_eq!(perimeters[0].print_z(), 1.2);
    assert!(perimeters[0].paths().is_empty());
}

#[test]
fn non_rectangular_contours_keep_only_external_loop() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(2.0, 3.0),
        ])],
    )];

    let perimeters = generate_perimeters(
        &layers,
        PerimeterOptions::new(
            3,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        ),
    )
    .unwrap();

    assert_eq!(perimeters[0].paths().len(), 1);
    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::External);
}

#[test]
fn slice_options_wall_loops_increase_rectangular_perimeter_artifacts() {
    let one_wall: SliceOptions = serde_json::from_value(json!({
        "wall_loops": 1,
        "line_width": 0.4
    }))
    .unwrap();
    let three_walls: SliceOptions = serde_json::from_value(json!({
        "wall_loops": 3,
        "line_width": 0.4
    }))
    .unwrap();
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];

    let one_wall_perimeters =
        generate_perimeters(&layers, one_wall.perimeter_options().unwrap()).unwrap();
    let three_wall_perimeters =
        generate_perimeters(&layers, three_walls.perimeter_options().unwrap()).unwrap();

    assert_eq!(one_wall_perimeters[0].paths().len(), 1);
    assert_eq!(three_wall_perimeters[0].paths().len(), 3);
    assert_eq!(
        three_wall_perimeters[0].paths()[0].role(),
        PerimeterRole::Internal
    );
    assert_eq!(
        three_wall_perimeters[0].paths()[1].role(),
        PerimeterRole::Internal
    );
    assert_eq!(
        three_wall_perimeters[0].paths()[2].role(),
        PerimeterRole::External
    );
}

pub(super) fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(rectangle_points(min_x, min_y, max_x, max_y).to_vec())
}

pub(super) fn rectangle_points(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<Point2> {
    vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ]
}
