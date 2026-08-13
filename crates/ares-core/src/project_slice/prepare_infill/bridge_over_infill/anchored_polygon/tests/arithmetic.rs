use crate::geometry::{CoordinateScale, Point};

use super::super::{rotate_point_for_test, scaled_flow_value_for_test};

#[test]
fn task22o53_flow_scaling_uses_f32_value_f64_division_and_truncation() {
    let value = f32::from_bits(0x3ed7_0a3e);
    assert_eq!(
        scaled_flow_value_for_test(value, CoordinateScale::Normal),
        420_000
    );
    assert_eq!(
        scaled_flow_value_for_test(value, CoordinateScale::LargeBed),
        42_000
    );
}

#[test]
fn task22o53_rotation_rounds_positive_and_negative_half_ties_away_from_zero() {
    assert_eq!(
        rotate_point_for_test(Point::new(1, 0), 0.5, 0.0),
        Point::new(1, 0)
    );
    assert_eq!(
        rotate_point_for_test(Point::new(-1, 0), 0.5, 0.0),
        Point::new(-1, 0)
    );
}
