use super::*;

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

#[test]
fn negative_position_sorts_below_positive() {
    // The rounded-diamond center vline: top crossing positive, bottom
    // crossing negative — the mixed-sign case of `operator<`.
    let radius = 5_112_000;
    let diamond = Polygon::new(vec![
        Point::new(3_797, radius),
        Point::new(-radius, 3_797),
        Point::new(-3_797, -radius),
        Point::new(radius, -3_797),
    ]);
    let raw = super::line_intersections(&[diamond], 0);
    assert_eq!(raw.len(), 2);
    let mut sorted = raw.clone();
    sorted.sort_by(|left, right| {
        if left.below(right) {
            std::cmp::Ordering::Less
        } else if right.below(left) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    // The OUTER_LOW (entering, `low`) crossing sorts first at the
    // geometrically lower position.
    assert!(sorted[0].low);
    assert!(!sorted[1].low);
}
