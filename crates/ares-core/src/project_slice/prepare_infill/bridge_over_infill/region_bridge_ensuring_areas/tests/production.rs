use super::*;

#[test]
fn task22o66_real_geometry_builds_near_ring_and_clips_ensuring_topology() {
    let fill_surfaces = vec![
        surface(
            RegionSurfaceKind::Bottom,
            expolygon(rectangle(0, 0, 100, 100), vec![rectangle(30, 30, 70, 70)]),
        ),
        surface(
            RegionSurfaceKind::Internal,
            expolygon(rectangle(200, 0, 300, 100), Vec::new()),
        ),
    ];
    let ensuring = vec![rectangle(-20, -20, 60, 120), rectangle(240, -20, 320, 120)];
    let before = surface_snapshot(&fill_surfaces);
    let ensuring_before = snapshot(&ensuring);
    let contour_ptrs = fill_surfaces
        .iter()
        .map(|surface| surface.as_parts().1.contour().points().as_ptr())
        .collect::<Vec<_>>();
    let ensuring_ptrs = ensuring
        .iter()
        .map(|polygon| polygon.points().as_ptr())
        .collect::<Vec<_>>();

    let first = prepare_region_bridge_ensuring_areas(
        &fill_surfaces,
        &ensuring,
        flow(0.000_01),
        CoordinateScale::Normal,
    )
    .unwrap();
    let second = prepare_region_bridge_ensuring_areas(
        &fill_surfaces,
        &ensuring,
        flow(0.000_01),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(
        snapshot(&first.near_perimeters),
        vec![
            vec![(110, 110), (-10, 110), (-10, -10), (110, -10)],
            vec![(310, 110), (190, 110), (190, -10), (310, -10)],
            vec![(199, -1), (199, 101), (301, 101), (301, -1)],
            vec![(-1, -1), (-1, 101), (101, 101), (101, -1)],
        ]
    );
    assert_eq!(
        first
            .additional_ensuring
            .iter()
            .map(|area| snapshot(std::slice::from_ref(area.contour())))
            .collect::<Vec<_>>(),
        vec![
            vec![vec![
                (310, 110),
                (240, 110),
                (240, 101),
                (301, 101),
                (301, -1),
                (240, -1),
                (240, -10),
                (310, -10),
            ]],
            vec![vec![
                (60, -1),
                (-1, -1),
                (-1, 101),
                (60, 101),
                (60, 110),
                (-10, 110),
                (-10, -10),
                (60, -10),
            ]],
        ]
    );
    assert!(
        first
            .additional_ensuring
            .iter()
            .all(|area| area.holes().is_empty())
    );
    assert!(
        first
            .additional_ensuring
            .iter()
            .all(|area| area.area() > 0.0)
    );
    assert_eq!(surface_snapshot(&fill_surfaces), before);
    assert_eq!(snapshot(&ensuring), ensuring_before);
    assert!(
        fill_surfaces
            .iter()
            .zip(contour_ptrs)
            .all(|(surface, pointer)| surface.as_parts().1.contour().points().as_ptr() == pointer)
    );
    assert!(
        ensuring
            .iter()
            .zip(ensuring_ptrs)
            .all(|(polygon, pointer)| polygon.points().as_ptr() == pointer)
    );
}

#[test]
fn task22o66_real_empty_inputs_return_empty_success() {
    assert_eq!(
        prepare_region_bridge_ensuring_areas(&[], &[], flow(0.000_01), CoordinateScale::Normal,),
        Ok(RegionBridgeEnsuringAreas {
            near_perimeters: Vec::new(),
            additional_ensuring: Vec::new(),
        })
    );
}

#[test]
fn task22o66_miter_probe() {
    let surfaces = [surface(
        RegionSurfaceKind::Internal,
        expolygon(polygon(&[(0, 0), (1_000, 0), (500, 207)]), Vec::new()),
    )];
    let output = prepare_region_bridge_ensuring_areas(
        &surfaces,
        &[rectangle(-100, -100, 400, 400)],
        flow(0.000_03),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        snapshot(&output.near_perimeters),
        vec![
            vec![(1011, 6), (500, 218), (-11, 6), (-8, -10), (1008, -10)],
            vec![(96, 19), (500, 186), (904, 19)],
        ]
    );
}

#[test]
fn task22o66_natural_union_and_intersection_range_errors_preserve_inputs() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let invalid_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        expolygon(rectangle(high + 1, 0, high + 101, 100), Vec::new()),
    )];
    let invalid_before = surface_snapshot(&invalid_surfaces);
    assert_eq!(
        prepare_region_bridge_ensuring_areas(
            &invalid_surfaces,
            &[],
            flow(0.000_01),
            CoordinateScale::Normal,
        )
        .unwrap_err(),
        ClipperError::CoordinateOutOfRange
    );
    assert_eq!(surface_snapshot(&invalid_surfaces), invalid_before);

    let surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        expolygon(rectangle(0, 0, 100, 100), Vec::new()),
    )];
    let invalid_ensuring = vec![polygon(&[(high + 1, 0), (high, 10), (high - 1, 0)])];
    let surfaces_before = surface_snapshot(&surfaces);
    let ensuring_before = snapshot(&invalid_ensuring);
    assert_eq!(
        prepare_region_bridge_ensuring_areas(
            &surfaces,
            &invalid_ensuring,
            flow(0.000_01),
            CoordinateScale::Normal,
        )
        .unwrap_err(),
        ClipperError::CoordinateOutOfRange
    );
    assert_eq!(surface_snapshot(&surfaces), surfaces_before);
    assert_eq!(snapshot(&invalid_ensuring), ensuring_before);
}
