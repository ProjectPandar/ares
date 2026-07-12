use crate::{BrimOptions, BrimType, Contour, LayerContours, Point2, generate_brims};

#[test]
fn disabled_combine_brims_preserves_per_outer_contour_brims() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 1.0, 1.0), square(3.0, 0.0, 4.0, 1.0)],
    )];

    let output = generate_brims(
        &contours,
        BrimOptions::new(0.4, 0.0, BrimType::OuterOnly),
        0.4,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.4, -0.4),
            Point2::new(1.4, -0.4),
            Point2::new(1.4, 1.4),
            Point2::new(-0.4, 1.4),
        ]
    );
    assert_eq!(
        output[0].paths()[1].points(),
        &[
            Point2::new(2.6, -0.4),
            Point2::new(4.4, -0.4),
            Point2::new(4.4, 1.4),
            Point2::new(2.6, 1.4),
        ]
    );
}

#[test]
fn combine_brims_generates_one_outer_envelope_for_multiple_contours() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 1.0, 1.0), square(3.0, 0.0, 4.0, 1.0)],
    )];
    let options = BrimOptions::new(0.4, 0.1, BrimType::OuterOnly).with_combine_brims(true);

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.5, -0.5),
            Point2::new(4.5, -0.5),
            Point2::new(4.5, 1.5),
            Point2::new(-0.5, 1.5),
        ]
    );
}

#[test]
fn combine_brims_keeps_inner_hole_brims_separate() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![
            square(0.0, 0.0, 4.0, 4.0),
            square(1.0, 1.0, 2.0, 2.0),
            square(6.0, 0.0, 8.0, 2.0),
        ],
    )];
    let options = BrimOptions::new(0.4, 0.0, BrimType::OuterAndInner).with_combine_brims(true);

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert_eq!(output[0].paths().len(), 2);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.4, -0.4),
            Point2::new(8.4, -0.4),
            Point2::new(8.4, 4.4),
            Point2::new(-0.4, 4.4),
        ]
    );
    assert_eq!(
        output[0].paths()[1].points(),
        &[
            Point2::new(1.4, 1.4),
            Point2::new(1.6, 1.4),
            Point2::new(1.6, 1.6),
            Point2::new(1.4, 1.6),
        ]
    );
}

fn square(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}
