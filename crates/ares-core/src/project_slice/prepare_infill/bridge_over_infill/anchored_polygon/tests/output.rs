use crate::geometry::{
    ClipperError, CoordinateScale, JoinType, raw_offset_paths, union_safety_offset_polygons,
};

use super::super::construct_anchored_polygon;
use super::{bridge_flow, flow, line, polygon};

fn normal_fixture() -> (Vec<crate::geometry::Polygon>, Vec<crate::geometry::Line>) {
    (
        vec![polygon(&[
            (0, 0),
            (2_000_000, 0),
            (2_000_000, 1_600_000),
            (0, 1_600_000),
        ])],
        vec![
            line(-500_000, -300_000, 2_500_000, -300_000),
            line(-500_000, 1_900_000, 2_500_000, 1_900_000),
        ],
    )
}

#[test]
fn task22o53_complete_rotated_oracle_is_ordered_repeatable_and_nonmutating() {
    let (area, anchors) = normal_fixture();
    let before_area = area.clone();
    let before_anchors = anchors.clone();
    let bridge_flow = bridge_flow(0.4_f32);
    let run = || {
        construct_anchored_polygon(&area, &anchors, bridge_flow, 0.37, CoordinateScale::Normal)
            .unwrap()
    };
    let expected = vec![polygon(&[
        (230_279, -4),
        (328_298, -444_660),
        (2_000_010, 203_736),
        (2_000_009, 686_405),
        (2_000_009, 1_169_070),
        (1_866_632, 1_600_002),
        (1_768_616, 2_044_649),
        (1_687_250, 2_254_433),
        (-81_377, 1_568_448),
        (-10, 1_358_666),
        (-10, 876_006),
        (-10, 393_340),
    ])];
    assert_eq!(run(), expected);
    assert_eq!(run(), expected);
    assert_eq!(area, before_area);
    assert_eq!(anchors, before_anchors);
}

#[test]
fn task22o53_axis_aligned_and_large_bed_outputs_match_pinned_oracle() {
    let (area, anchors) = normal_fixture();
    assert_eq!(
        construct_anchored_polygon(
            &area,
            &anchors,
            bridge_flow(0.4_f32),
            std::f64::consts::PI * 0.5,
            CoordinateScale::Normal,
        )
        .unwrap(),
        vec![polygon(&[
            (1_800_010, 2_300_010),
            (-10, 2_300_010),
            (-10, -700_009),
            (1_800_010, -700_009),
        ])]
    );

    let area = vec![polygon(&[
        (0, 0),
        (200_000, 0),
        (200_000, 160_000),
        (0, 160_000),
    ])];
    let anchors = vec![
        line(-50_000, -30_000, 250_000, -30_000),
        line(-50_000, 190_000, 250_000, 190_000),
    ];
    assert_eq!(
        construct_anchored_polygon(
            &area,
            &anchors,
            bridge_flow(0.4_f32),
            0.37,
            CoordinateScale::LargeBed,
        )
        .unwrap(),
        vec![polygon(&[
            (23_020, -4),
            (32_823, -44_479),
            (200_009, 20_367),
            (200_009, 68_639),
            (200_009, 116_908),
            (186_672, 160_002),
            (176_871, 204_467),
            (168_731, 225_455),
            (-8_150, 156_849),
            (-10, 135_864),
            (-10, 87_600),
            (-10, 39_331),
        ])]
    );
}

#[test]
fn task22o53_multiple_sections_preserve_two_path_output_order() {
    let area = vec![
        polygon(&[(0, 0), (1_000_000, 0), (1_000_000, 400_000), (0, 400_000)]),
        polygon(&[
            (0, 1_200_000),
            (1_000_000, 1_200_000),
            (1_000_000, 1_600_000),
            (0, 1_600_000),
        ]),
    ];
    let anchors = vec![
        line(-200_000, -300_000, 1_200_000, -300_000),
        line(-200_000, 600_000, 1_200_000, 600_000),
        line(-200_000, 1_000_000, 1_200_000, 1_000_000),
        line(-200_000, 1_900_000, 1_200_000, 1_900_000),
    ];
    assert_eq!(
        construct_anchored_polygon(
            &area,
            &anchors,
            bridge_flow(0.1_f32),
            std::f64::consts::PI * 0.5,
            CoordinateScale::Normal,
        )
        .unwrap(),
        vec![
            polygon(&[
                (975_010, 2_000_010),
                (-10, 2_000_010),
                (-10, 899_990),
                (975_010, 899_990),
            ]),
            polygon(&[
                (975_010, 700_010),
                (-10, 700_010),
                (-10, -400_010),
                (975_010, -400_010),
            ]),
        ]
    );
}

#[test]
fn task22o53_valid_narrow_area_has_empty_output() {
    let area = vec![polygon(&[
        (0, 0),
        (100_000, 0),
        (100_000, 1_600_000),
        (0, 1_600_000),
    ])];
    let anchors = normal_fixture().1;
    assert!(
        construct_anchored_polygon(
            &area,
            &anchors,
            bridge_flow(0.4_f32),
            std::f64::consts::PI * 0.5,
            CoordinateScale::Normal,
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn task22o53_flat_paths_safety_union_matches_pinned_topology_and_order() {
    let input = vec![
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        polygon(&[(50, 0), (150, 0), (150, 100), (50, 100)]),
    ];
    assert_eq!(
        union_safety_offset_polygons(&input).unwrap(),
        vec![polygon(&[(-10, -10), (160, -10), (160, 110), (-10, 110)])]
    );
}

#[test]
fn task22o53_flat_paths_preserve_hole_and_component_order() {
    let input = vec![
        polygon(&[(0, 0), (200, 0), (200, 200), (0, 200)]),
        polygon(&[(50, 50), (50, 150), (150, 150), (150, 50)]),
        polygon(&[(300, 0), (400, 0), (400, 100), (300, 100)]),
    ];
    let flat = union_safety_offset_polygons(&input).unwrap();
    assert_eq!(
        flat,
        vec![
            polygon(&[(210, 210), (-10, 210), (-10, -10), (210, -10)]),
            polygon(&[(60, 60), (60, 140), (140, 140), (140, 60)]),
            polygon(&[(410, 110), (290, 110), (290, -10), (410, -10)]),
        ]
    );
    assert_ne!(
        flat,
        raw_offset_paths(&input, 10.0, JoinType::Miter, 3.0).unwrap()
    );
}

#[test]
fn task22o53_final_safety_union_error_is_atomic() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let area = vec![polygon(&[
        (0, high - 2_000),
        (200, high - 2_000),
        (200, high - 1_200),
        (0, high - 1_200),
    ])];
    let anchors = vec![
        line(-100, high - 1_800, 300, high - 1_800),
        line(-100, high - 600, 300, high - 600),
    ];
    let before_area = area.clone();
    let before_anchors = anchors.clone();
    assert_eq!(
        construct_anchored_polygon(
            &area,
            &anchors,
            flow(0.001_f32, 0.000_1_f32),
            std::f64::consts::PI * 0.5,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(area, before_area);
    assert_eq!(anchors, before_anchors);
}
