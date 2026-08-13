use super::*;

#[test]
fn task22o57_normal_epsilon_closes_gap_199_but_not_201() {
    assert_eq!(fill_components(199, CoordinateScale::Normal), 1);
    assert_eq!(fill_components(201, CoordinateScale::Normal), 2);
}

#[test]
fn task22o57_normal_disjoint_closing_matches_pinned_flat_order() {
    let fill = [
        expolygon(rectangle(0, 0, 1_000, 1_000), Vec::new()),
        expolygon(rectangle(1_201, 0, 2_201, 1_000), Vec::new()),
    ];
    let regions = [region(&[], &fill, ProcessInfillPattern::Rectilinear)];
    let output = prepare(&[], &regions, &[], 100, CoordinateScale::Normal);
    assert_eq!(
        snapshot_polygons(&output.total_fill_area),
        vec![
            vec![(1_000, 1_000), (0, 1_000), (0, 0), (1_000, 0)],
            vec![(2_201, 1_000), (1_201, 1_000), (1_201, 0), (2_201, 0)],
        ]
    );
}

#[test]
fn task22o57_large_bed_epsilon_closes_gap_19_but_not_21() {
    assert_eq!(fill_components(19, CoordinateScale::LargeBed), 1);
    assert_eq!(fill_components(21, CoordinateScale::LargeBed), 2);
}

#[test]
fn task22o57_preserves_spacing_promotion_before_multiplication_and_cast() {
    let spacing = 16_777_217;
    let expected_expand = (spacing as f64 * 1.5_f64) as f32;
    let early_f32_expand = spacing as f32 * 1.5_f32;
    assert_ne!(expected_expand.to_bits(), early_f32_expand.to_bits());
    assert_eq!(expected_expand.to_bits(), 0x4bc0_0001);

    let expected_shrink = (spacing as f64 * 4.5_f64) as f32;
    let early_f32_shrink = spacing as f32 * 4.5_f32;
    assert_ne!(expected_shrink.to_bits(), early_f32_shrink.to_bits());
    assert_eq!(expected_shrink.to_bits(), 0x4c90_0001);

    let deep = [rectangle(0, 0, 1_000_000_000, 1_000_000_000)];
    let output = prepare(&deep, &[], &[], spacing, CoordinateScale::Normal);
    let expand = i64::from(expected_expand as i32);
    let shrink = i64::from(expected_shrink as i32);
    assert_eq!(
        bounds(&output.deep_infill_area),
        (
            -expand,
            -expand,
            1_000_000_000 + expand,
            1_000_000_000 + expand
        )
    );
    assert_eq!(
        bounds(&output.internal_unsupported_area),
        (
            shrink - expand,
            shrink - expand,
            1_000_000_000 + expand - shrink,
            1_000_000_000 + expand - shrink,
        )
    );
}

#[test]
fn task22o57_closes_before_deep_intersection_and_shrinks_before_anchor_clip() {
    let surfaces = [
        surface(RegionSurfaceKind::Internal, rectangle(0, 0, 1_000, 1_000)),
        surface(
            RegionSurfaceKind::Internal,
            rectangle(1_199, 0, 2_199, 1_000),
        ),
    ];
    let regions = [region(&surfaces, &[], ProcessInfillPattern::Rectilinear)];
    let deep = [rectangle(500, -500, 1_699, 1_500)];
    let lines = [
        line(&[(-500, 500), (2_700, 500)]),
        line(&[(1_100, -500), (1_100, 1_500)]),
    ];
    let output = prepare(&deep, &regions, &lines, 100, CoordinateScale::Normal);

    assert_eq!(bounds(&output.expansion_area), (350, 0, 1_849, 1_000));
    assert_eq!(
        snapshot_polylines(&output.anchors),
        vec![
            vec![(1_100, 100), (1_100, 900)],
            vec![(1_749, 500), (450, 500)],
        ]
    );
}

#[test]
fn task22o57_matches_pinned_actual_source_ordered_context() {
    let surfaces = [
        surface(RegionSurfaceKind::Internal, rectangle(0, 0, 1_000, 1_000)),
        surface(
            RegionSurfaceKind::Internal,
            rectangle(1_199, 0, 2_199, 1_000),
        ),
        surface(RegionSurfaceKind::Top, rectangle(3_000, 0, 4_000, 1_000)),
    ];
    let fill = [expolygon(rectangle(0, -1_000, 2_199, 2_000), Vec::new())];
    let regions = [region(&surfaces, &fill, ProcessInfillPattern::Rectilinear)];
    let deep = [rectangle(500, -500, 1_699, 1_500)];
    let lines = [
        line(&[(-500, 500), (2_700, 500)]),
        line(&[(1_100, -500), (1_100, 1_500)]),
    ];
    let output = prepare(&deep, &regions, &lines, 100, CoordinateScale::Normal);

    assert_eq!(
        snapshot_polygons(&output.deep_infill_area),
        vec![vec![
            (1_849, 1_650),
            (350, 1_650),
            (350, -650),
            (1_849, -650)
        ]]
    );
    assert_eq!(
        snapshot_polygons(&output.total_top_area),
        vec![vec![(3_000, 0), (4_000, 0), (4_000, 1_000), (3_000, 1_000)]]
    );
    assert_eq!(
        snapshot_polygons(&output.expansion_area),
        vec![vec![(1_849, 1_000), (350, 1_000), (350, 0), (1_849, 0)]]
    );
    assert_eq!(
        snapshot_polygons(&output.total_fill_area),
        vec![vec![
            (2_199, 2_000),
            (0, 2_000),
            (0, -1_000),
            (2_199, -1_000)
        ]]
    );
    assert_eq!(
        snapshot_polylines(&output.anchors),
        vec![
            vec![(1_100, 100), (1_100, 900)],
            vec![(1_749, 500), (450, 500)],
        ]
    );
    assert_eq!(
        snapshot_polygons(&output.internal_unsupported_area),
        vec![vec![
            (1_399, 1_200),
            (800, 1_200),
            (800, -200),
            (1_399, -200)
        ]]
    );
}

fn fill_components(gap: i64, scale: CoordinateScale) -> usize {
    let fill = [
        expolygon(rectangle(0, 0, 1_000, 1_000), Vec::new()),
        expolygon(rectangle(1_000 + gap, 0, 2_000 + gap, 1_000), Vec::new()),
    ];
    let regions = [region(&[], &fill, ProcessInfillPattern::Rectilinear)];
    prepare(&[], &regions, &[], 100, scale)
        .total_fill_area
        .len()
}
