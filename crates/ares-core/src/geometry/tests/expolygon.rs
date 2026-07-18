use crate::geometry::{ExPolygon, Point, Polygon};

#[test]
fn task22f_polygon_into_points_preserves_order_without_normalization() {
    let points = vec![
        Point::new(5, 5),
        Point::new(0, 5),
        Point::new(0, 0),
        Point::new(5, 5),
    ];

    assert_eq!(Polygon::new(points.clone()).into_points(), points);
}

#[test]
fn task22f_expolygon_owns_contour_and_ordered_holes() {
    let contour = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(20, 0),
        Point::new(20, 20),
        Point::new(0, 20),
    ]);
    let holes = vec![
        Polygon::new(vec![
            Point::new(2, 2),
            Point::new(2, 4),
            Point::new(4, 4),
            Point::new(4, 2),
        ]),
        Polygon::new(vec![
            Point::new(10, 10),
            Point::new(10, 14),
            Point::new(14, 14),
            Point::new(14, 10),
        ]),
    ];

    let expolygon = ExPolygon::new(contour.clone(), holes.clone());
    assert_eq!(expolygon.contour(), &contour);
    assert_eq!(expolygon.holes(), holes.as_slice());
    assert_eq!(expolygon.into_parts(), (contour, holes));
}
