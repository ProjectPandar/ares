use super::helpers::{expolygon, params, snapshots};
use crate::geometry::region_expansion::wave_seeds;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, RegionExpansionParameters, propagate_waves,
    propagate_waves_from_sources, propagate_waves_from_sources_with_steps,
};

type Snapshot = (u32, u32, Vec<(i64, i64)>);

#[rustfmt::skip]
const SMALL_OUTER: &[(i64, i64)] = &[(33, 20), (33, 30), (30, 33), (20, 33), (17, 30), (17, 20), (20, 17), (30, 17)];
#[rustfmt::skip]
const SMALL_INNER: &[(i64, i64)] = &[(21, 21), (21, 29), (29, 29), (29, 21)];
#[rustfmt::skip]
const LARGE_OUTER: &[(i64, i64)] = &[(233, 220), (233, 230), (230, 233), (220, 233), (217, 230), (217, 220), (220, 217), (230, 217)];
#[rustfmt::skip]
const LARGE_INNER: &[(i64, i64)] = &[(221, 221), (221, 229), (229, 229), (229, 221)];
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

fn explicit_pipeline(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    params: &RegionExpansionParameters,
    sorted: bool,
    scale: CoordinateScale,
) -> Vec<Snapshot> {
    let seeds = wave_seeds(src, boundary, params.tiny_expansion, sorted, scale).unwrap();
    snapshots(&propagate_waves(&seeds, boundary, params).unwrap())
}

fn snapshot(src: u32, boundary: u32, points: &[(i64, i64)]) -> Snapshot {
    (src, boundary, points.to_vec())
}

#[test]
fn parameter_entry_matches_complete_sorted_pipeline() {
    let src = [square(20, 30)];
    let boundary = [square(0, 100)];
    let params = params((2.0, 2.0, 0, 4.0, 0.25, 0.0));
    let expected = vec![snapshot(0, 0, SMALL_OUTER), snapshot(0, 0, SMALL_INNER)];
    let actual = snapshots(
        &propagate_waves_from_sources(&src, &boundary, &params, CoordinateScale::Normal).unwrap(),
    );
    assert_eq!(actual, expected);
    assert_eq!(
        explicit_pipeline(&src, &boundary, &params, true, CoordinateScale::Normal),
        expected
    );
}

#[test]
fn parameter_entry_requires_sorted_discovery_with_complete_results() {
    let src = [square(220, 230), square(20, 30)];
    let boundary = [square(0, 100), square(200, 300)];
    let params = params((2.0, 2.0, 0, 4.0, 0.25, 0.0));
    let sorted_literal = vec![
        snapshot(1, 0, SMALL_OUTER),
        snapshot(1, 0, SMALL_INNER),
        snapshot(0, 1, LARGE_OUTER),
        snapshot(0, 1, LARGE_INNER),
    ];
    let unsorted_literal = vec![
        snapshot(0, 1, LARGE_OUTER),
        snapshot(0, 1, LARGE_INNER),
        snapshot(1, 0, SMALL_OUTER),
        snapshot(1, 0, SMALL_INNER),
    ];
    let sorted = explicit_pipeline(&src, &boundary, &params, true, CoordinateScale::Normal);
    let unsorted = explicit_pipeline(&src, &boundary, &params, false, CoordinateScale::Normal);
    let actual = snapshots(
        &propagate_waves_from_sources(&src, &boundary, &params, CoordinateScale::Normal).unwrap(),
    );
    assert_eq!(actual, sorted);
    assert_eq!(sorted, sorted_literal);
    assert_eq!(unsorted, unsorted_literal);
    assert_ne!(actual, unsorted);
}

