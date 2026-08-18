use super::{Point, Segment, circle_from_three, fit};

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
fn task22o205_circle_center_uses_source_expression_order_at_large_coordinates() {
    let point = |x: i64, y: i64| Point {
        x: x as f64 / 1_000_000.0,
        y: y as f64 / 1_000_000.0,
    };

    let (center, _) = circle_from_three(
        point(153_312_000, 102_774_000),
        point(152_854_553, 103_959_935),
        point(151_357_170, 104_529_875),
    )
    .unwrap();

    assert_eq!(center, point(151_545_998, 102_773_998));
}
