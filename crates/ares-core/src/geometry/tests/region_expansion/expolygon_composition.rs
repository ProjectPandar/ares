use super::helpers::expolygon;
use crate::geometry::region_expansion::wave_seeds;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Polygon, RegionExpansionEx,
    RegionExpansionParameters, propagate_waves_ex, propagate_waves_ex_from_sources_with_steps,
};

type ExSnapshot = (u32, u32, Vec<(i64, i64)>, Vec<Vec<(i64, i64)>>);

#[rustfmt::skip]
const SMALL_OUTER: &[(i64, i64)] = &[(32, 20), (32, 30), (30, 32), (20, 32), (18, 30), (18, 20), (20, 18), (30, 18)];
#[rustfmt::skip]
const SMALL_INNER: &[(i64, i64)] = &[(22, 22), (22, 28), (28, 28), (28, 22)];
#[rustfmt::skip]
const LARGE_OUTER: &[(i64, i64)] = &[(232, 220), (232, 230), (230, 232), (220, 232), (218, 230), (218, 220), (220, 218), (230, 218)];
#[rustfmt::skip]
const LARGE_INNER: &[(i64, i64)] = &[(222, 222), (222, 228), (228, 228), (228, 222)];
#[rustfmt::skip]
const NORMAL_POINTS: &[(i64, i64)] = &[
    (364853, 123431), (376569, 135147), (400000, 191716), (400000, 308284),
    (376569, 364853), (364853, 376569), (308284, 400000), (191716, 400000),
    (135147, 376569), (123431, 364853), (100000, 308284), (100000, 191716),
    (123431, 135147), (135147, 123431), (191716, 100000), (308284, 100000),
];
#[rustfmt::skip]
const LARGE_BED_POINTS: &[(i64, i64)] = &[
    (304397, 100114), (309031, 100570), (311339, 100912), (320427, 102720),
    (322691, 103288), (327145, 104640), (329335, 105424), (346129, 112380),
    (348232, 113374), (352337, 115567), (354339, 116766), (362045, 121914),
    (363918, 123303), (367519, 126258), (369247, 127824), (372176, 130753),
    (373742, 132481), (376697, 136082), (378086, 137955), (383234, 145661),
    (384433, 147663), (386626, 151768), (387620, 153871), (394576, 170665),
    (395360, 172855), (396712, 177309), (397280, 179573), (399088, 188661),
    (399430, 190969), (399886, 195603), (400000, 197929), (400000, 302071),
    (399886, 304397), (399430, 309031), (399088, 311339), (397280, 320427),
    (396712, 322691), (395360, 327145), (394576, 329335), (387620, 346129),
    (386626, 348232), (384433, 352337), (383234, 354339), (378086, 362045),
    (376697, 363918), (373742, 367519), (372176, 369247), (369247, 372176),
    (367519, 373742), (363918, 376697), (362045, 378086), (354339, 383234),
    (352337, 384433), (348232, 386626), (346129, 387620), (329335, 394576),
    (327145, 395360), (322691, 396712), (320427, 397280), (311339, 399088),
    (309031, 399430), (304397, 399886), (302071, 400000), (197929, 400000),
    (195603, 399886), (190969, 399430), (188661, 399088), (179573, 397280),
    (177309, 396712), (172855, 395360), (170665, 394576), (153871, 387620),
    (151768, 386626), (147663, 384433), (145661, 383234), (137955, 378086),
    (136082, 376697), (132481, 373742), (130753, 372176), (127824, 369247),
    (126258, 367519), (123303, 363918), (121914, 362045), (116766, 354339),
    (115567, 352337), (113374, 348232), (112380, 346129), (105424, 329335),
    (104640, 327145), (103288, 322691), (102720, 320427), (100912, 311339),
    (100570, 309031), (100114, 304397), (100000, 302071), (100000, 197929),
    (100114, 195603), (100570, 190969), (100912, 188661), (102720, 179573),
    (103288, 177309), (104640, 172855), (105424, 170665), (112380, 153871),
    (113374, 151768), (115567, 147663), (116766, 145661), (121914, 137955),
    (123303, 136082), (126258, 132481), (127824, 130753), (130753, 127824),
    (132481, 126258), (136082, 123303), (137955, 121914), (145661, 116766),
    (147663, 115567), (151768, 113374), (153871, 112380), (170665, 105424),
    (172855, 104640), (177309, 103288), (179573, 102720), (188661, 100912),
    (190969, 100570), (195603, 100114), (197929, 100000), (302071, 100000),
];

fn square(min: i64, max: i64) -> ExPolygon {
    expolygon(&[(min, min), (max, min), (max, max), (min, max)], vec![])
}

