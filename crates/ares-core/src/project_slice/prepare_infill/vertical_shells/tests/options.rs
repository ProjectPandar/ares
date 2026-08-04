use crate::{
    ProcessEnsureVerticalShellThickness,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::vertical_shells::{cache::build, geometry_events, reset_geometry_hooks},
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn inactive_modes_skip_geometry_and_return_empty_caches() {
    reset_geometry_hooks();
    let geometry = ExPolygon::new(
        Polygon::new(vec![
            Point::new(i64::MAX, i64::MAX),
            Point::new(i64::MIN, i64::MAX),
            Point::new(i64::MIN, i64::MIN),
        ]),
        Vec::new(),
    );
    let slices = [RegionSurface::new(RegionSurfaceKind::Top, geometry)];
    for mode in [
        ProcessEnsureVerticalShellThickness::None,
        ProcessEnsureVerticalShellThickness::CriticalOnly,
        ProcessEnsureVerticalShellThickness::Moderate,
    ] {
        let cache = build(&slices, &[], mode, i64::MAX).unwrap();
        assert!(cache.top_surfaces.is_empty());
        assert!(cache.bottom_surfaces.is_empty());
        assert!(cache.holes.is_empty());
    }
    assert!(geometry_events().is_empty());
    reset_geometry_hooks();
}
