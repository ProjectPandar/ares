use crate::{
    geometry::ExPolygon,
    project_slice::{
        prepare_infill::vertical_shell_trimming::trim::polygons_internal,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn task22o24_internal_void_is_representable_but_has_no_o21_producer() {
    let representable = [
        RegionSurfaceKind::Top,
        RegionSurfaceKind::Bottom,
        RegionSurfaceKind::BottomBridge,
        RegionSurfaceKind::Internal,
        RegionSurfaceKind::InternalSolid,
        RegionSurfaceKind::InternalVoid,
    ];
    assert_eq!(representable.len(), 6);
    assert_eq!(RegionSurfaceKind::Internal as u8, 4);
    assert_eq!(RegionSurfaceKind::InternalSolid as u8, 5);
    assert_eq!(RegionSurfaceKind::InternalVoid as u8, 8);
    assert!(!RegionSurfaceKind::InternalVoid.is_bridge());
}

#[test]
fn task22o24_polygons_internal_includes_void_in_collection_and_hole_order() {
    let internal = super::square(0, 100);
    let void_contour = super::square(200, 300);
    let void_hole = super::square(220, 280);
    let solid = super::square(400, 500);
    let record = super::record(vec![
        RegionSurface::new(
            RegionSurfaceKind::Internal,
            ExPolygon::new(internal.clone(), Vec::new()),
        ),
        RegionSurface::new(
            RegionSurfaceKind::InternalVoid,
            ExPolygon::new(void_contour.clone(), vec![void_hole.clone()]),
        ),
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            ExPolygon::new(solid.clone(), Vec::new()),
        ),
    ]);
    assert_eq!(
        polygons_internal(&record),
        vec![internal, void_contour, void_hole, solid]
    );
}
