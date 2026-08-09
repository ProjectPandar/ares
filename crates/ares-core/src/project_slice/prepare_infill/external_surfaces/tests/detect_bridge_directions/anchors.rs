use super::*;
use crate::geometry::CoordinateScale;

#[test]
fn task22o39_repeated_boundaries_ignore_seed_paths_and_extract_contour_then_hole() {
    let geometry = expolygon(
        &[(0, 0), (1000, 0), (1000, 1000), (0, 1000)],
        vec![polygon(&[(200, 200), (200, 800), (800, 800), (800, 200)])],
    );
    let contour_pointer = geometry.contour().points().as_ptr();
    let hole_pointers = geometry
        .holes()
        .iter()
        .map(|hole| hole.points().as_ptr())
        .collect::<Vec<_>>();
    let anchor_geometry = expolygon(
        &[(400, -200), (600, -200), (600, 200), (400, 200)],
        vec![polygon(&[(450, -100), (450, 100), (550, 100), (550, -100)])],
    );
    let selected = bridge_polygons(&anchor_geometry);
    assert_eq!(
        polygon_points(&selected),
        vec![
            vec![(400, -200), (600, -200), (600, 200), (400, 200)],
            vec![(450, -100), (450, 100), (550, 100), (550, -100)],
        ]
    );
    let expected = manual(&geometry, &selected, CoordinateScale::Normal).unwrap();
    let anchors = vec![
        seed(0, 0, &[(i64::MIN, i64::MAX)]),
        seed(0, 0, &[(17, 23), (29, 31)]),
    ];
    let anchor_snapshot = anchors.clone();
    let zone_geometry_snapshot = snapshots(std::slice::from_ref(&anchor_geometry));
    let mut bridges = vec![bridge(geometry, Some(-8.0))];
    let zones = vec![zone(vec![anchor_geometry])];

    assert_eq!(
        polygon_points(&expected.expanded),
        vec![vec![(700, 300), (300, 300), (300, -300), (700, -300)]]
    );
    assert_eq!(
        fragment_points(&expected.fragments),
        vec![
            vec![(700, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)],
            vec![(700, 200), (800, 200), (800, 800), (200, 800), (200, 200)],
            vec![(200, 200), (300, 200)],
            vec![(0, 0), (300, 0)],
        ]
    );
    assert_eq!(
        line_points(&expected.lines),
        vec![
            ((700, 0), (1000, 0)),
            ((1000, 0), (1000, 1000)),
            ((1000, 1000), (0, 1000)),
            ((0, 1000), (0, 0)),
            ((700, 200), (800, 200)),
            ((800, 200), (800, 800)),
            ((800, 800), (200, 800)),
            ((200, 800), (200, 200)),
            ((200, 200), (300, 200)),
            ((0, 0), (300, 0)),
        ]
    );
    assert_eq!(
        (
            expected.direction.0.to_bits(),
            expected.direction.1.to_bits()
        ),
        (0x0000_0000_0000_0000, 0xbff0_0000_0000_0000)
    );
    assert_eq!(expected.cost.to_bits(), 0x40a2_c000_0000_0000);
    assert_eq!(expected.angle.to_bits(), 0x3ff9_21fb_5444_2d18);
    DETECT(&anchors, &mut bridges, &zones, CoordinateScale::Normal).unwrap();

    assert_eq!(anchors, anchor_snapshot);
    assert_eq!(snapshots(&zones[0].expolygons), zone_geometry_snapshot);
    assert_eq!(angles(&bridges), vec![Some(expected.angle.to_bits())]);
    assert_eq!(
        (
            bridges[0].expolygon.contour().points().as_ptr(),
            bridges[0]
                .expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().as_ptr())
                .collect::<Vec<_>>(),
        ),
        (contour_pointer, hole_pointers)
    );
}

#[test]
fn task22o39_multiple_bridges_rebase_global_boundaries_through_leading_empty_zones() {
    let geometries = vec![
        rectangle(0, 0, 1000, 1000),
        rectangle(2000, 0, 3000, 1000),
        rectangle(4000, 0, 5000, 1000),
    ];
    let area_a = rectangle(400, -200, 600, 200);
    let area_b = rectangle(2200, -200, 2400, 200);
    let area_c = rectangle(2600, -200, 2800, 200);
    let manual_expected = vec![
        manual(
            &geometries[0],
            std::slice::from_ref(area_a.contour()),
            CoordinateScale::Normal,
        )
        .unwrap()
        .angle
        .to_bits(),
        manual(
            &geometries[1],
            std::slice::from_ref(area_c.contour()),
            CoordinateScale::Normal,
        )
        .unwrap()
        .angle
        .to_bits(),
        manual(&geometries[2], &[], CoordinateScale::Normal)
            .unwrap()
            .angle
            .to_bits(),
    ];
    let expected = [0x3ff9_21fb_5444_2d18; 3];
    assert_eq!(manual_expected.as_slice(), &expected);
    let anchors = vec![
        seed(0, 0, &[(1, 1)]),
        seed(0, 99, &[(2, 2)]),
        seed(1, 2, &[(3, 3)]),
    ];
    let zones = vec![
        zone(Vec::new()),
        zone(vec![area_a]),
        zone(vec![area_b, area_c]),
    ];
    let mut bridges = geometries
        .into_iter()
        .enumerate()
        .map(|(index, geometry)| bridge(geometry, Some(-(index as f64) - 1.0)))
        .collect::<Vec<_>>();

    DETECT(&anchors, &mut bridges, &zones, CoordinateScale::Normal).unwrap();

    assert_eq!(
        angles(&bridges),
        expected.into_iter().map(Some).collect::<Vec<_>>()
    );
}

