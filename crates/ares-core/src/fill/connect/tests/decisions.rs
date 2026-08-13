use super::super::contour::{complete_arc_attempts, limited_hook_is_clockwise};
use super::{FillConnectionParams, KSR_ANCHOR, KSR_SPACING, connect_infill, point};
use crate::geometry::{CoordinateScale, ExPolygon, Polygon, Polyline};

type RawPath<'a> = &'a [(i64, i64)];

fn polylines(raw: &[RawPath<'_>]) -> Vec<Polyline> {
    raw.iter()
        .map(|path| Polyline::new(path.iter().map(|&(x, y)| point(x, y)).collect()))
        .collect()
}

fn assert_normal_orca_case(
    boundary_size: (i64, i64),
    input: &[RawPath<'_>],
    multiline: i32,
    dont_sort: bool,
    expected: &[RawPath<'_>],
) {
    let (width, height) = boundary_size;
    let boundary = ExPolygon::new(
        Polygon::new(vec![
            point(0, 0),
            point(width, 0),
            point(width, height),
            point(0, height),
        ]),
        Vec::new(),
    );
    let before = boundary.clone();
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
        CoordinateScale::Normal,
    )
    .unwrap();

    assert_eq!(actual, polylines(expected));
    assert_eq!(boundary, before);
}

#[test]
fn task22o44_remaining_pass_accepts_anchor_max_equality() {
    assert_normal_orca_case(
        (25_000_000, 30_000_000),
        &[
            &[(0, 2_000_000), (25_000_000, 3_000_000)],
            &[(0, 22_000_000), (25_000_000, 26_000_000)],
        ],
        1,
        true,
        &[&[
            (25_000_000, 1_371_681),
            (25_000_000, 3_000_000),
            (0, 2_000_000),
            (0, 22_000_000),
            (25_000_000, 26_000_000),
            (25_000_000, 27_628_318),
        ]],
    );
}

#[test]
fn task22o44_equal_complete_arc_attempts_both_prefer_previous() {
    assert_eq!(
        complete_arc_attempts(20_000_000.0, 20_000_000.0),
        [(20_000_000.0, true), (20_000_000.0, true)]
    );
}

#[test]
fn task22o44_equal_limited_hook_sides_prefer_next() {
    assert!(!limited_hook_is_clockwise(1_000_000.0, 1_000_000.0));
}

#[test]
fn task22o44_equal_complete_and_limited_ties_match_orca() {
    assert_normal_orca_case(
        (30_000_000, 20_000_000),
        &[
            &[
                (30_000_000, 10_000_000),
                (15_000_000, 10_000_000),
                (30_000_000, 5_000_000),
            ],
            &[(30_000_000, 15_000_000), (15_000_000, 15_000_000)],
        ],
        1,
        true,
        &[
            &[
                (30_000_000, 11_628_318),
                (30_000_000, 10_000_000),
                (15_000_000, 10_000_000),
                (30_000_000, 5_000_000),
                (30_000_000, 3_371_681),
            ],
            &[
                (30_000_000, 16_628_318),
                (30_000_000, 15_000_000),
                (15_000_000, 15_000_000),
            ],
        ],
    );
}

#[test]
fn task22o44_dont_sort_ignores_multiline_two_in_remaining_pass() {
    assert_normal_orca_case(
        (25_000_000, 4_000_000),
        &[
            &[(0, 500_000), (25_000_000, 600_000)],
            &[(0, 1_000_000), (25_000_000, 1_300_000)],
        ],
        2,
        true,
        &[&[
            (23_971_681, 0),
            (25_000_000, 0),
            (25_000_000, 600_000),
            (0, 500_000),
            (0, 1_000_000),
            (25_000_000, 1_300_000),
            (25_000_000, 2_928_318),
        ]],
    );
}

#[test]
fn task22o44_multivertex_interior_clipping_trims_both_hooks() {
    assert_normal_orca_case(
        (12_000_000, 8_000_000),
        &[&[
            (0, 2_000_000),
            (200_000, 2_000_000),
            (1_000_000, 7_700_000),
            (8_000_000, 7_700_000),
            (11_800_000, 3_000_000),
            (12_000_000, 3_000_000),
        ]],
        1,
        false,
        &[&[
            (0, 371_681),
            (0, 2_000_000),
            (200_000, 2_000_000),
            (1_000_000, 7_700_000),
            (8_000_000, 7_700_000),
            (11_800_000, 3_000_000),
            (12_000_000, 3_000_000),
            (12_000_000, 1_371_681),
        ]],
    );
}
