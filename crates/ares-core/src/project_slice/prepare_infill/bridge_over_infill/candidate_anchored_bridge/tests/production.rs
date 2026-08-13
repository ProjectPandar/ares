use crate::geometry::CoordinateScale;

use super::{
    construct_candidate_anchored_bridge, flow, polygon, polyline, snapshot_polygons,
    snapshot_polylines,
};

#[test]
fn task22o61_real_o53_composition_matches_ordered_axis_aligned_oracle() {
    let area = vec![polygon(&[
        (0, 0),
        (2_000_000, 0),
        (2_000_000, 1_600_000),
        (0, 1_600_000),
    ])];
    let boundaries = vec![
        polyline(&[(-500_000, -300_000), (2_500_000, -300_000)]),
        polyline(&[(-500_000, 1_900_000), (2_500_000, 1_900_000)]),
    ];
    let anchors = Vec::new();
    let area_before = snapshot_polygons(&area);
    let boundaries_before = snapshot_polylines(&boundaries);

    let run = || {
        construct_candidate_anchored_bridge(
            &area,
            boundaries.clone(),
            &anchors,
            &[],
            flow(),
            std::f64::consts::PI * 0.5,
            CoordinateScale::Normal,
        )
        .unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(first.boundary_polylines, boundaries);
    assert_eq!(
        first.bridging_area,
        vec![polygon(&[
            (1_800_010, 2_300_010),
            (-10, 2_300_010),
            (-10, -700_009),
            (1_800_010, -700_009),
        ])]
    );
    assert_eq!(second, first);
    assert_eq!(snapshot_polygons(&area), area_before);
    assert_eq!(snapshot_polylines(&boundaries), boundaries_before);
}

#[test]
fn task22o61_real_lightning_path_freezes_closed_expand_and_open_kernels() {
    let area = vec![polygon(&[(0, 0), (2_000_000, 0), (200_000, 1_600_000)])];
    let lightning = vec![polygon(&[
        (100_000, 100_000),
        (500_000, 100_000),
        (200_000, 500_000),
    ])];
    let boundaries = vec![
        polyline(&[(-20_000_000, -9_000_000), (20_000_000, -9_000_000)]),
        polyline(&[(-20_000_000, 10_000_000), (20_000_000, 10_000_000)]),
    ];

    let output = construct_candidate_anchored_bridge(
        &area,
        boundaries,
        &[],
        &lightning,
        flow(),
        std::f64::consts::PI * 0.5,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        snapshot_polylines(&output.boundary_polylines),
        vec![
            vec![(5_801_993, 10_000_000), (-8_827_822, 10_000_000)],
            vec![(20_000_000, -9_000_000), (-11_202_822, -9_000_000)],
        ]
    );
    assert!(!output.bridging_area.is_empty());
}

#[test]
fn task22o61_real_lightning_gate_uses_no_safety_offset() {
    let area = vec![polygon(&[
        (0, 0),
        (2_000_000, 0),
        (2_000_000, 1_600_000),
        (0, 1_600_000),
    ])];
    let lightning = vec![polygon(&[
        (2_000_005, 100_000),
        (2_500_000, 100_000),
        (2_500_000, 500_000),
        (2_000_005, 500_000),
    ])];
    let boundaries = vec![
        polyline(&[(-20_000_000, -9_000_000), (20_000_000, -9_000_000)]),
        polyline(&[(-20_000_000, 10_000_000), (20_000_000, 10_000_000)]),
    ];

    let output = construct_candidate_anchored_bridge(
        &area,
        boundaries.clone(),
        &[],
        &lightning,
        flow(),
        std::f64::consts::PI * 0.5,
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(output.boundary_polylines, boundaries);
    assert!(!output.bridging_area.is_empty());
}