fn points(polygon: &Polygon) -> Vec<(i64, i64)> {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

fn snapshots(expansions: &[RegionExpansionEx]) -> Vec<ExSnapshot> {
    expansions
        .iter()
        .map(|expansion| {
            (
                expansion.src_id,
                expansion.boundary_id,
                points(expansion.expolygon.contour()),
                expansion.expolygon.holes().iter().map(points).collect(),
            )
        })
        .collect()
}

fn explicit_pipeline(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    (full_expansion, expansion_step, max_steps, scale): (f32, f32, usize, CoordinateScale),
) -> Result<Vec<RegionExpansionEx>, ClipperError> {
    let params = RegionExpansionParameters::build(full_expansion, expansion_step, max_steps, scale);
    let seeds = wave_seeds(src, boundary, params.tiny_expansion, true, scale)?;
    propagate_waves_ex(&seeds, boundary, &params)
}

#[test]
fn task22o31_empty_inputs_preserve_builder_preconditions() {
    assert_eq!(
        propagate_waves_ex_from_sources_with_steps(
            &[],
            &[square(0, 100)],
            2.0,
            2.0,
            1,
            CoordinateScale::Normal,
        ),
        Ok(Vec::new())
    );
    assert_eq!(
        propagate_waves_ex_from_sources_with_steps(
            &[square(20, 30)],
            &[],
            2.0,
            2.0,
            1,
            CoordinateScale::Normal,
        ),
        Ok(Vec::new())
    );
    for arguments in [(0.0, 1.0, 1), (1.0, 0.0, 1), (1.0, 1.0, 0)] {
        assert!(
            std::panic::catch_unwind(|| {
                propagate_waves_ex_from_sources_with_steps(
                    &[],
                    &[],
                    arguments.0,
                    arguments.1,
                    arguments.2,
                    CoordinateScale::Normal,
                )
            })
            .is_err()
        );
    }
}

#[test]
fn task22o31_single_source_keeps_natural_hole_and_matches_explicit_pipeline() {
    let src = [square(20, 30)];
    let boundary = [square(0, 100)];
    let actual = propagate_waves_ex_from_sources_with_steps(
        &src,
        &boundary,
        2.0,
        2.0,
        1,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        snapshots(&actual),
        vec![(0, 0, SMALL_OUTER.to_vec(), vec![SMALL_INNER.to_vec()],)]
    );
    assert_eq!(
        actual,
        explicit_pipeline(&src, &boundary, (2.0, 2.0, 1, CoordinateScale::Normal)).unwrap()
    );
}

#[test]
fn task22o31_sorted_discovery_preserves_complete_ids_topology_and_order() {
    let src = [square(220, 230), square(20, 30)];
    let boundary = [square(0, 100), square(200, 300)];
    let actual = propagate_waves_ex_from_sources_with_steps(
        &src,
        &boundary,
        2.0,
        2.0,
        1,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        snapshots(&actual),
        vec![
            (1, 0, SMALL_OUTER.to_vec(), vec![SMALL_INNER.to_vec()]),
            (0, 1, LARGE_OUTER.to_vec(), vec![LARGE_INNER.to_vec()]),
        ]
    );
    assert_eq!(
        actual,
        explicit_pipeline(&src, &boundary, (2.0, 2.0, 1, CoordinateScale::Normal)).unwrap()
    );
}

fn scalar_case(scale: CoordinateScale, literal: &[(i64, i64)]) -> Vec<ExSnapshot> {
    let src = [square(200_000, 300_000)];
    let boundary = [square(0, 1_000_000)];
    let actual =
        propagate_waves_ex_from_sources_with_steps(&src, &boundary, 100_000.0, 10_000.0, 5, scale)
            .unwrap();
    assert_eq!(snapshots(&actual), vec![(0, 0, literal.to_vec(), vec![])]);
    assert_eq!(
        actual,
        explicit_pipeline(&src, &boundary, (100_000.0, 10_000.0, 5, scale)).unwrap()
    );
    snapshots(&actual)
}

#[test]
fn task22o31_same_explicit_scale_reaches_build_and_discovery() {
    let normal = scalar_case(CoordinateScale::Normal, NORMAL_POINTS);
    let large_bed = scalar_case(CoordinateScale::LargeBed, LARGE_BED_POINTS);
    assert_ne!(normal, large_bed);
}

#[test]
fn task22o31_discovery_and_propagation_errors_escape_directly_in_order() {
    const OUTSIDE: i64 = 0x4000_0000_0000_0000;
    let invalid = [expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        vec![],
    )];
    assert_eq!(
        propagate_waves_ex_from_sources_with_steps(
            &[square(20, 30)],
            &invalid,
            100.0,
            100.0,
            1,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );

    let source = [square(
        -1_000_000_000_000_000_000,
        1_000_000_000_000_000_000,
    )];
    let boundary = [square(
        -4_000_000_000_000_000_000,
        4_000_000_000_000_000_000,
    )];
    let full_expansion = 6.0e18_f32;
    let params = RegionExpansionParameters::build(
        full_expansion,
        full_expansion,
        1,
        CoordinateScale::Normal,
    );
    let seeds = wave_seeds(
        &source,
        &boundary,
        params.tiny_expansion,
        true,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(!seeds.is_empty());
    assert_eq!(
        propagate_waves_ex(&seeds, &boundary, &params),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        propagate_waves_ex_from_sources_with_steps(
            &source,
            &boundary,
            full_expansion,
            full_expansion,
            1,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
