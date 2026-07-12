use super::*;
use crate::{Contour, LayerContours, Point2};

fn options(detect: bool) -> PerimeterOptions {
    PerimeterOptions::new(
        1,
        0.4,
        0.4,
        WallDirection::CounterClockwise,
        WallSequence::OuterInner,
    )
    .with_detect_overhang_wall(detect)
}

#[test]
fn unsupported_second_layer_rectangle_becomes_overhang_when_detection_enabled() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(10.0, 0.0, 14.0, 4.0)]),
    ];

    let perimeters = generate_perimeters(&layers, options(true)).unwrap();

    assert_eq!(perimeters[0].paths()[0].role(), PerimeterRole::External);
    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
}

#[test]
fn disabled_detection_preserves_external_role_for_unsupported_rectangle() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(10.0, 0.0, 14.0, 4.0)]),
    ];

    let perimeters = generate_perimeters(&layers, options(false)).unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::External);
}

#[test]
fn edge_only_contact_counts_as_unsupported() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, vec![rectangle(4.0, 0.0, 8.0, 4.0)]),
    ];

    let perimeters = generate_perimeters(&layers, options(true)).unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::Overhang);
}

#[test]
fn non_rectangular_previous_contour_bounds_support_current_rectangle() {
    let layers = [
        LayerContours::new(
            0,
            0.2,
            vec![Contour::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(4.0, 0.0),
                Point2::new(2.0, 4.0),
            ])],
        ),
        LayerContours::new(1, 0.4, vec![rectangle(1.0, 1.0, 2.0, 2.0)]),
    ];

    let perimeters = generate_perimeters(&layers, options(true)).unwrap();

    assert_eq!(perimeters[1].paths()[0].role(), PerimeterRole::External);
}

#[test]
fn direct_empty_previous_layer_is_not_skipped() {
    let layers = [
        LayerContours::new(0, 0.2, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
        LayerContours::new(1, 0.4, Vec::new()),
        LayerContours::new(2, 0.6, vec![rectangle(0.0, 0.0, 4.0, 4.0)]),
    ];

    let perimeters = generate_perimeters(&layers, options(true)).unwrap();

    assert_eq!(perimeters[2].paths()[0].role(), PerimeterRole::Overhang);
}

fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}
