use super::{FillConnectionParams, KSR_ANCHOR, KSR_SPACING, connect_infill, point};
use crate::geometry::{CoordinateScale, ExPolygon, Polygon, Polyline};

type RawPath<'a> = &'a [(i64, i64)];

fn polygon(raw: RawPath<'_>) -> Polygon {
    Polygon::new(raw.iter().map(|&(x, y)| point(x, y)).collect())
}

fn polylines(raw: &[RawPath<'_>]) -> Vec<Polyline> {
    raw.iter()
        .map(|path| Polyline::new(path.iter().map(|&(x, y)| point(x, y)).collect()))
        .collect()
}

fn boundary(width: i64, height: i64, holes: &[RawPath<'_>]) -> ExPolygon {
    ExPolygon::new(
        polygon(&[(0, 0), (width, 0), (width, height), (0, height)]),
        holes.iter().map(|hole| polygon(hole)).collect(),
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the literal Orca oracle wrapper keeps every source input dimension visible"
)]
fn assert_orca_case(
    scale: CoordinateScale,
    boundary: ExPolygon,
    input: &[RawPath<'_>],
    multiline: i32,
    dont_sort: bool,
    expected: &[RawPath<'_>],
) {
    let actual = connect_infill(
        polylines(input),
        &boundary,
        KSR_SPACING,
        FillConnectionParams {
            anchor_length: KSR_ANCHOR,
            anchor_length_max: 20.0,
            multiline,
            dont_sort,
        },
        scale,
    )
    .unwrap();

    assert_eq!(actual, polylines(expected));
}

#[test]
fn task22o44_comparator_distinct_threshold_equality_and_below_one_raw_match_orca() {
    let normal_first: RawPath<'_> = &[(0, 2_000_000), (25_000_000, 3_000_000)];
    let normal_equal: RawPath<'_> = &[(0, 22_000_000), (25_000_000, 26_000_000)];
    let normal_below: RawPath<'_> = &[(0, 21_999_999), (25_000_000, 26_000_000)];
    assert_orca_case(
        CoordinateScale::Normal,
        boundary(25_000_000, 30_000_000, &[]),
        &[normal_first, normal_equal],
        1,
        false,
        &[
            &[
                (0, 3_628_318),
                (0, 2_000_000),
                (25_000_000, 3_000_000),
                (25_000_000, 4_628_318),
            ],
            &[
                (0, 20_371_681),
                (0, 22_000_000),
                (25_000_000, 26_000_000),
                (25_000_000, 24_371_681),
            ],
        ],
    );
    assert_orca_case(
        CoordinateScale::Normal,
        boundary(25_000_000, 30_000_000, &[]),
        &[normal_first, normal_below],
        1,
        false,
        &[&[
            (25_000_000, 27_628_318),
            (25_000_000, 26_000_000),
            (0, 21_999_999),
            (0, 2_000_000),
            (25_000_000, 3_000_000),
            (25_000_000, 1_371_681),
        ]],
    );

    let large_first: RawPath<'_> = &[(0, 199_999), (2_500_000, 300_000)];
    let large_equal: RawPath<'_> = &[(0, 2_199_998), (2_500_000, 2_600_000)];
    let large_below: RawPath<'_> = &[(0, 2_199_997), (2_500_000, 2_600_000)];
    let large_equal_output: RawPath<'_> = &[
        (2_500_000, 2_762_831),
        (2_500_000, 2_600_000),
        (0, 2_199_998),
        (0, 199_999),
        (2_500_000, 300_000),
        (2_500_000, 137_168),
    ];
    let large_below_output: RawPath<'_> = &[
        (2_500_000, 2_762_831),
        (2_500_000, 2_600_000),
        (0, 2_199_997),
        (0, 199_999),
        (2_500_000, 300_000),
        (2_500_000, 137_168),
    ];
    for (second, expected) in [
        (large_equal, large_equal_output),
        (large_below, large_below_output),
    ] {
        assert_orca_case(
            CoordinateScale::LargeBed,
            boundary(2_500_000, 2_999_999, &[]),
            &[large_first, second],
            1,
            false,
            &[expected],
        );
    }
}

#[test]
fn task22o44_multiline_one_two_and_dont_sort_match_orca() {
    let normal_input: &[RawPath<'_>] = &[
        &[(0, 500_000), (25_000_000, 600_000)],
        &[(0, 1_000_000), (25_000_000, 1_300_000)],
    ];
    let normal_one: RawPath<'_> = &[
        (25_000_000, 2_928_318),
        (25_000_000, 1_300_000),
        (0, 1_000_000),
        (0, 500_000),
        (25_000_000, 600_000),
        (25_000_000, 0),
        (23_971_681, 0),
    ];
    let normal_two: RawPath<'_> = &[
        (23_971_681, 0),
        (25_000_000, 0),
        (25_000_000, 600_000),
        (0, 500_000),
        (0, 1_000_000),
        (25_000_000, 1_300_000),
        (25_000_000, 2_928_318),
    ];
    for (multiline, dont_sort, expected) in [
        (1, false, normal_one),
        (2, false, normal_two),
        (1, true, normal_two),
    ] {
        assert_orca_case(
            CoordinateScale::Normal,
            boundary(25_000_000, 4_000_000, &[]),
            normal_input,
            multiline,
            dont_sort,
            &[expected],
        );
    }

    let large_input: &[RawPath<'_>] = &[
        &[(0, 49_999), (2_500_000, 59_999)],
        &[(0, 99_999), (2_500_000, 130_000)],
    ];
    let large_one: RawPath<'_> = &[
        (2_500_000, 292_831),
        (2_500_000, 130_000),
        (0, 99_999),
        (0, 49_999),
        (2_500_000, 59_999),
        (2_500_000, 0),
        (2_397_167, 0),
    ];
    let large_two: RawPath<'_> = &[
        (2_397_167, 0),
        (2_500_000, 0),
        (2_500_000, 59_999),
        (0, 49_999),
        (0, 99_999),
        (2_500_000, 130_000),
        (2_500_000, 292_831),
    ];
    for (multiline, dont_sort, expected) in [
        (1, false, large_one),
        (2, false, large_two),
        (1, true, large_two),
    ] {
        assert_orca_case(
            CoordinateScale::LargeBed,
            boundary(2_500_000, 399_999, &[]),
            large_input,
            multiline,
            dont_sort,
            &[expected],
        );
    }
}

