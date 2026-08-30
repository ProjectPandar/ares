use super::super::{bounds::Bounds, output::InfillPolylineOutput};
use crate::geometry::Point;

#[test]
fn output_rounding_matches_floor_value_plus_half() {
    let mut output = InfillPolylineOutput::plain(1.0);

    output.add_point(-0.5, 0.5).unwrap();

    assert_eq!(output.result(), [Point::new(0, 1)]);
}

#[test]
fn clipper_drops_only_a_middle_point_outside_the_same_side() {
    let bounds = Bounds::new(Point::new(-5, -5), Point::new(5, 5));
    let mut output = InfillPolylineOutput::clipped(bounds, 1.0);

    for x in [-10.0, -9.0, -8.0, 0.0] {
        output.add_point(x, 0.0).unwrap();
    }

    assert_eq!(
        output.result(),
        [Point::new(-10, 0), Point::new(-8, 0), Point::new(0, 0)]
    );
}

#[test]
fn clipper_keeps_a_corner_crossing_candidate() {
    let bounds = Bounds::new(Point::new(-5, -5), Point::new(5, 5));
    let mut output = InfillPolylineOutput::clipped(bounds, 1.0);

    for (x, y) in [(-10.0, -10.0), (-9.0, -9.0), (10.0, 10.0)] {
        output.add_point(x, y).unwrap();
    }

    assert_eq!(
        output.result(),
        [Point::new(-10, -10), Point::new(-9, -9), Point::new(10, 10)]
    );
}
