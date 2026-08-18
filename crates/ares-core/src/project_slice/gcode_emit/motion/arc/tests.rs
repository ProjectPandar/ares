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
fn circle_center_truncates_to_scaled_coordinate() {
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
            x: 1.0,
            y: 0.749_999,
        }
    );
}
