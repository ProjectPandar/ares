use crate::geometry::clipper::{ClipperOffset, JoinType};

use super::helpers::{coordinates, polygon, raw};

#[test]
fn task22g_convex_miter_emits_one_point_per_corner() {
    assert_eq!(
        raw(
            &[(0, 0), (100, 0), (100, 100), (0, 100)],
            JoinType::Miter,
            10.0,
        ),
        vec![vec![(-10, -10), (110, -10), (110, 110), (-10, 110)]]
    );
}

#[test]
fn task22g_sharp_miter_falls_back_to_square() {
    assert_eq!(
        raw(&[(0, 0), (100, 0), (0, 100)], JoinType::Miter, 10.0),
        vec![vec![(-10, -10), (107, -10), (112, 2), (2, 112), (-10, 107)]]
    );
}

#[test]
fn task22g_square_join_emits_ordered_pair_per_corner() {
    assert_eq!(
        raw(
            &[(0, 0), (100, 0), (100, 100), (0, 100)],
            JoinType::Square,
            10.0,
        ),
        vec![vec![
            (-10, -4),
            (-4, -10),
            (104, -10),
            (110, -4),
            (110, 104),
            (104, 110),
            (-4, 110),
            (-10, 104),
        ]]
    );
}

#[test]
fn task22g_round_join_uses_default_arc_tolerance_and_exact_endpoint() {
    assert_eq!(round_square(), vec![round_square_expected()]);
}

#[test]
fn task22g_concave_corner_emits_previous_vertex_current_triplet() {
    assert_eq!(
        raw(
            &[(0, 0), (100, 0), (100, 40), (40, 40), (40, 100), (0, 100)],
            JoinType::Miter,
            10.0,
        ),
        vec![vec![
            (-10, -10),
            (110, -10),
            (110, 50),
            (40, 50),
            (40, 40),
            (50, 40),
            (50, 110),
            (-10, 110),
        ]]
    );
}

#[test]
fn task22g_collinear_corner_emits_one_point() {
    assert_eq!(
        raw(
            &[(0, 0), (50, 0), (100, 0), (100, 100), (0, 100)],
            JoinType::Miter,
            10.0,
        ),
        vec![vec![
            (-10, -10),
            (50, -10),
            (110, -10),
            (110, 110),
            (-10, 110)
        ]]
    );
}

#[test]
fn task22g_near_collinear_early_return_does_not_advance_previous_index() {
    assert_eq!(
        raw(
            &[
                (0, 0),
                (10_000, 0),
                (20_000, 90),
                (20_000, 10_000),
                (0, 10_000)
            ],
            JoinType::Miter,
            100.0,
        ),
        vec![vec![
            (-100, -100),
            (10_000, -100),
            (20_100, -10),
            (20_100, 10_100),
            (-100, 10_100),
        ]]
    );
}

#[test]
fn task22g_round_raw_generation_is_repeatable() {
    let mut offset = ClipperOffset::default();
    offset.add_closed_path(
        &polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        JoinType::Round,
    );

    let first = offset
        .generate_raw(10.0)
        .iter()
        .map(coordinates)
        .collect::<Vec<_>>();
    let second = offset
        .generate_raw(10.0)
        .iter()
        .map(coordinates)
        .collect::<Vec<_>>();
    assert_eq!(first, vec![round_square_expected()]);
    assert_eq!(second, first);
}

fn round_square() -> Vec<Vec<(i64, i64)>> {
    raw(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        JoinType::Round,
        10.0,
    )
}

fn round_square_expected() -> Vec<(i64, i64)> {
    vec![
        (-10, 0),
        (-9, -4),
        (-6, -8),
        (-2, -10),
        (0, -10),
        (100, -10),
        (104, -9),
        (108, -6),
        (110, -2),
        (110, 0),
        (110, 100),
        (109, 104),
        (106, 108),
        (102, 110),
        (100, 110),
        (0, 110),
        (-4, 109),
        (-8, 106),
        (-10, 102),
        (-10, 100),
    ]
}
