use super::*;
use crate::geometry::Point;

#[test]
fn round_corner_uses_clipper2_relative_arc_tolerance() {
    let output = apply(
        vec![Polyline::new(vec![
            Point::new(0, 0),
            Point::new(1_000_000, 0),
            Point::new(1_000_000, 1_000_000),
        ])],
        2,
        0.4,
        CoordinateScale::Normal,
    )
    .unwrap();
    // Vendored Clipper2 Execute(200000), Round joins and Round ends.
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].points().len(), 68);
    assert!(
        output[0]
            .points()
            .contains(&Point::new(1_025_235, -198_402))
    );
}

#[test]
fn duplicate_vertices_do_not_change_round_offsets() {
    let a = Point::new(-1_000_000, -1_000_000);
    let b = Point::new(0, -1_000_000);
    let c = Point::new(0, 0);
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        for count in [2, 3, 4] {
            let expected = apply(vec![Polyline::new(vec![a, b, c])], count, 0.4, scale).unwrap();
            let actual =
                apply(vec![Polyline::new(vec![a, a, b, b, c])], count, 0.4, scale).unwrap();
            // Odd bundles retain the unmodified input centreline.
            assert_eq!(
                &actual[(count % 2) as usize..],
                &expected[(count % 2) as usize..]
            );
        }
    }
}

#[test]
fn collapsed_open_path_produces_a_round_disk() {
    let p = Point::new(-1_000_000, 0);
    let output = apply(
        vec![Polyline::new(vec![p, p])],
        2,
        0.4,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].points().len(), 51);
    assert!(output[0].points().contains(&Point::new(-800_000, 0)));
    assert!(output[0].points().contains(&Point::new(-1_200_000, 0)));
}

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
