use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::vertical_shells::{
            GeometryStep, cache::build, fail_geometry_at, geometry_events, reset_geometry_hooks,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn range_failure_uses_stable_vertical_shell_error() {
    reset_geometry_hooks();
    const HIGH: i64 = 0x3fff_ffff_ffff_ffff;
    let geometry = ExPolygon::new(
        Polygon::new(vec![
            Point::new(HIGH - 1_500_000, 0),
            Point::new(HIGH - 500_000, 0),
            Point::new(HIGH - 500_000, 1_000_000),
            Point::new(HIGH - 1_500_000, 1_000_000),
        ]),
        Vec::new(),
    );
    let error = build(
        &[RegionSurface::new(RegionSurfaceKind::Top, geometry)],
        &[],
        ProcessEnsureVerticalShellThickness::EnsureAll,
        40_000_000,
    )
    .err()
    .unwrap();
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "vertical-shell cache geometry is outside the supported Clipper range".to_owned()
        )
    );
    assert_eq!(geometry_events(), vec![GeometryStep::Top]);
    reset_geometry_hooks();
}

#[test]
fn injected_bottom_failure_occurs_after_top_call() {
    reset_geometry_hooks();
    fail_geometry_at(GeometryStep::Bottom);
    let error = build(
        &[],
        &[],
        ProcessEnsureVerticalShellThickness::EnsureAll,
        1_000,
    )
    .unwrap_err();
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "vertical-shell cache geometry is outside the supported Clipper range".to_owned()
        )
    );
    assert_eq!(
        geometry_events(),
        vec![GeometryStep::Top, GeometryStep::Bottom]
    );
    reset_geometry_hooks();
}
