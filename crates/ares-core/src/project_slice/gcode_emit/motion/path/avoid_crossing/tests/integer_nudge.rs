//! `AvoidCrossingPerimeters.cpp:573-574`: cast the displacement, not the sum.
use super::*;

#[test]
fn closest_retry_keeps_native_post_wipe_route_outside_top_boundary() {
    let slices = vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(5_000_000, 5_000_000),
            Point::new(-5_000_000, 5_000_000),
            Point::new(-5_000_000, -5_000_000),
            Point::new(5_000_000, -5_000_000),
        ]),
        Vec::new(),
    )];
    let geometry = AvoidCrossingGeometry {
        external_perimeter_width: 0.4,
        layer_slices: &slices,
        perimeter_spacing: 0.4,
        top_surfaces: &[],
        chunk_slices: &[],
        chunk_perimeter_spacing: 0.0,
    };
    let start = Point::new(-13_729_879, 4_425_418);
    let end = Point::new(4_401_461, 4_401_461);
    let super::super::boundary::BuildResult::Ready(boundary) =
        super::super::boundary::Boundary::build_with_spacing(
            &geometry,
            CoordinateScale::Normal,
            [start, end],
            400_000.0,
        )
        .unwrap()
    else {
        panic!("the captured square has a boundary");
    };
    assert_eq!(
        boundary.contours,
        [vec![
            Point::new(4_400_000, 4_400_000),
            Point::new(-4_400_000, 4_400_000),
            Point::new(-4_400_000, -4_400_000),
            Point::new(4_400_000, -4_400_000),
        ]]
    );
    for endpoints in [[start, end], [end, start]] {
        let (route, _) =
            super::super::router::avoid_perimeters(&boundary, endpoints[0], endpoints[1]).unwrap();
        assert_eq!(route, endpoints);
    }
}
