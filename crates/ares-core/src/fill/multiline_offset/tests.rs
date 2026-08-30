use super::*;
use crate::geometry::Point;

#[test]
fn even_bundle_offsets_around_the_centerline_without_retaining_it() {
    let output = apply(
        vec![Polyline::new(vec![
            Point::new(0, 0),
            Point::new(100_000, 0),
        ])],
        2,
        0.01,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(output.len(), 1);
    assert_eq!(output[0].points().first(), output[0].points().last());
    let min_y = output[0].points().iter().map(|point| point.y()).min();
    let max_y = output[0].points().iter().map(|point| point.y()).max();
    assert_eq!((min_y, max_y), (Some(-5_000), Some(5_000)));
}
