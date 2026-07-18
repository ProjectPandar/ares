use super::helpers::{point, polygon, square, traced_fixed_sort};
use crate::geometry::clipper::ordering::SortTrace;
use crate::geometry::clipper::{
    ClipperError, ClipperOptions, ClosedClipper, PathRole, fixed_round, point_in_polygon,
    slopes_equal,
};

const LO_RANGE: i64 = 0x3fff_ffff;
const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

#[test]
fn task22f_closed_input_switches_to_full_range_only_beyond_lo_range() {
    let mut low = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(
        low.add_closed_path(
            &polygon(&[(LO_RANGE, 0), (0, 1), (0, 2)]),
            PathRole::Subject,
        ),
        Ok(true)
    );
    assert!(!low.input_snapshot().use_full_range);

    let mut full = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(
        full.add_closed_path(
            &polygon(&[(LO_RANGE + 1, 0), (0, 1), (0, 2)]),
            PathRole::Subject,
        ),
        Ok(true)
    );
    assert!(full.input_snapshot().use_full_range);

    let rejected_flat = polygon(&[(LO_RANGE + 1, 0), (LO_RANGE + 2, 0), (LO_RANGE + 3, 0)]);
    let mut monotonic = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(
        monotonic.add_closed_paths(&[rejected_flat, square()], PathRole::Subject),
        Ok(true)
    );
    let snapshot = monotonic.input_snapshot();
    assert!(snapshot.use_full_range);
    assert_eq!(snapshot.edges.len(), 4);
    assert_eq!(snapshot.edges[0].current, Some(point(0, 0)));
}

#[test]
fn task22f_closed_input_accepts_inclusive_positive_and_negative_hi_range() {
    for coordinate in [HI_RANGE, -HI_RANGE] {
        let mut clipper = ClosedClipper::new(ClipperOptions::default());
        assert_eq!(
            clipper.add_closed_path(
                &polygon(&[(coordinate, 0), (0, 1), (0, 2)]),
                PathRole::Subject,
            ),
            Ok(true)
        );
    }
}

#[test]
fn task22f_closed_input_rejects_one_unit_outside_each_hi_range_bound() {
    for coordinate in [HI_RANGE + 1, -(HI_RANGE + 1)] {
        let mut clipper = ClosedClipper::new(ClipperOptions::default());
        assert_eq!(
            clipper.add_closed_path(
                &polygon(&[(coordinate, 0), (0, 1), (0, 2)]),
                PathRole::Subject,
            ),
            Err(ClipperError::CoordinateOutOfRange)
        );
    }
}

#[test]
fn task22f_closed_input_checks_candidate_count_before_coordinate_range() {
    let mut ignored = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(
        ignored.add_closed_path(&polygon(&[(i64::MAX, 0), (0, 0)]), PathRole::Subject,),
        Ok(false)
    );
    assert!(ignored.input_snapshot().edges.is_empty());

    let mut checked = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(
        checked.add_closed_path(
            &polygon(&[(i64::MAX, 0), (0, 0), (1, 0)]),
            PathRole::Subject,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22f_clipper_round_matches_fixed_fractional_contract() {
    assert_eq!(fixed_round(0.499_999_999_999_999_94), 0);
    assert_eq!(fixed_round(0.5), 1);
    assert_eq!(fixed_round(-0.5), 0);
    assert_eq!(fixed_round(-1.5), -1);
    assert_eq!(fixed_round(1.5), 2);
}

#[test]
fn task22f_full_range_slope_equality_uses_exact_i128_products() {
    let s = 1_i64 << 32;

    assert!(!slopes_equal(s, s - 1, s + 1, s, true));
    assert!(slopes_equal(s, s - 1, 2 * s, 2 * s - 2, true));
    assert!(!slopes_equal(0, s, s, 0, true));
}

#[test]
fn task22f_point_in_polygon_preserves_fixed_f64_boundary_behavior() {
    let square = polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)]);
    assert_eq!(point_in_polygon(point(5, 5), square.points()), 1);
    assert_eq!(point_in_polygon(point(10, 5), square.points()), -1);
    assert_eq!(point_in_polygon(point(11, 5), square.points()), 0);

    let n = 1_i64 << 60;
    let floating_boundary = polygon(&[(-n, -n), (n + 1, n), (n + 1, -n)]);
    assert_eq!(
        point_in_polygon(point(0, 0), floating_boundary.points()),
        -1
    );
}

