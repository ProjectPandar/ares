use super::{candidate, ids, rectangle};
use crate::project_slice::prepare_infill::bridge_over_infill::candidate_ordering::order_candidate_surfaces;

#[test]
fn task22o55_two_skips_distance_sort_and_three_uses_front_max_origin_on_tail_only() {
    let two = order_candidate_surfaces(vec![
        candidate(0, vec![rectangle(0, 0, 100, 100)]),
        candidate(1, vec![rectangle(10, 0, 20, 10)]),
    ]);
    assert_eq!(ids(&two), vec![0, 1]);

    let three = order_candidate_surfaces(vec![
        candidate(1, vec![rectangle(10, 0, 20, 10)]),
        candidate(0, vec![rectangle(0, 0, 100, 100)]),
        candidate(2, vec![rectangle(80, 0, 90, 10)]),
    ]);
    assert_eq!(ids(&three), vec![0, 2, 1]);
}

#[test]
fn task22o55_equal_distance_tail_keeps_first_sort_order() {
    let ordered = order_candidate_surfaces(vec![
        candidate(2, vec![rectangle(20, 10, 30, 20)]),
        candidate(0, vec![rectangle(0, 0, 100, 100)]),
        candidate(1, vec![rectangle(10, 20, 20, 30)]),
    ]);
    assert_eq!(ids(&ordered), vec![0, 1, 2]);
}

#[test]
fn task22o55_fixed_msvc_equal_key_ninther_permutation_is_preserved_by_stable_tail() {
    let ordered = order_candidate_surfaces(
        (0..42)
            .map(|id| candidate(id, vec![rectangle(3, 9, 103, 109)]))
            .collect(),
    );
    assert_eq!(ids(&ordered), (0..42).collect::<Vec<_>>());
}

#[test]
fn task22o55_fixed_msvc_mixed_ninther_permutation_feeds_stable_tail() {
    const KEYS: [i64; 42] = [
        3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6,
        1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7,
    ];
    let ordered = order_candidate_surfaces(
        KEYS.iter()
            .enumerate()
            .map(|(id, &key)| candidate(id, vec![rectangle(key, 0, 100, 100)]))
            .collect(),
    );
    assert_eq!(
        ids(&ordered),
        vec![
            5, 25, 14, 3, 36, 23, 12, 34, 1, 21, 10, 32, 30, 8, 41, 19, 17, 28, 6, 39, 4, 15, 26,
            37, 2, 35, 13, 24, 22, 0, 33, 11, 9, 31, 20, 40, 7, 29, 18, 38, 27, 16,
        ]
    );
}

#[test]
fn task22o55_squared_distance_differs_from_sum_square() {
    let ordered = order_candidate_surfaces(vec![
        candidate(0, vec![rectangle(0, 0, 100, 100)]),
        candidate(1, vec![rectangle(90, 90, 91, 91)]),
        candidate(2, vec![rectangle(100, 85, 101, 86)]),
    ]);
    assert_eq!(ids(&ordered), vec![0, 1, 2]);
}

#[test]
fn task22o55_separate_products_differ_from_mul_add_rounding() {
    let origin = 4_000_000_000_000_000_000_i64;
    let ordered = order_candidate_surfaces(vec![
        candidate(0, vec![rectangle(0, 0, origin, origin)]),
        candidate(
            1,
            vec![rectangle(
                997_300_736,
                993_560_576,
                997_300_737,
                993_560_577,
            )],
        ),
        candidate(
            2,
            vec![rectangle(
                997_936_128,
                992_925_696,
                997_936_129,
                992_925_697,
            )],
        ),
    ]);
    assert_eq!(ids(&ordered), vec![0, 1, 2]);
}

#[test]
fn task22o55_high_coordinates_cast_before_subtraction_and_square_components() {
    let high = 4_611_686_018_427_387_000_i64;
    let ordered = order_candidate_surfaces(vec![
        candidate(0, vec![rectangle(high - 10_000, high - 10_000, high, high)]),
        candidate(
            2,
            vec![rectangle(high - 1_025, high - 3, high - 1_020, high + 2)],
        ),
        candidate(
            1,
            vec![rectangle(high - 1_024, high - 20, high - 1_019, high - 15)],
        ),
    ]);
    assert_eq!(ids(&ordered), vec![0, 2, 1]);
}
