use crate::{
    geometry::ExPolygon,
    project_slice::{
        prepare_infill::vertical_shell_trimming::trim,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn task22o21_filtering_is_collection_order_not_kind_order_and_contour_then_holes() {
    let solid_contour = super::square(100, 140);
    let solid_hole = super::square(110, 120);
    let internal_contour = super::square(0, 40);
    let internal_hole = super::square(10, 20);
    let record = super::record(vec![
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            ExPolygon::new(solid_contour.clone(), vec![solid_hole.clone()]),
        ),
        super::surface(RegionSurfaceKind::Top, 200, 220),
        RegionSurface::new(
            RegionSurfaceKind::Internal,
            ExPolygon::new(internal_contour.clone(), vec![internal_hole.clone()]),
        ),
    ]);
    assert_eq!(
        trim::polygons_internal(&record),
        vec![
            solid_contour.clone(),
            solid_hole.clone(),
            internal_contour,
            internal_hole,
        ]
    );
    assert_eq!(trim::solid_paths(&record), vec![solid_contour, solid_hole]);
}

#[test]
fn task22o21_surface_metadata_does_not_change_path_selection() {
    let geometry = ExPolygon::new(super::square(0, 40), vec![super::square(10, 20)]);
    let plain = super::record(vec![RegionSurface::new(
        RegionSurfaceKind::Internal,
        geometry.clone(),
    )]);
    let metadata = super::record(vec![RegionSurface::internal_with_metadata(
        geometry, 3.5, 7, 1.25, 9,
    )]);
    assert_eq!(
        trim::polygons_internal(&plain),
        trim::polygons_internal(&metadata)
    );
}
