use super::intersect;
use crate::geometry::{Point, Polygon, Polyline};

fn diamond() -> Polygon {
    Polygon::new(vec![
        Point::new(0, -5_684_701),
        Point::new(5_684_701, -1),
        Point::new(0, 5_684_700),
        Point::new(-5_684_701, -1),
    ])
}

#[test]
fn clipper1_open_intersection_preserves_exact_octagram_boundary_point() {
    let subject = Polyline::new(vec![
        Point::new(0, 4_029_883),
        Point::new(-2_849_557, 6_879_440),
    ]);
    let output = intersect(&[subject], &[diamond()]).unwrap();

    assert_eq!(
        output[0].points(),
        [Point::new(0, 4_029_883), Point::new(-827_408, 4_857_292)]
    );
}

#[test]
fn clipper1_double_intersection_keeps_source_one_unit_rounding() {
    let subject = Polyline::new(vec![
        Point::new(2_849_557, 6_879_440),
        Point::new(0, 4_029_883),
    ]);

    let output = intersect(&[subject], &[diamond()]).unwrap();

    assert_eq!(
        output[0].points(),
        [Point::new(827_409, 4_857_292), Point::new(0, 4_029_883)]
    );
}
