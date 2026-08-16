use super::{Point, Segment, fit};

#[test]
fn fits_clockwise_circle_with_analytic_length() {
    let diagonal = 2.0_f64.sqrt() * 0.5;
    let points = [
        Point { x: 1.0, y: 0.0 },
        Point {
            x: diagonal,
            y: -diagonal,
        },
        Point { x: 0.0, y: -1.0 },
    ];

    let fitted = fit(&points, 0.0125);
    let [Segment::Arc(arc)] = fitted.as_slice() else {
        panic!("expected one fitted arc");
    };
    assert!(arc.clockwise);
    assert!((arc.center.x).abs() < 1e-9);
    assert!((arc.center.y).abs() < 1e-9);
    assert!((arc.length - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    assert_eq!(arc.end, points[2]);
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
