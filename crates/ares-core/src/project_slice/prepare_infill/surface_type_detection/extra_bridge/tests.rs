use super::{Cache, rewrite};
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
fn source_external_second_bridge_splits_only_upper_internal_surface() {
    let internal = RegionSurface::internal(rectangle(0, 0, 1_000_000, 1_000_000));
    let top = RegionSurface::new(
        RegionSurfaceKind::Top,
        rectangle(2_000_000, 0, 3_000_000, 1_000_000),
    );
    let cache = Cache {
        polygons: vec![rectangle(0, 0, 600_000, 1_000_000)],
        offset: 10_000.0,
        region: 0,
    };

    let output = rewrite(vec![internal, top.clone()], &cache).unwrap();

    assert!(
        output
            .iter()
            .any(|surface| { surface.as_parts().0 == RegionSurfaceKind::BottomBridge })
    );
    assert!(
        output
            .iter()
            .any(|surface| { surface.as_parts().0 == RegionSurfaceKind::Internal })
    );
    assert!(
        output
            .iter()
            .any(|surface| surface.as_parts() == top.as_parts())
    );
}
