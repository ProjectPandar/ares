use crate::geometry::{
    CoordinateScale, ExPolygon, Point, Polygon, medial_axis, medial_axis::validate::rounded_point,
};

#[test]
fn task22o13_voronoi_point_conversion_rounds_fractional_literals_away_from_zero() {
    assert_eq!(rounded_point(4.5, 4.49), Point::new(5, 4));
    assert_eq!(rounded_point(-4.5, -4.49), Point::new(-5, -4));
    assert_eq!(rounded_point(0.5, -0.5), Point::new(1, -1));
}

#[test]
fn task22o13_validation_preserves_literal_width_direction_and_limits() {
    let input = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000, 0),
            Point::new(1_000, 400),
            Point::new(0, 400),
        ]),
        Vec::new(),
    );
    let accepted = medial_axis(&input, 0.0, 1_000.0, CoordinateScale::Normal).unwrap();
    assert_eq!(
        accepted[0].points,
        vec![Point::new(1_000, 200), Point::new(0, 200)]
    );
    assert_eq!(accepted[0].width, vec![400.0, 400.0]);
    assert!(
        medial_axis(&input, 500.0, 1_000.0, CoordinateScale::Normal)
            .unwrap()
            .is_empty()
    );
    assert!(
        medial_axis(&input, 0.0, 300.0, CoordinateScale::Normal)
            .unwrap()
            .is_empty()
    );
}