#[test]
fn entries_preserve_empty_preconditions_and_construction_order() {
    let params = params((2.0, 2.0, 0, 4.0, 0.25, 0.0));
    assert_eq!(
        propagate_waves_from_sources(&[], &[square(0, 100)], &params, CoordinateScale::Normal),
        Ok(vec![])
    );
    assert_eq!(
        propagate_waves_from_sources(&[square(20, 30)], &[], &params, CoordinateScale::Normal),
        Ok(vec![])
    );
    for tiny_expansion in [0.0, -1.0, f32::NAN] {
        let invalid = RegionExpansionParameters {
            tiny_expansion,
            ..params
        };
        assert!(
            std::panic::catch_unwind(|| {
                propagate_waves_from_sources(&[], &[], &invalid, CoordinateScale::Normal)
            })
            .is_err()
        );
    }
    for expansion in [0.0, -1.0, f32::NAN] {
        assert!(
            std::panic::catch_unwind(|| {
                propagate_waves_from_sources_with_steps(
                    &[],
                    &[],
                    expansion,
                    1.0,
                    1,
                    CoordinateScale::Normal,
                )
            })
            .is_err()
        );
    }
    for expansion_step in [0.0, -1.0, f32::NAN] {
        assert!(
            std::panic::catch_unwind(|| {
                propagate_waves_from_sources_with_steps(
                    &[],
                    &[],
                    1.0,
                    expansion_step,
                    1,
                    CoordinateScale::Normal,
                )
            })
            .is_err()
        );
    }
    assert!(
        std::panic::catch_unwind(|| {
            propagate_waves_from_sources_with_steps(&[], &[], 1.0, 1.0, 0, CoordinateScale::Normal)
        })
        .is_err()
    );
}

#[test]
fn discovery_and_propagation_errors_escape_directly_in_order() {
    const OUTSIDE: i64 = 0x4000_0000_0000_0000;
    let discovery_error_params = params((100.0, 100.0, 0, 500.0, 25.0, 0.1));
    let invalid = [expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        vec![],
    )];
    assert_eq!(
        propagate_waves_from_sources(
            &[square(20, 30)],
            &invalid,
            &discovery_error_params,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
    let boundary = [square(0, 1000)];
    let source = [square(200, 300)];
    let propagation_error_params = params((100.0, OUTSIDE as f32, 1, 500.0, f64::MAX, 0.1));
    let seeds = wave_seeds(
        &source,
        &boundary,
        propagation_error_params.tiny_expansion,
        true,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(!seeds.is_empty());
    assert_eq!(
        propagate_waves(&seeds, &boundary, &propagation_error_params),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        propagate_waves_from_sources(
            &source,
            &boundary,
            &propagation_error_params,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert!(
        std::panic::catch_unwind(|| {
            propagate_waves_from_sources_with_steps(
                &[square(20, 30)],
                &invalid,
                0.0,
                1.0,
                1,
                CoordinateScale::Normal,
            )
        })
        .is_err()
    );
}

fn scalar_case(scale: CoordinateScale, literal: &[(i64, i64)]) -> Vec<Snapshot> {
    let src = [square(200_000, 300_000)];
    let boundary = [square(0, 1_000_000)];
    let scalar = snapshots(
        &propagate_waves_from_sources_with_steps(&src, &boundary, 100_000.0, 10_000.0, 5, scale)
            .unwrap(),
    );
    let params = RegionExpansionParameters::build(100_000.0, 10_000.0, 5, scale);
    let parameter =
        snapshots(&propagate_waves_from_sources(&src, &boundary, &params, scale).unwrap());
    assert_eq!(scalar, parameter);
    assert_eq!(scalar, vec![snapshot(0, 0, literal)]);
    scalar
}

#[test]
fn scalar_entry_builds_once_delegates_and_preserves_complete_dual_scale_vectors() {
    let normal = scalar_case(CoordinateScale::Normal, NORMAL_POINTS);
    let large_bed = scalar_case(CoordinateScale::LargeBed, LARGE_BED_POINTS);
    assert!(!normal.is_empty());
    assert!(!large_bed.is_empty());
    assert_ne!(normal, large_bed);
}