#[test]
fn task22o44_hole_fragments_match_orca_at_both_scales() {
    let normal_hole: RawPath<'_> = &[
        (5_000_000, 3_000_000),
        (5_000_000, 11_000_000),
        (13_000_000, 11_000_000),
        (13_000_000, 3_000_000),
    ];
    assert_orca_case(
        CoordinateScale::Normal,
        boundary(20_000_000, 15_000_000, &[normal_hole]),
        &[
            &[(0, 2_000_000), (5_000_000, 4_000_000)],
            &[(13_000_000, 5_000_000), (20_000_000, 4_000_000)],
            &[(0, 9_000_000), (5_000_000, 8_000_000)],
            &[(13_000_000, 10_000_000), (20_000_000, 12_000_000)],
        ],
        1,
        false,
        &[
            &[
                (0, 371_681),
                (0, 2_000_000),
                (5_000_000, 4_000_000),
                (5_000_000, 8_000_000),
                (0, 9_000_000),
                (0, 10_628_318),
            ],
            &[
                (20_000_000, 13_628_318),
                (20_000_000, 12_000_000),
                (13_000_000, 10_000_000),
                (13_000_000, 5_000_000),
                (20_000_000, 4_000_000),
                (20_000_000, 2_371_681),
            ],
        ],
    );

    let large_hole: RawPath<'_> = &[
        (499_999, 300_000),
        (499_999, 1_100_000),
        (1_300_000, 1_100_000),
        (1_300_000, 300_000),
    ];
    assert_orca_case(
        CoordinateScale::LargeBed,
        boundary(1_999_999, 1_499_999, &[large_hole]),
        &[
            &[(0, 199_999), (499_999, 399_999)],
            &[(1_300_000, 499_999), (1_999_999, 399_999)],
            &[(0, 899_999), (499_999, 799_999)],
            &[(1_300_000, 999_999), (1_999_999, 1_200_000)],
        ],
        1,
        false,
        &[
            &[
                (0, 37_167),
                (0, 199_999),
                (499_999, 399_999),
                (499_999, 799_999),
                (0, 899_999),
                (0, 1_062_830),
            ],
            &[
                (1_999_999, 1_362_831),
                (1_999_999, 1_200_000),
                (1_300_000, 999_999),
                (1_300_000, 499_999),
                (1_999_999, 399_999),
                (1_999_998, 237_167),
            ],
        ],
    );
}

#[test]
fn task22o44_near_boundary_outputs_keep_original_endpoints_at_both_scales() {
    assert_orca_case(
        CoordinateScale::Normal,
        boundary(12_000_000, 15_000_000, &[]),
        &[
            &[(2, 2_000_000), (11_999_998, 3_000_000)],
            &[(2, 6_000_000), (11_999_998, 10_000_000)],
        ],
        1,
        false,
        &[&[
            (11_999_998, 11_628_318),
            (11_999_998, 10_000_000),
            (2, 6_000_000),
            (2, 2_000_000),
            (11_999_998, 3_000_000),
            (11_999_999, 1_371_681),
        ]],
    );
    assert_orca_case(
        CoordinateScale::LargeBed,
        boundary(1_200_000, 1_499_999, &[]),
        &[
            &[(2, 199_999), (1_199_998, 300_000)],
            &[(2, 600_000), (1_199_998, 999_999)],
        ],
        1,
        false,
        &[&[
            (1_199_998, 1_162_830),
            (1_199_998, 999_999),
            (2, 600_000),
            (2, 199_999),
            (1_199_998, 300_000),
            (1_199_999, 137_168),
        ]],
    );
}

#[test]
fn task22o44_unconnected_interior_stays_separate_at_both_scales() {
    assert_orca_case(
        CoordinateScale::Normal,
        boundary(12_000_000, 8_000_000, &[]),
        &[
            &[(1_000_000, 1_000_000), (11_000_000, 1_000_000)],
            &[(0, 3_000_000), (12_000_000, 3_000_000)],
        ],
        1,
        false,
        &[
            &[(1_000_000, 1_000_000), (11_000_000, 1_000_000)],
            &[
                (0, 4_628_318),
                (0, 3_000_000),
                (12_000_000, 3_000_000),
                (12_000_000, 4_628_318),
            ],
        ],
    );
    assert_orca_case(
        CoordinateScale::LargeBed,
        boundary(1_200_000, 799_999, &[]),
        &[
            &[(99_999, 99_999), (1_100_000, 99_999)],
            &[(0, 300_000), (1_200_000, 300_000)],
        ],
        1,
        false,
        &[
            &[(99_999, 99_999), (1_100_000, 99_999)],
            &[
                (0, 462_831),
                (0, 300_000),
                (1_200_000, 300_000),
                (1_200_000, 462_831),
            ],
        ],
    );
}
