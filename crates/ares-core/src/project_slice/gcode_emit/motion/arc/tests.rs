use super::{Point, Segment, arc_direction, circle_from_three, fit};

#[test]
fn fits_clockwise_circle_with_analytic_length() {
    let points = (0..=18)
        .map(|step| {
            let angle = -(step as f64) * std::f64::consts::FRAC_PI_2 / 18.0;
            Point {
                x: angle.cos(),
                y: angle.sin(),
            }
        })
        .collect::<Vec<_>>();

    let fitted = fit(&points, 0.0125);
    let [Segment::Arc(arc)] = fitted.as_slice() else {
        panic!("expected one fitted arc");
    };
    assert!(arc.clockwise);
    assert!(arc.center.x.abs() < 0.0125, "{arc:?}");
    assert!(arc.center.y.abs() < 0.0125, "{arc:?}");
    assert!((arc.length - std::f64::consts::FRAC_PI_2).abs() < 0.005);
    assert_eq!(arc.end, points[18]);
}

#[test]
fn rejects_circle_whose_chords_exceed_fitting_tolerance() {
    let diagonal = 2.0_f64.sqrt() * 0.5;
    let points = [
        Point { x: 1.0, y: 0.0 },
        Point {
            x: diagonal,
            y: -diagonal,
        },
        Point { x: 0.0, y: -1.0 },
    ];

    assert!(
        fit(&points, 0.0125)
            .iter()
            .all(|segment| matches!(segment, Segment::Line { .. }))
    );
}

#[test]
fn keeps_non_circular_path_linear() {
    let points = [
        Point { x: 0.0, y: 0.0 },
        Point { x: 1.0, y: 0.0 },
        Point { x: 2.0, y: 0.1 },
        Point { x: 3.0, y: 1.0 },
    ];

    assert!(
        fit(&points, 0.0125)
            .iter()
            .any(|segment| matches!(segment, Segment::Line { .. }))
    );
}

#[test]
fn circle_center_rounds_to_source_coordinate_grid() {
    let (center, _) = circle_from_three(
        Point { x: 0.0, y: 0.0 },
        Point {
            x: 2.000_001,
            y: 0.0,
        },
        Point { x: 1.0, y: 2.0 },
    )
    .unwrap();

    assert_eq!(
        center,
        Point {
            x: 1.000_001,
            y: 0.75,
        }
    );
}

#[test]
fn rejects_wrapped_arc_with_middle_at_zero_polar_angle() {
    assert_eq!(arc_direction(4.8, 0.0, 1.3), None);
}

#[test]
fn integer_grid_midpoint_on_zero_polar_axis_splits_the_arc() {
    let points = [
        (-1_987_296, -25_907_679),
        (-1_490_170, -25_774_473),
        (-1_023_727, -25_556_970),
        (-602_159, -25_261_777),
        (-238_228, -24_897_846),
        (56_972, -24_476_271),
        (274_474, -24_009_833),
        (407_677, -23_512_704),
        (452_541, -23_000_001),
        (407_677, -22_487_296),
        (274_474, -21_990_167),
        (56_972, -21_523_729),
        (-238_228, -21_102_154),
        (-602_159, -20_738_223),
        (-1_023_727, -20_443_030),
        (-1_490_171, -20_225_526),
        (-1_987_296, -20_092_322),
    ]
    .map(|(x, y)| Point {
        x: x as f64 * 1.0e-6,
        y: y as f64 * 1.0e-6,
    });

    let fitted = fit(&points, 0.012);
    let [Segment::Arc(arc), Segment::Line { end, .. }] = fitted.as_slice() else {
        panic!("expected an arc followed by the zero-axis boundary segment");
    };
    assert_eq!(arc.end, points[15]);
    assert_eq!(*end, points[16]);
}
