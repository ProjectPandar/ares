use super::{FillConnectionParams, KSR_ANCHOR, KSR_SPACING, connect_infill, point};
use crate::geometry::{CoordinateScale, ExPolygon, Polygon, Polyline};

fn rectangle(width: i64, height: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            point(0, 0),
            point(width, 0),
            point(width, height),
            point(0, height),
        ]),
        Vec::new(),
    )
}

fn params() -> FillConnectionParams {
    FillConnectionParams {
        anchor_length: KSR_ANCHOR,
        anchor_length_max: 20.0,
        multiline: 1,
        dont_sort: false,
    }
}

#[test]
fn task22o44_distinct_merge_matches_pinned_orca() {
    let actual = connect_infill(
        vec![
            Polyline::new(vec![point(0, 2_000_000), point(12_000_000, 3_000_000)]),
            Polyline::new(vec![point(0, 6_000_000), point(12_000_000, 10_000_000)]),
        ],
        &rectangle(12_000_000, 15_000_000),
        KSR_SPACING,
        params(),
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(
        actual,
        vec![Polyline::new(vec![
            point(12_000_000, 11_628_318),
            point(12_000_000, 10_000_000),
            point(0, 6_000_000),
            point(0, 2_000_000),
            point(12_000_000, 3_000_000),
            point(12_000_000, 1_371_681),
        ])]
    );
}

#[test]
fn task22o44_large_bed_single_hook_matches_pinned_orca() {
    let actual = connect_infill(
        vec![Polyline::new(vec![
            point(0, 199_999),
            point(1_200_000, 300_000),
        ])],
        &rectangle(1_200_000, 799_999),
        KSR_SPACING,
        params(),
        CoordinateScale::LargeBed,
    )
    .unwrap();

    assert_eq!(
        actual,
        vec![Polyline::new(vec![
            point(0, 362_830),
            point(0, 199_999),
            point(1_200_000, 300_000),
            point(1_200_000, 462_831),
        ])]
    );
}
