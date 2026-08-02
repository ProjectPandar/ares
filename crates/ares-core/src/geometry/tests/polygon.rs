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

#[test]
fn task22o6_polygon_split_at_first_point_appends_exact_closure_duplicate() {
    let polygon = Polygon::new(vec![
        Point::new(7, 3),
        Point::new(11, 5),
        Point::new(-2, 13),
    ]);

    assert_eq!(
        polygon.split_at_first_point().points(),
        &[
            Point::new(7, 3),
            Point::new(11, 5),
            Point::new(-2, 13),
            Point::new(7, 3),
        ]
    );
    assert!(
        Polygon::new(Vec::new())
            .split_at_first_point()
            .points()
            .is_empty()
    );
}

#[test]
fn task22o4_polygon_contains_includes_boundaries() {
    let polygon = Polygon::new(vec![
        Point::new(0, 0),
        Point::new(10, 0),
        Point::new(10, 10),
        Point::new(0, 10),
    ]);
    assert!(polygon.contains(&Point::new(5, 5)));
    assert!(polygon.contains(&Point::new(0, 5)));
    assert!(!polygon.contains(&Point::new(-1, 5)));
}

#[test]
fn task22h_polygon_area_is_positive_zero_for_fewer_than_three_points() {
    for points in [
        vec![],
        vec![Point::new(7, -11)],
        vec![Point::new(7, -11), Point::new(-13, 17)],
    ] {
        assert_eq!(Polygon::new(points).area().to_bits(), 0.0f64.to_bits());
    }
}

#[test]
fn task22h_polygon_area_is_signed_by_vertex_order() {
    let counterclockwise = Polygon::new(vec![Point::new(0, 0), Point::new(4, 0), Point::new(0, 3)]);
    let clockwise = Polygon::new(vec![Point::new(0, 3), Point::new(4, 0), Point::new(0, 0)]);

    assert_eq!(counterclockwise.area(), 6.0);
    assert_eq!(clockwise.area(), -6.0);
}

#[test]
fn task22h_polygon_area_preserves_upstream_floating_point_operation_order() {
    let mut polygon = Polygon::new(vec![
        Point::new(-806_358_058, -56_288_362),
        Point::new(86_494_274, -121_692_832),
        Point::new(620_027_314, 829_088_933),
        Point::new(151_642_914, 356_616_509),
        Point::new(-458_957_691, 876_703_926),
    ]);

    assert_eq!(polygon.area().to_bits(), 0x43a3_14c1_7e96_6778);

    polygon.reverse();
    assert_eq!(polygon.area().to_bits(), 0xc3a3_14c1_7e96_6779);
    assert!(polygon.area().is_sign_negative());
}
