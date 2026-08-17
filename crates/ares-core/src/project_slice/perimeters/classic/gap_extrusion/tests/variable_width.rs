use super::{flow, thick};
use crate::{
    geometry::CoordinateScale,
    project_slice::perimeters::classic::{
        gap_extrusion::{GapFillEntity, variable_width},
        materialize::ExtrusionRole,
    },
};

#[test]
fn task22o14_variable_width_empty_open_and_closed_entities_are_literal() {
    let empty = variable_width::convert(&[], flow(), CoordinateScale::Normal).unwrap();
    assert!(empty.entities.is_empty());

    let open = variable_width::convert(
        &[thick(&[(0, 0), (200, 0)], &[400_000.0, 400_000.0])],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let GapFillEntity::Path(path) = &open.entities[0] else {
        panic!("expected path")
    };
    assert_eq!(path.role, ExtrusionRole::GapFill);
    assert_eq!(
        path.polyline
            .points
            .iter()
            .map(|p| (p.x, p.y, p.z))
            .collect::<Vec<_>>(),
        vec![(0, 0, 0), (200, 0, 0)]
    );
    assert_eq!(path.height.to_bits(), 0x3e4c_cccd);
    assert_eq!(path.width.to_bits(), 0x3ee2_c676);
    assert_eq!(path.mm3_per_mm.to_bits(), 0x3fb4_7ae1_6000_0000);

    let closed = variable_width::convert(
        &[thick(&[(0, 0), (200, 0), (0, 0)], &[400_000.0; 4])],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(matches!(&closed.entities[..], [GapFillEntity::Loop(paths)] if paths.len() == 1));
}

#[test]
fn task22o14_variable_width_tolerance_split_and_reversed_widths_are_stable() {
    let split = variable_width::convert(
        &[thick(&[(0, 0), (300_000, 0)], &[300_000.0, 450_000.0])],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let forward_points = split
        .entities
        .iter()
        .map(|entity| match entity {
            GapFillEntity::Path(path) => path
                .polyline
                .points
                .iter()
                .map(|point| point.x)
                .collect::<Vec<_>>(),
            GapFillEntity::Loop(_) => panic!("split line cannot form a loop"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        forward_points,
        vec![
            vec![0, 100_000],
            vec![100_000, 200_000],
            vec![200_000, 300_000]
        ]
    );
    let forward_metadata = split
        .entities
        .iter()
        .map(|entity| match entity {
            GapFillEntity::Path(path) => (path.width.to_bits(), path.mm3_per_mm.to_bits()),
            GapFillEntity::Loop(_) => panic!("split line cannot form a loop"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        forward_metadata,
        vec![
            (0x3ebc_600e, 0x3fb0_a3d7_0000_0000),
            (0x3ed5_f9a8, 0x3fb3_3333_2000_0000),
            (0x3ee2_c676, 0x3fb4_7ae1_6000_0000),
        ]
    );

    let reversed = variable_width::convert(
        &[thick(&[(300_000, 0), (0, 0)], &[450_000.0, 300_000.0])],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let reversed_points = reversed
        .entities
        .iter()
        .map(|entity| match entity {
            GapFillEntity::Path(path) => path
                .polyline
                .points
                .iter()
                .map(|point| point.x)
                .collect::<Vec<_>>(),
            GapFillEntity::Loop(_) => panic!("reversed split line cannot form a loop"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reversed_points,
        vec![
            vec![300_000, 200_000],
            vec![200_000, 100_000],
            vec![100_000, 0]
        ]
    );
}

#[test]
fn task22o14_variable_width_epsilon_is_strict_at_both_scales() {
    for (scale, epsilon) in [
        (CoordinateScale::Normal, 100),
        (CoordinateScale::LargeBed, 10),
    ] {
        let equal = variable_width::convert(
            &[thick(&[(0, 0), (epsilon, 0)], &[40_000.0, 40_000.0])],
            flow(),
            scale,
        )
        .unwrap();
        assert!(equal.entities.is_empty());
        let above = variable_width::convert(
            &[thick(&[(0, 0), (epsilon + 1, 0)], &[40_000.0, 40_000.0])],
            flow(),
            scale,
        )
        .unwrap();
        assert_eq!(above.entities.len(), 1);
    }
}

#[test]
fn task22o14_grouped_midpoint_and_asymmetric_final_flush_are_literal() {
    let result = variable_width::convert(
        &[thick(
            &[(0, 0), (200_000, 0), (400_000, 0)],
            &[300_000.0, 340_000.0, 340_000.0, 380_000.0],
        )],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let [GapFillEntity::Path(first), GapFillEntity::Path(second)] = &result.entities[..] else {
        panic!("expected two ordered paths")
    };
    assert_eq!(
        first
            .polyline
            .points
            .iter()
            .map(|point| point.x)
            .collect::<Vec<_>>(),
        vec![0, 200_000]
    );
    assert_eq!(
        second
            .polyline
            .points
            .iter()
            .map(|point| point.x)
            .collect::<Vec<_>>(),
        vec![200_000, 400_000]
    );
    assert_eq!(first.width.to_bits(), 0x3eb9_d0b2);
    assert_eq!(first.mm3_per_mm.to_bits(), 0x3fb0_624d_c000_0000);
    assert_eq!(second.width.to_bits(), 0x3ec4_0e24);
    assert_eq!(second.mm3_per_mm.to_bits(), 0x3fb1_6872_c000_0000);
}

#[test]
fn task22o14_tolerance_equality_stays_grouped_and_above_splits() {
    let equal = variable_width::convert(
        &[thick(&[(0, 0), (300_000, 0)], &[300_000.0, 350_000.0])],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(equal.entities.len(), 1);
    let above = variable_width::convert(
        &[thick(&[(0, 0), (300_000, 0)], &[300_000.0, 350_001.0])],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(above.entities.len(), 2);
}

#[test]
fn task22o14_exact_epsilon_line_is_processed_and_split_before_final_flush() {
    let result = variable_width::convert(
        &[thick(
            &[(0, 0), (10, 0), (30, 0)],
            &[30_000.0, 40_000.0, 40_000.0, 35_000.0],
        )],
        flow(),
        CoordinateScale::LargeBed,
    )
    .unwrap();
    let [GapFillEntity::Path(path)] = &result.entities[..] else {
        panic!("expected one final path")
    };
    assert_eq!(
        path.polyline
            .points
            .iter()
            .map(|point| point.x)
            .collect::<Vec<_>>(),
        vec![0, 5, 10, 30]
    );
}

#[test]
fn task22o14_multiple_input_polylines_preserve_entity_order() {
    let result = variable_width::convert(
        &[
            thick(&[(0, 10), (200, 10)], &[400_000.0, 400_000.0]),
            thick(&[(0, 20), (200, 20)], &[400_000.0, 400_000.0]),
        ],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let ys = result
        .entities
        .iter()
        .map(|entity| match entity {
            GapFillEntity::Path(path) => path.polyline.points[0].y,
            GapFillEntity::Loop(_) => panic!("open inputs cannot form loops"),
        })
        .collect::<Vec<_>>();
    assert_eq!(ys, vec![10, 20]);
}

#[test]
fn task22o14_unequal_below_epsilon_line_contributes_actual_width_and_point() {
    let result = variable_width::convert(
        &[thick(
            &[(0, 0), (50, 0), (250, 0)],
            &[400_000.0, 430_000.0, 430_000.0, 440_000.0],
        )],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let [GapFillEntity::Path(path)] = &result.entities[..] else {
        panic!("expected one path")
    };
    assert_eq!(
        path.polyline
            .points
            .iter()
            .map(|point| point.x)
            .collect::<Vec<_>>(),
        vec![0, 50, 250]
    );
    assert_eq!(
        (path.width.to_bits(), path.mm3_per_mm.to_bits()),
        (0x3eef_1030, 0x3fb5_b574_0000_0000),
    );
}

#[test]
fn task22o14_skipped_zero_and_below_epsilon_lines_remain_in_final_group() {
    let result = variable_width::convert(
        &[thick(
            &[(0, 0), (0, 0), (50, 0), (250, 0)],
            &[
                400_000.0, 400_000.0, 400_000.0, 400_000.0, 400_000.0, 400_000.0,
            ],
        )],
        flow(),
        CoordinateScale::Normal,
    )
    .unwrap();
    let GapFillEntity::Path(path) = &result.entities[0] else {
        panic!("expected path")
    };
    assert_eq!(
        path.polyline.points.iter().map(|p| p.x).collect::<Vec<_>>(),
        vec![0, 0, 50, 250]
    );
}

#[test]
fn task22o200_variable_width_conversion_preserves_solid_infill_role() {
    let result = variable_width::convert_with_role(
        &[thick(&[(0, 0), (200, 0)], &[400_000.0, 400_000.0])],
        flow(),
        CoordinateScale::Normal,
        ExtrusionRole::SolidInfill,
    )
    .unwrap();

    let GapFillEntity::Path(path) = &result.entities[0] else {
        panic!("expected path")
    };
    assert_eq!(path.role, ExtrusionRole::SolidInfill);
}
