use super::*;
use crate::{Contour, LayerContours, Point2};

mod combine_brims;
mod efc_outline;

#[test]
fn generates_outer_brim_loops_on_first_layer() {
    let contours = [LayerContours::new(0, 0.2, vec![unit_square_contour()])];

    let output = generate_brims(
        &contours,
        BrimOptions::new(1.2, 0.2, BrimType::OuterOnly),
        0.4,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 3);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.6, -0.6),
            Point2::new(1.6, -0.6),
            Point2::new(1.6, 1.6),
            Point2::new(-0.6, 1.6),
        ]
    );
    assert_eq!(
        output[0].paths()[2].points(),
        &[
            Point2::new(-1.4, -1.4),
            Point2::new(2.4, -1.4),
            Point2::new(2.4, 2.4),
            Point2::new(-1.4, 2.4),
        ]
    );
}

#[test]
fn clamps_single_brim_loop_to_requested_width() {
    let contours = [LayerContours::new(0, 0.2, vec![unit_square_contour()])];

    let output = generate_brims(
        &contours,
        BrimOptions::new(0.2, 0.0, BrimType::OuterOnly),
        0.4,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.2, -0.2),
            Point2::new(1.2, -0.2),
            Point2::new(1.2, 1.2),
            Point2::new(-0.2, 1.2),
        ]
    );
}

#[test]
fn brim_ears_max_angle_zero_suppresses_corner_ears() {
    let contours = [LayerContours::new(0, 0.2, vec![unit_square_contour()])];
    let options =
        BrimOptions::new(0.4, 0.0, BrimType::BrimEars).with_brim_ears_max_angle_degrees(0.0);

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert!(output[0].paths().is_empty());
}

#[test]
fn rejects_invalid_or_excessive_brim_line_widths() {
    let contours = [LayerContours::new(0, 0.2, vec![unit_square_contour()])];
    let options = BrimOptions::new(1.2, 0.0, BrimType::OuterOnly);

    assert!(generate_brims(&contours, options, 0.0).is_err());
    assert!(generate_brims(&contours, options, f64::NAN).is_err());
    assert!(generate_brims(&contours, options, 0.00001).is_err());
}

#[test]
fn preserves_layers_and_honors_disabled_or_no_output_types() {
    let contours = vec![
        LayerContours::new(0, 0.2, vec![unit_square_contour()]),
        LayerContours::new(1, 0.4, vec![unit_square_contour()]),
        LayerContours::new(2, 0.6, Vec::new()),
    ];

    let disabled = generate_brims(
        &contours,
        BrimOptions::new(0.0, 0.0, BrimType::OuterOnly),
        0.4,
    )
    .unwrap();
    assert!(disabled.iter().all(|layer| layer.paths().is_empty()));

    for brim_type in [BrimType::NoBrim, BrimType::Painted] {
        let no_output =
            generate_brims(&contours, BrimOptions::new(1.2, 0.0, brim_type), 0.4).unwrap();
        assert!(no_output.iter().all(|layer| layer.paths().is_empty()));
    }
}

#[test]
fn inner_only_generates_brim_loops_inside_holes() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 4.0, 4.0), square(1.0, 1.0, 3.0, 3.0)],
    )];

    let output = generate_brims(
        &contours,
        BrimOptions::new(0.8, 0.0, BrimType::InnerOnly),
        0.4,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(1.4, 1.4),
            Point2::new(2.6, 1.4),
            Point2::new(2.6, 2.6),
            Point2::new(1.4, 2.6),
        ]
    );
    assert_eq!(
        output[0].paths()[1].points(),
        &[
            Point2::new(1.8, 1.8),
            Point2::new(2.2, 1.8),
            Point2::new(2.2, 2.2),
            Point2::new(1.8, 2.2),
        ]
    );
}

#[test]
fn outer_and_inner_generates_outer_bounds_and_inner_hole_brims() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 4.0, 4.0), square(1.0, 1.0, 3.0, 3.0)],
    )];

    let output = generate_brims(
        &contours,
        BrimOptions::new(0.4, 0.0, BrimType::OuterAndInner),
        0.4,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.4, -0.4),
            Point2::new(4.4, -0.4),
            Point2::new(4.4, 4.4),
            Point2::new(-0.4, 4.4),
        ]
    );
    assert_eq!(
        output[0].paths()[1].points(),
        &[
            Point2::new(1.4, 1.4),
            Point2::new(2.6, 1.4),
            Point2::new(2.6, 2.6),
            Point2::new(1.4, 2.6),
        ]
    );
}

#[test]
fn painted_and_no_brim_remain_empty() {
    let contours = [LayerContours::new(0, 0.2, vec![unit_square_contour()])];

    for brim_type in [BrimType::Painted, BrimType::NoBrim] {
        let output = generate_brims(&contours, BrimOptions::new(0.8, 0.0, brim_type), 0.4).unwrap();
        assert!(output[0].paths().is_empty());
    }
}

fn unit_square_contour() -> Contour {
    square(0.0, 0.0, 1.0, 1.0)
}

fn square(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}
