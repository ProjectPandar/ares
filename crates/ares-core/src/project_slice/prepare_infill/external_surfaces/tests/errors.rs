use super::{expand_merge_surfaces, helpers::*};
use crate::{
    geometry::{ClipperError, CoordinateScale},
    project_slice::region_slices::RegionSurfaceKind,
};

const OUTSIDE: i64 = 0x4000_0000_0000_0000;
const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

fn selected(
    source: crate::geometry::ExPolygon,
) -> Vec<crate::project_slice::region_slices::RegionSurface> {
    vec![surface(RegionSurfaceKind::Top, source, (0.2, 1, -1.0, 0))]
}

fn geometry_is_empty(surface: &crate::project_slice::region_slices::RegionSurface) -> bool {
    let geometry = surface.as_parts().1;
    geometry.contour().points().is_empty() && geometry.holes().is_empty()
}

#[test]
fn task22o35_first_zone_discovery_error_follows_source_extraction() {
    let mut surfaces = selected(square(20, 30));
    let invalid = expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    );
    let mut zones = vec![zone(vec![invalid])];

    assert!(matches!(
        expand_merge_surfaces(
            &mut surfaces,
            RegionSurfaceKind::Top,
            &mut zones,
            1.0,
            -1.0,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(geometry_is_empty(&surfaces[0]));
    assert!(!zones[0].expanded_into);
}

#[test]
fn task22o35_later_zone_error_preserves_the_earlier_flag_without_trimming() {
    let mut surfaces = selected(square(20, 30));
    let first_geometry = square(0, 100);
    let first_snapshot = snapshots(std::slice::from_ref(&first_geometry));
    let invalid = expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    );
    let mut zones = vec![zone(vec![first_geometry]), zone(vec![invalid])];

    assert!(matches!(
        expand_merge_surfaces(
            &mut surfaces,
            RegionSurfaceKind::Top,
            &mut zones,
            1.0,
            -1.0,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(geometry_is_empty(&surfaces[0]));
    assert!(zones[0].expanded_into);
    assert!(!zones[1].expanded_into);
    assert_eq!(snapshots(&zones[0].expolygons), first_snapshot);
}

#[test]
fn task22o35_invalid_radius_panics_after_nonempty_source_extraction() {
    for radius in [0.0, -1.0, f32::NAN] {
        let mut surfaces = selected(square(20, 30));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            expand_merge_surfaces(
                &mut surfaces,
                RegionSurfaceKind::Top,
                &mut [],
                radius,
                -1.0,
                CoordinateScale::Normal,
            )
        }));
        assert!(result.is_err());
        assert!(geometry_is_empty(&surfaces[0]));
    }
}

#[test]
fn task22o35_closing_coordinate_error_escapes_after_no_zone_merge() {
    let near_limit = expolygon(
        &[
            (HI_RANGE - 1_000, 0),
            (HI_RANGE - 500, 0),
            (HI_RANGE - 500, 500),
            (HI_RANGE - 1_000, 500),
        ],
        Vec::new(),
    );
    let mut surfaces = selected(near_limit);

    assert!(matches!(
        expand_merge_surfaces(
            &mut surfaces,
            RegionSurfaceKind::Top,
            &mut [],
            2_000.0,
            -1.0,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    ));
    assert!(geometry_is_empty(&surfaces[0]));
}
