use super::{Cache, rewrite_surfaces};
use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        ]),
        Vec::new(),
    )
}

#[test]
fn source_internal_second_bridge_splits_overlap_and_rotates_angle() {
    let source = RegionSurface::internal(rectangle(0, 0, 1_000_000, 1_000_000));
    let cache = Cache {
        polygons: vec![rectangle(0, 0, 600_000, 1_000_000)],
        angle: 0.25,
        offset: 10_000.0,
    };

    let output = rewrite_surfaces(vec![source], &cache.polygons, &cache).unwrap();

    assert!(output.iter().any(|surface| {
        let (kind, _, _, _, angle, _) = surface.as_parts();
        kind == RegionSurfaceKind::InternalBridge
            && (angle - (0.25 + std::f64::consts::FRAC_PI_2)).abs() < f64::EPSILON
    }));
    assert!(
        output
            .iter()
            .any(|surface| { surface.as_parts().0 == RegionSurfaceKind::Internal })
    );
}
