use super::*;

fn diamond(radius: Coord) -> Polygon {
    Polygon::new(vec![
        Point::new(0, radius),
        Point::new(radius, 0),
        Point::new(0, -radius),
        Point::new(-radius, 0),
    ])
}

#[test]
fn center_line_spans_the_diamond() {
    // Rounded diamond (as the rotated square produces after integer
    // rotation) so no vertex lies exactly on the sliced line. CCW like
    // the upstream polygon orientation convention.
    let radius = 5_112_000;
    let diamond = Polygon::new(vec![
        Point::new(3_797, radius),
        Point::new(-radius, 3_797),
        Point::new(-3_797, -radius),
        Point::new(radius, -3_797),
    ]);
    let spans = vertical_spans(&[diamond], -6_000_000, 3_000_000, 5);
    let center = spans
        .iter()
        .find(|span| span.points()[0].x() == 0)
        .expect("center span exists");
    let mut ys = [center.points()[0].y(), center.points()[1].y()];
    ys.sort();
    assert_eq!(ys, [-5_108_209, 5_108_209]);
    assert_eq!(
        spans
            .iter()
            .filter(|span| span.points()[0].x() == 0)
            .count(),
        1
    );
}
