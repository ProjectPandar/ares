use crate::geometry::{Point, Polygon};

#[test]
fn task22c_polygon_preserves_integer_points_without_normalization() {
    let polygon = Polygon::new(vec![
        Point::new(5, 5),
        Point::new(5, 0),
        Point::new(0, 0),
        Point::new(5, 0),
        Point::new(0, 5),
    ]);

    assert_eq!(
        polygon.points(),
        &[
            Point::new(5, 5),
            Point::new(5, 0),
            Point::new(0, 0),
            Point::new(5, 0),
            Point::new(0, 5),
        ]
    );
    assert_ne!(polygon.points().first(), polygon.points().last());
}
