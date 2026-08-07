use super::helpers::expolygon;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Polygon, expand_expolygons,
    propagate_waves_from_sources_with_steps,
};

type SlotSnapshot = Vec<Vec<Vec<(i64, i64)>>>;

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

fn snapshots(slots: &[Vec<Polygon>]) -> SlotSnapshot {
    slots
        .iter()
        .map(|slot| slot.iter().map(points).collect())
        .collect()
}

fn explicit_grouping(
    src: &[ExPolygon],
    boundary: &[ExPolygon],
    (expansion, expansion_step, max_nr_steps, scale): (f32, f32, usize, CoordinateScale),
) -> Result<Vec<Vec<Polygon>>, ClipperError> {
    let records = propagate_waves_from_sources_with_steps(
        src,
        boundary,
        expansion,
        expansion_step,
        max_nr_steps,
        scale,
    )?;
    let mut output = vec![Vec::new(); src.len()];
    for record in records {
        output[record.src_id as usize].push(record.polygon);
    }
    Ok(output)
}

#[test]
fn task22o32_empty_inputs_keep_source_sized_slots_and_builder_order() {
    assert_eq!(
        expand_expolygons(&[], &[square(0, 100)], 2.0, 2.0, 1, CoordinateScale::Normal),
        Ok(Vec::new())
    );
    let src = [square(20, 30), square(120, 130), square(220, 230)];
    assert_eq!(
        expand_expolygons(&src, &[], 2.0, 2.0, 1, CoordinateScale::Normal),
        Ok(vec![Vec::new(), Vec::new(), Vec::new()])
    );

    for expansion in [0.0, -1.0, f32::NAN] {
        assert!(
            std::panic::catch_unwind(|| {
                expand_expolygons(&[], &[], expansion, 1.0, 1, CoordinateScale::Normal)
            })
            .is_err()
        );
    }
    for expansion_step in [0.0, -1.0, f32::NAN] {
        assert!(
            std::panic::catch_unwind(|| {
                expand_expolygons(&[], &[], 1.0, expansion_step, 1, CoordinateScale::Normal)
            })
            .is_err()
        );
    }
    assert!(
        std::panic::catch_unwind(|| {
            expand_expolygons(&[], &[], 1.0, 1.0, 0, CoordinateScale::Normal)
        })
        .is_err()
    );
}

#[test]
fn task22o32_single_source_keeps_every_raw_polygon_in_order() {
    let src = [square(20, 30)];
    let boundary = [square(0, 100)];
    let actual = expand_expolygons(&src, &boundary, 2.0, 2.0, 1, CoordinateScale::Normal).unwrap();
    assert_eq!(
        snapshots(&actual),
        vec![vec![SMALL_OUTER.to_vec(), SMALL_INNER.to_vec()]]
    );
    assert_eq!(
        actual,
        explicit_grouping(&src, &boundary, (2.0, 2.0, 1, CoordinateScale::Normal)).unwrap()
    );
}

#[test]
fn task22o32_boundary_first_records_fill_source_slots_without_compaction() {
    let src = [
        square(400, 410),
        square(220, 230),
        square(500, 510),
        square(20, 30),
        square(600, 610),
    ];
    let boundary = [square(0, 100), square(200, 300)];
    let records = propagate_waves_from_sources_with_steps(
        &src,
        &boundary,
        2.0,
        2.0,
        1,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        records
            .iter()
            .map(|record| (record.src_id, record.boundary_id, points(&record.polygon)))
            .collect::<Vec<_>>(),
        vec![
            (3, 0, SMALL_OUTER.to_vec()),
            (3, 0, SMALL_INNER.to_vec()),
            (1, 1, LARGE_OUTER.to_vec()),
            (1, 1, LARGE_INNER.to_vec()),
        ]
    );
    assert_eq!(
        snapshots(
            &expand_expolygons(&src, &boundary, 2.0, 2.0, 1, CoordinateScale::Normal).unwrap()
        ),
        vec![
            vec![],
            vec![LARGE_OUTER.to_vec(), LARGE_INNER.to_vec()],
            vec![],
            vec![SMALL_OUTER.to_vec(), SMALL_INNER.to_vec()],
            vec![],
        ]
    );
}

fn scale_case(scale: CoordinateScale, literal: &[(i64, i64)]) -> SlotSnapshot {
    let src = [square(200_000, 300_000)];
    let boundary = [square(0, 1_000_000)];
    let actual = expand_expolygons(&src, &boundary, 100_000.0, 10_000.0, 5, scale).unwrap();
    assert_eq!(snapshots(&actual), vec![vec![literal.to_vec()]]);
    assert_eq!(
        actual,
        explicit_grouping(&src, &boundary, (100_000.0, 10_000.0, 5, scale)).unwrap()
    );
    snapshots(&actual)
}

#[test]
fn task22o32_forwards_the_same_explicit_scale_to_scalar_propagation() {
    let normal = scale_case(CoordinateScale::Normal, NORMAL_POINTS);
    let large_bed = scale_case(CoordinateScale::LargeBed, LARGE_BED_POINTS);
    assert_ne!(normal, large_bed);
}

#[test]
fn task22o32_discovery_and_propagation_errors_escape_unchanged() {
    const OUTSIDE: i64 = 0x4000_0000_0000_0000;
    let invalid = [expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        vec![],
    )];
    assert_eq!(
        expand_expolygons(
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
    let expansion = 6.0e18_f32;
    assert_eq!(
        expand_expolygons(
            &source,
            &boundary,
            expansion,
            expansion,
            1,
            CoordinateScale::Normal,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
