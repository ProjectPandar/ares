use crate::{
    Contour, LayerContours, PerimeterOptions, Point2, WallDirection, WallSequence,
    generate_gap_fills,
};

#[test]
fn generates_x_axis_rectangular_gap_fill_when_next_loop_collapses_vertically() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 3.0, 0.7)],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    );

    let gap_fills = generate_gap_fills(&layers, options, 30.0).unwrap();

    assert_eq!(gap_fills[0].paths().len(), 1);
    assert_eq!(
        gap_fills[0].paths()[0].points(),
        &[Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
    );
}

#[test]
fn generates_y_axis_rectangular_gap_fill_when_next_loop_collapses_horizontally() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 0.7, 3.0)],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    );

    let gap_fills = generate_gap_fills(&layers, options, 30.0).unwrap();

    assert_eq!(
        gap_fills[0].paths()[0].points(),
        &[Point2::new(0.35, 0.4), Point2::new(0.35, 2.6)]
    );
}

#[test]
fn gap_infill_speed_zero_disables_generation() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![rectangle(0.0, 0.0, 3.0, 0.7)],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    );

    let gap_fills = generate_gap_fills(&layers, options, 0.0).unwrap();

    assert!(gap_fills[0].paths().is_empty());
}

#[test]
fn non_rectangular_contours_generate_no_gap_fill() {
    let layers = [LayerContours::new(
        0,
        0.2,
        vec![Contour::new(vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(1.5, 0.7),
        ])],
    )];
    let options = PerimeterOptions::new(
        4,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::InnerOuter,
    );

    let gap_fills = generate_gap_fills(&layers, options, 30.0).unwrap();

    assert!(gap_fills[0].paths().is_empty());
}

#[test]
fn rectangular_bounds_accepts_rotated_clockwise_rectangle_points() {
    let points = [
        Point2::new(3.0, 0.7),
        Point2::new(3.0, 0.0),
        Point2::new(0.0, 0.0),
        Point2::new(0.0, 0.7),
    ];

    assert_eq!(
        crate::contours::axis_aligned_rectangle_bounds(&points),
        Some((0.0, 0.0, 3.0, 0.7))
    );
}

fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}
