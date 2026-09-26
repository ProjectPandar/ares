use super::*;

#[test]
fn junction_trim_truncates_displacement_before_adding_endpoint() {
    for sign in [-1, 1] {
        let end = Point::new(sign * 10, sign * 10);
        for reduction in [-1, 1] {
            let mut lines = vec![Polyline::new(vec![Point::new(0, 0), end])];
            Node::remove_junction_overlap(&mut lines, reduction);
            // length truncates to 14; each displacement is +/-10/14,
            // cast to zero before adding to the integer endpoint.
            assert_eq!(lines[0].points(), &[Point::new(0, 0), end]);
        }
    }
}

#[test]
fn junction_trim_carries_reduction_across_consumed_segments() {
    let mut lines = vec![Polyline::new(vec![
        Point::new(0, 0),
        Point::new(10, 10),
        Point::new(10, 11),
    ])];
    Node::remove_junction_overlap(&mut lines, 2);
    assert_eq!(lines[0].points(), &[Point::new(0, 0), Point::new(10, 10)]);
}