#[test]
fn task22o39_forward_cursor_does_not_sort_search_ahead_or_restart_for_earlier_sources() {
    let first_geometry = rectangle(0, 0, 1000, 1000);
    let second_geometry = rectangle(2000, 0, 3000, 1000);
    let anchor_geometry = rectangle(2400, -200, 2600, 200);
    let first_expected = manual(&first_geometry, &[], CoordinateScale::Normal)
        .unwrap()
        .angle
        .to_bits();
    let second_expected = manual(
        &second_geometry,
        std::slice::from_ref(anchor_geometry.contour()),
        CoordinateScale::Normal,
    )
    .unwrap()
    .angle
    .to_bits();
    let anchors = vec![seed(1, 0, &[(91, 92)]), seed(0, 0, &[(93, 94)])];
    let mut bridges = vec![bridge(first_geometry, None), bridge(second_geometry, None)];

    DETECT(
        &anchors,
        &mut bridges,
        &[zone(vec![anchor_geometry])],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        angles(&bridges),
        vec![Some(first_expected), Some(second_expected)]
    );
}

#[test]
fn task22o39_nonconsecutive_duplicate_boundary_is_processed_in_supplied_order() {
    let geometry = rectangle(0, 0, 1000, 1000);
    let area_a = rectangle(400, -200, 600, 200);
    let area_b = rectangle(-200, 400, 200, 600);
    let selected = vec![
        area_a.contour().clone(),
        area_b.contour().clone(),
        area_a.contour().clone(),
    ];
    let expected = manual(&geometry, &selected, CoordinateScale::Normal)
        .unwrap()
        .angle
        .to_bits();
    let anchors = vec![
        seed(0, 0, &[(1, 2)]),
        seed(0, 1, &[(3, 4)]),
        seed(0, 0, &[(5, 6)]),
    ];
    let mut bridges = vec![bridge(geometry, None)];

    DETECT(
        &anchors,
        &mut bridges,
        &[zone(vec![area_a, area_b])],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(angles(&bridges), vec![Some(expected)]);
}

#[test]
fn task22o39_forward_cursor_continues_to_later_source_and_preserves_error_commit_order() {
    let first_geometry = rectangle(0, 0, 1000, 1000);
    let second_geometry = rectangle(2000, 0, 3000, 1000);
    let first_pointer = first_geometry.contour().points().as_ptr();
    let second_pointer = second_geometry.contour().points().as_ptr();
    let first_anchor = rectangle(400, -200, 600, 200);
    let first_expected = manual(
        &first_geometry,
        std::slice::from_ref(first_anchor.contour()),
        CoordinateScale::Normal,
    )
    .unwrap()
    .angle
    .to_bits();
    let anchors = vec![seed(0, 0, &[(1, 2)]), seed(1, 1, &[(3, 4)])];
    let mut bridges = vec![
        bridge(first_geometry, Some(-1.0)),
        bridge(second_geometry, Some(7.25)),
    ];

    let result = DETECT(
        &anchors,
        &mut bridges,
        &[zone(vec![first_anchor, invalid_triangle()])],
        CoordinateScale::Normal,
    );

    assert_eq!(result, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(
        angles(&bridges),
        vec![Some(first_expected), Some(7.25_f64.to_bits())]
    );
    assert_eq!(
        [
            bridges[0].expolygon.contour().points().as_ptr(),
            bridges[1].expolygon.contour().points().as_ptr(),
        ],
        [first_pointer, second_pointer]
    );
}

#[test]
fn task22o39_supplied_order_does_not_sort_later_invalid_earlier_source() {
    let first = rectangle(0, 0, 1000, 1000);
    let second = rectangle(2000, 0, 3000, 1000);
    let pointers = [
        first.contour().points().as_ptr(),
        second.contour().points().as_ptr(),
    ];
    let valid = rectangle(2400, -200, 2600, 200);
    let angle = |geometry, anchors| {
        manual(geometry, anchors, CoordinateScale::Normal)
            .unwrap()
            .angle
            .to_bits()
    };
    let expected = [
        angle(&first, &[]),
        angle(&second, std::slice::from_ref(valid.contour())),
    ];
    assert_eq!(expected, [0x3ff9_21fb_5444_2d18; 2]);
    let anchors = vec![seed(1, 1, &[(1, 2)]), seed(0, 0, &[(3, 4)])];
    let mut bridges = vec![bridge(first, Some(-1.0)), bridge(second, Some(-2.0))];

    assert_eq!(
        DETECT(
            &anchors,
            &mut bridges,
            &[zone(vec![invalid_triangle(), valid])],
            CoordinateScale::Normal,
        ),
        Ok(())
    );
    assert_eq!(angles(&bridges), expected.map(Some).to_vec());
    assert_eq!(
        bridges
            .iter()
            .map(|bridge| bridge.expolygon.contour().points().as_ptr())
            .collect::<Vec<_>>(),
        pointers.to_vec()
    );
}
