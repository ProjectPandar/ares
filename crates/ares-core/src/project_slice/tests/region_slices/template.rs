use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

fn square(x: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + 10, 0),
            Point::new(x + 10, 10),
            Point::new(x, 10),
        ]),
        Vec::new(),
    )
}

#[test]
fn task22o26_template_replaces_only_geometry_and_preserves_all_metadata() {
    let original = RegionSurface::internal_with_metadata(square(0), 2.5, 7, 0.75, 9)
        .clone_with_kind(RegionSurfaceKind::BottomBridge);
    let replacement = square(100);

    let output = original.clone_with_expolygon(replacement.clone());
    let (kind, geometry, thickness, layers, angle, extra) = output.as_parts();

    assert_eq!(kind, RegionSurfaceKind::BottomBridge);
    assert_eq!(geometry, &replacement);
    assert_eq!(thickness.to_bits(), 2.5_f64.to_bits());
    assert_eq!(layers, 7);
    assert_eq!(angle.to_bits(), 0.75_f64.to_bits());
    assert_eq!(extra, 9);
    assert_eq!(original.as_parts().1, &square(0));
}
