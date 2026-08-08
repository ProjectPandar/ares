use super::{EXPAND_MERGE_SURFACES, helpers::*};
use crate::{geometry::CoordinateScale, project_slice::region_slices::RegionSurfaceKind};

#[test]
fn task22o35_zero_selected_sources_short_circuit_without_mutation() {
    let untouched = square(200, 260);
    let untouched_pointer = untouched.contour().points().as_ptr();
    let mut surfaces = vec![surface(
        RegionSurfaceKind::Bottom,
        untouched,
        (0.2, 2, 0.4, 3),
    )];
    let zone_geometry = square(0, 400);
    let zone_pointer = zone_geometry.contour().points().as_ptr();
    let mut zones = vec![zone(vec![zone_geometry])];

    let actual = EXPAND_MERGE_SURFACES(
        &mut surfaces,
        RegionSurfaceKind::Top,
        &mut zones,
        f32::NAN,
        f64::INFINITY,
        CoordinateScale::Normal,
    );

    assert!(actual.unwrap().is_empty());
    let (kind, geometry, thickness, layers, angle, extra) = surfaces[0].as_parts();
    assert_eq!(kind, RegionSurfaceKind::Bottom);
    assert_eq!(geometry.contour().points().as_ptr(), untouched_pointer);
    assert_eq!((thickness, layers, angle, extra), (0.2, 2, 0.4, 3));
    assert!(!zones[0].expanded_into);
    assert_eq!(
        zones[0].expolygons[0].contour().points().as_ptr(),
        zone_pointer
    );
}

#[test]
fn task22o35_selected_geometry_moves_and_output_uses_exact_metadata_defaults() {
    let selected = expolygon(
        &[(20, 20), (80, 20), (80, 80), (20, 80)],
        vec![polygon(&[(35, 35), (35, 50), (50, 50), (50, 35)])],
    );
    let second_selected = square(120, 180);
    let untouched = square(220, 280);
    let untouched_pointer = untouched.contour().points().as_ptr();
    let source = vec![selected, second_selected];
    let mut expected_zones = Vec::new();
    let expected =
        explicit_pipeline(source, &mut expected_zones, 5.0, CoordinateScale::Normal).unwrap();
    let mut surfaces = vec![
        surface(
            RegionSurfaceKind::Top,
            expolygon(
                &[(20, 20), (80, 20), (80, 80), (20, 80)],
                vec![polygon(&[(35, 35), (35, 50), (50, 50), (50, 35)])],
            ),
            (0.25, 4, 1.25, 6),
        ),
        surface(RegionSurfaceKind::Bottom, untouched, (0.3, 3, 0.5, 7)),
        surface(RegionSurfaceKind::Top, square(120, 180), (0.4, 5, 1.5, 8)),
    ];

    let actual = EXPAND_MERGE_SURFACES(
        &mut surfaces,
        RegionSurfaceKind::Top,
        &mut [],
        5.0,
        0.75,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(surface_snapshots(&actual), snapshots(&expected));
    for index in [0, 2] {
        let (_, geometry, thickness, layers, angle, extra) = surfaces[index].as_parts();
        assert!(geometry.contour().points().is_empty());
        assert!(geometry.holes().is_empty());
        let original = if index == 0 {
            (0.25, 4, 1.25, 6)
        } else {
            (0.4, 5, 1.5, 8)
        };
        assert_eq!((thickness, layers, angle, extra), original);
    }
    let (_, geometry, thickness, layers, angle, extra) = surfaces[1].as_parts();
    assert_eq!(geometry.contour().points().as_ptr(), untouched_pointer);
    assert_eq!((thickness, layers, angle, extra), (0.3, 3, 0.5, 7));

    for output in &actual {
        let (kind, _, thickness, layers, angle, extra) = output.as_parts();
        assert_eq!(kind, RegionSurfaceKind::Top);
        assert_eq!((thickness, layers, angle, extra), (-1.0, 1, 0.75, 0));
    }
}
