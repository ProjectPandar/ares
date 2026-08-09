use super::*;
use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, difference_open_polylines, offset_paths,
};

#[test]
#[should_panic(expected = "At least one expansion zone must exist!")]
fn task22o39_empty_zone_assertion_precedes_zero_bridges_and_invalid_anchor_geometry() {
    let anchors = vec![seed(
        u32::MAX,
        u32::MAX,
        &[(i64::MIN, i64::MAX), (i64::MAX, i64::MIN)],
    )];
    DETECT(&anchors, &mut [], &[], CoordinateScale::Normal).unwrap();
}

#[test]
fn task22o39_nonempty_zone_with_zero_bridges_does_not_consume_anchors() {
    let anchors = vec![seed(
        u32::MAX,
        u32::MAX,
        &[(i64::MIN, i64::MAX), (i64::MAX, i64::MIN)],
    )];
    let snapshot = anchors.clone();

    DETECT(
        &anchors,
        &mut [],
        &[zone(Vec::new())],
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(anchors, snapshot);
}

#[test]
fn task22o39_offset_error_is_direct_and_first_bridge_retains_all_owned_and_borrowed_state() {
    let invalid_anchor = invalid_triangle();
    let direct = offset_paths(
        std::slice::from_ref(invalid_anchor.contour()),
        100.0,
        JoinType::Miter,
        3.0,
    );
    assert_eq!(direct, Err(ClipperError::CoordinateOutOfRange));

    let bridge_geometry = rectangle(0, 0, 1000, 1000);
    let bridge_pointer = bridge_geometry.contour().points().as_ptr();
    let bridge_snapshot = snapshots(std::slice::from_ref(&bridge_geometry));
    let zone_snapshot = snapshots(std::slice::from_ref(&invalid_anchor));
    let anchors = vec![seed(0, 0, &[(i64::MIN, i64::MAX)])];
    let anchor_snapshot = anchors.clone();
    let zones = vec![zone(vec![invalid_anchor])];
    let mut bridges = vec![bridge(bridge_geometry, Some(7.25))];

    let result = DETECT(&anchors, &mut bridges, &zones, CoordinateScale::Normal);

    assert_eq!(result, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(angles(&bridges), vec![Some(7.25_f64.to_bits())]);
    assert_eq!(
        bridges[0].expolygon.contour().points().as_ptr(),
        bridge_pointer
    );
    assert_eq!(
        snapshots(std::slice::from_ref(&bridges[0].expolygon)),
        bridge_snapshot
    );
    assert_eq!(anchors, anchor_snapshot);
    assert_eq!(snapshots(&zones[0].expolygons), zone_snapshot);
}

#[test]
fn task22o39_later_open_difference_error_keeps_prior_commit_and_failing_later_angles() {
    let invalid = invalid_triangle();
    let invalid_paths = bridge_polygons(&invalid)
        .iter()
        .map(crate::geometry::Polygon::split_at_first_point)
        .collect::<Vec<_>>();
    assert_eq!(
        difference_open_polylines(&invalid_paths, &[]),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let first_geometry = rectangle(0, 0, 1000, 1000);
    let first_expected = manual(&first_geometry, &[], CoordinateScale::Normal)
        .unwrap()
        .angle
        .to_bits();
    let later_geometry = rectangle(4000, 0, 5000, 1000);
    let first_pointer = first_geometry.contour().points().as_ptr();
    let invalid_pointer = invalid.contour().points().as_ptr();
    let later_pointer = later_geometry.contour().points().as_ptr();
    let mut bridges = vec![
        bridge(first_geometry, Some(-1.0)),
        bridge(invalid, Some(2.5)),
        bridge(later_geometry, Some(9.75)),
    ];
    let geometry_snapshot = bridges
        .iter()
        .map(|bridge| snapshots(std::slice::from_ref(&bridge.expolygon)).remove(0))
        .collect::<Vec<_>>();

    let result = DETECT(
        &[],
        &mut bridges,
        &[zone(Vec::new())],
        CoordinateScale::Normal,
    );

    assert_eq!(result, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(
        angles(&bridges),
        vec![
            Some(first_expected),
            Some(2.5_f64.to_bits()),
            Some(9.75_f64.to_bits()),
        ]
    );
    assert_eq!(
        [
            bridges[0].expolygon.contour().points().as_ptr(),
            bridges[1].expolygon.contour().points().as_ptr(),
            bridges[2].expolygon.contour().points().as_ptr(),
        ],
        [first_pointer, invalid_pointer, later_pointer]
    );
    assert_eq!(
        bridges
            .iter()
            .map(|bridge| snapshots(std::slice::from_ref(&bridge.expolygon)).remove(0))
            .collect::<Vec<_>>(),
        geometry_snapshot
    );
}

#[test]
fn task22o39_trusted_signed_boundary_cast_and_wrapping_local_index_may_panic() {
    let geometry = rectangle(0, 0, 1000, 1000);
    let pointer = geometry.contour().points().as_ptr();
    let mut bridges = vec![bridge(geometry, Some(4.5))];
    let anchors = vec![seed(0, u32::MAX - 1, &[(1, 2)])];
    let zones = vec![zone(vec![rectangle(10, 10, 20, 20)])];

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DETECT(&anchors, &mut bridges, &zones, CoordinateScale::Normal)
    }));

    assert!(result.is_err());
    assert_eq!(angles(&bridges), vec![Some(4.5_f64.to_bits())]);
    assert_eq!(bridges[0].expolygon.contour().points().as_ptr(), pointer);
}
