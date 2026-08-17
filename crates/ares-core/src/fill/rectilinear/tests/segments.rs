use crate::geometry::{ExPolygon, Point, Polygon};

use super::super::{IntersectionKind, slice_vertical_lines};

#[test]
fn task22o157_scanline_intersections_sort_by_exact_rational_position() {
    let shape = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 104),
            Point::new(0, 100),
        ]),
        vec![Polygon::new(vec![
            Point::new(0, 20),
            Point::new(0, 99),
            Point::new(10, 105),
            Point::new(10, 20),
        ])],
    );

    let sections = slice_vertical_lines(&shape, 0.0, 0.0, 0.0, 1, 1, 1).unwrap();
    assert_eq!(
        sections[0]
            .intersections
            .iter()
            .map(|intersection| (intersection.point.y(), intersection.kind))
            .collect::<Vec<_>>(),
        vec![
            (0, IntersectionKind::OuterLow),
            (20, IntersectionKind::OuterHigh),
            (100, IntersectionKind::OuterLow),
            (100, IntersectionKind::OuterHigh),
        ]
    );
}
