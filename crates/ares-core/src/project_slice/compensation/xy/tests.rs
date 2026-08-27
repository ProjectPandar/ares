use super::apply;
use crate::geometry::{ExPolygon, Point, Polygon};

fn square(minimum: i64, maximum: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(minimum, minimum),
        Point::new(maximum, minimum),
        Point::new(maximum, maximum),
        Point::new(minimum, maximum),
    ])
}

fn bounds(polygon: &Polygon) -> (i64, i64) {
    polygon
        .points()
        .iter()
        .fold((i64::MAX, i64::MIN), |(minimum, maximum), point| {
            (
                minimum.min(point.x()).min(point.y()),
                maximum.max(point.x()).max(point.y()),
            )
        })
}

#[test]
fn contour_and_hole_deltas_are_applied_independently() {
    let mut hole = square(300, 700);
    hole.reverse();
    let source = ExPolygon::new(square(0, 1_000), vec![hole]);

    let adjusted = apply(&[source], 100.0, 50.0).unwrap();

    assert_eq!(adjusted.len(), 1);
    assert_eq!(bounds(adjusted[0].contour()), (-100, 1_100));
    assert_eq!(adjusted[0].holes().len(), 1);
    assert_eq!(bounds(&adjusted[0].holes()[0]), (250, 750));
}

#[test]
fn zero_compensation_preserves_geometry() {
    let source = ExPolygon::new(square(0, 1_000), Vec::new());

    assert_eq!(
        apply(std::slice::from_ref(&source), 0.0, 0.0).unwrap(),
        [source]
    );
}
