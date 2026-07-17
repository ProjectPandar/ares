use crate::geometry::{Coord, CoordinateScale, Point};
use crate::{Point2d, Point2dList};

fn normal_scale() -> CoordinateScale {
    CoordinateScale::from_printable_area(&Point2dList(vec![
        Point2d::new(0.0, 0.0),
        Point2d::new(256.0, 256.0),
    ]))
}

#[test]
fn task22b_checked_coordinate_scaling_truncates_and_round_trips() {
    let normal = normal_scale();
    let large = CoordinateScale::from_printable_area(&Point2dList(vec![
        Point2d::new(0.0, 0.0),
        Point2d::new(2_147.001, 0.0),
    ]));

    assert_eq!(normal.checked_scale(1.9e-6), Some(1));
    assert_eq!(normal.checked_scale(-1.9e-6), Some(-1));

    for scale in [normal, large] {
        for coordinate in [0, 1, -1, 10, -10] {
            let unscaled = scale.unscale(coordinate);
            assert_eq!(scale.checked_scale(unscaled), Some(coordinate));
        }
    }
}

#[test]
fn task22b_checked_coordinate_scaling_rejects_nonfinite_and_half_open_i64_range() {
    let normal = normal_scale();
    let large = CoordinateScale::from_printable_area(&Point2dList(vec![
        Point2d::new(0.0, 0.0),
        Point2d::new(2_147.001, 0.0),
    ]));
    let lower_quotient = i64::MIN as f64;
    let upper_quotient = -lower_quotient;
    let expected_upper_coord = 9_223_372_036_854_773_760;

    for scale in [normal, large] {
        for coordinate in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(scale.checked_scale(coordinate), None);
        }

        let factor = scale.factor();
        let upper_coordinate = upper_quotient * factor;
        let lower_coordinate = lower_quotient * factor;
        assert_eq!(
            scale.checked_scale(upper_coordinate.next_down()),
            Some(expected_upper_coord)
        );
        assert_eq!(scale.checked_scale(upper_coordinate), None);
        assert_eq!(scale.checked_scale(lower_coordinate), Some(i64::MIN));
        assert_eq!(scale.checked_scale(lower_coordinate.next_down()), None);
        assert!((f64::MAX / factor).is_infinite());
        assert_eq!(scale.checked_scale(f64::MAX), None);
    }
}

#[test]
fn task22b_point_equality_and_order_are_integer_x_then_y() {
    let origin = Point::new(0, 0);
    let x_equal_y_lower = Point::new(1, -1);
    let x_equal_y_higher = Point::new(1, 2);
    let x_higher = Point::new(2, -100);

    assert_eq!(origin, Point::new(0, 0));
    assert_ne!(origin, Point::new(0, 1));
    assert_ne!(origin, Point::new(1, 0));
    assert!(origin < x_equal_y_lower);
    assert!(x_equal_y_lower < x_equal_y_higher);
    assert!(x_equal_y_higher < x_higher);

    let x: Coord = x_equal_y_higher.x();
    let y: Coord = x_equal_y_higher.y();
    let _: i64 = x;
    let _: i64 = y;
    assert_eq!((x, y), (1, 2));
}