#[test]
fn task22f_fixed_sort_insertion_freezes_32_item_identity_permutation_and_trace() {
    const KEYS: [i64; 32] = [
        3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6,
        1, 7, 2,
    ];
    const EXPECTED_IDENTITIES: [usize; 32] = [
        5, 16, 27, 7, 18, 29, 9, 20, 31, 0, 11, 22, 2, 13, 24, 4, 15, 26, 6, 17, 28, 8, 19, 30, 10,
        21, 1, 12, 23, 3, 14, 25,
    ];

    let (identities, trace) = traced_fixed_sort(&KEYS, false);

    assert_eq!(identities, EXPECTED_IDENTITIES);
    assert_eq!(
        trace,
        SortTrace {
            insertion_calls: 1,
            median3_calls: 0,
            ninther_calls: 0,
            partition_calls: 0,
            heap_fallback_calls: 0,
            partitions: Vec::new(),
            heap_entry_identities: Vec::new(),
        }
    );
}

#[test]
fn task22f_fixed_sort_ninther_freezes_42_item_identity_permutation_and_trace() {
    const KEYS: [i64; 42] = [
        3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6,
        1, 7, 2, 8, 3, 9, 4, 10, 5, 0, 6, 1, 7,
    ];
    const EXPECTED_IDENTITIES: [usize; 42] = [
        5, 38, 27, 16, 40, 7, 29, 18, 9, 31, 20, 22, 0, 33, 11, 2, 35, 13, 24, 4, 15, 26, 37, 17,
        28, 6, 39, 30, 8, 41, 19, 21, 10, 32, 23, 12, 34, 1, 25, 14, 3, 36,
    ];

    let (identities, trace) = traced_fixed_sort(&KEYS, false);

    assert_eq!(identities, EXPECTED_IDENTITIES);
    assert_eq!(
        trace,
        SortTrace {
            insertion_calls: 2,
            median3_calls: 4,
            ninther_calls: 1,
            partition_calls: 1,
            heap_fallback_calls: 0,
            partitions: vec![[42, 42, 19, 4, 19]],
            heap_entry_identities: Vec::new(),
        }
    );
}

#[test]
fn task22f_fixed_sort_heap_fallback_freezes_87_item_identity_permutation_and_trace() {
    const KEYS: [i64; 87] = [
        0, 6, 10, 16, 20, 26, 32, 36, 42, 48, 1, 50, 7, 11, 52, 17, 21, 27, 54, 33, 37, 43, 55, 56,
        57, 58, 59, 60, 61, 62, 63, 64, 65, 2, 8, 12, 18, 22, 34, 28, 38, 66, 44, 3, 49, 9, 46, 13,
        67, 51, 19, 68, 23, 69, 70, 29, 71, 72, 35, 73, 39, 74, 75, 45, 76, 77, 4, 78, 14, 53, 24,
        79, 30, 80, 40, 81, 5, 15, 25, 31, 41, 47, 82, 83, 84, 85, 85,
    ];
    const EXPECTED_IDENTITIES: [usize; 87] = [
        0, 10, 33, 43, 66, 76, 1, 12, 34, 45, 2, 13, 35, 47, 68, 77, 3, 15, 36, 50, 4, 16, 37, 52,
        70, 78, 5, 17, 39, 55, 72, 79, 6, 19, 38, 58, 7, 20, 40, 60, 74, 80, 8, 21, 42, 63, 46, 81,
        9, 44, 11, 49, 14, 69, 18, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 41, 48, 51, 53, 54,
        56, 57, 59, 61, 62, 64, 65, 67, 71, 73, 75, 82, 83, 84, 85, 86,
    ];
    const HEAP_ENTRY_IDENTITIES: [usize; 33] = [
        18, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 41, 48, 56, 67, 54, 71, 57, 73, 62, 75, 65,
        51, 53, 59, 61, 64, 82, 83, 84, 85, 86,
    ];

    let (identities, trace) = traced_fixed_sort(&KEYS, false);

    assert_eq!(identities, EXPECTED_IDENTITIES);
    assert_eq!(
        trace,
        SortTrace {
            insertion_calls: 12,
            median3_calls: 39,
            ninther_calls: 9,
            partition_calls: 12,
            heap_fallback_calls: 1,
            partitions: vec![
                [87, 87, 3, 1, 83],
                [83, 64, 3, 1, 79],
                [79, 48, 5, 1, 73],
                [73, 36, 3, 1, 69],
                [69, 27, 5, 1, 63],
                [63, 19, 5, 1, 57],
                [57, 13, 3, 1, 53],
                [53, 9, 5, 1, 47],
                [47, 6, 5, 1, 41],
                [41, 4, 3, 1, 37],
                [37, 3, 1, 1, 35],
                [35, 1, 1, 1, 33],
            ],
            heap_entry_identities: vec![HEAP_ENTRY_IDENTITIES.to_vec()],
        }
    );
}
