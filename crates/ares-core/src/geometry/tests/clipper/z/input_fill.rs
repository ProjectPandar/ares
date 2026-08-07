use super::*;
use crate::geometry::Point;
use crate::geometry::clipper::{
    ClipOperation, ClipperError, FillRule, reverse_horizontal_for_test,
    z::{set_z_for_test, z_fill_for_test},
};

#[test]
fn ordinary_equality_ignores_z_but_full_equality_does_not() {
    let first = KernelPoint::new(3, 4, 1);
    let second = KernelPoint::new(3, 4, 2);
    assert_eq!(first, second);
    assert!(!first.full_eq(second));
    assert!(first.full_cmp(second).is_lt());
}

#[test]
fn collector_sorts_labels_and_uses_negative_one_based_index() {
    assert_eq!(z_fill_for_test([2, 1, 2, 1]), (-1, vec![(1, 2)]));
    assert_eq!(z_fill_for_test([7, 7, 7, 7]), (7, Vec::new()));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn collector_rejects_more_than_two_labels_in_debug() {
    z_fill_for_test([4, 1, 3, 2]);
}

#[cfg(not(debug_assertions))]
pub(crate) mod release {
    use super::*;

    #[test]
    fn collector_keeps_first_two_labels_in_release() {
        assert_eq!(z_fill_for_test([4, 1, 3, 2]), (-1, vec![(1, 2)]));
    }
}

#[test]
fn crossing_records_boundary_source_pair() {
    let (paths, pairs) = crossing_clipper().execute_z_paths(
        ClipOperation::Intersection,
        FillRule::NonZero,
        FillRule::NonZero,
    );
    assert_eq!(pairs, vec![(1, 2), (1, 2)]);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].first().unwrap().z, -2);
    assert_eq!(paths[0].last().unwrap().z, -1);
}

#[test]
fn set_z_uses_first_bottom_top_then_second_bottom_top() {
    let endpoints = [
        KernelPoint::new(5, 5, 11),
        KernelPoint::new(5, 5, 12),
        KernelPoint::new(5, 5, 21),
        KernelPoint::new(5, 5, 22),
    ];
    for (index, expected) in [11, 12, 21, 22].into_iter().enumerate() {
        let mut distinct = endpoints;
        for (other, point) in distinct.iter_mut().enumerate() {
            point.xy = Point::new(other as i64, other as i64);
        }
        distinct[index].xy = Point::new(9, 7);
        let (point, table) = set_z_for_test(KernelPoint::new(9, 7, 0), distinct);
        assert_eq!((point.z, table), (expected, Vec::new()));
    }

    let (point, table) = set_z_for_test(KernelPoint::new(5, 5, 0), endpoints);
    assert_eq!((point.z, table), (11, Vec::new()));
}

#[test]
fn nonzero_candidate_bypasses_endpoints_and_collector() {
    let endpoints = [
        KernelPoint::new(0, 0, 1),
        KernelPoint::new(1, 0, 2),
        KernelPoint::new(0, 1, 3),
        KernelPoint::new(1, 1, 4),
    ];
    let (point, table) = set_z_for_test(KernelPoint::new(4, 4, 99), endpoints);
    assert_eq!((point.z, table), (99, Vec::new()));
}

#[test]
fn horizontal_direction_orders_choose_ltr_horizontal_and_rtl_crossing_endpoint() {
    let horizontal = [KernelPoint::new(5, 5, 10), KernelPoint::new(9, 5, 11)];
    let crossing = [KernelPoint::new(5, 5, 20), KernelPoint::new(5, 9, 21)];
    let (ltr, ltr_table) = set_z_for_test(
        KernelPoint::new(5, 5, 0),
        [horizontal[0], horizontal[1], crossing[0], crossing[1]],
    );
    let (rtl, rtl_table) = set_z_for_test(
        KernelPoint::new(5, 5, 0),
        [crossing[0], crossing[1], horizontal[0], horizontal[1]],
    );
    assert_eq!((ltr.z, ltr_table), (10, Vec::new()));
    assert_eq!((rtl.z, rtl_table), (20, Vec::new()));
}

#[test]
fn horizontal_reversal_swaps_x_and_asymmetric_z_together() {
    let (bottom, top) =
        reverse_horizontal_for_test(KernelPoint::new(2, 7, 11), KernelPoint::new(19, 7, 29));
    assert_eq!((bottom.x(), bottom.y(), bottom.z), (19, 7, 29));
    assert_eq!((top.x(), top.y(), top.z), (2, 7, 11));
}

#[test]
fn strict_type3_previous_current_order_fills_one_shared_candidate() {
    let previous = [KernelPoint::new(5, 5, 31), KernelPoint::new(0, 0, 32)];
    let current = [KernelPoint::new(5, 5, 41), KernelPoint::new(9, 9, 42)];
    let (candidate, table) = set_z_for_test(
        KernelPoint::new(5, 5, 0),
        [previous[0], previous[1], current[0], current[1]],
    );
    assert_eq!((candidate.z, table), (31, Vec::new()));
}

#[test]
fn z_input_cleanup_closure_and_range_follow_xy_only_rules() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    let closed = path(&[(0, 0), (10, 0), (10, 0), (10, 10), (0, 0)], 7);
    assert_eq!(
        clipper.add_z_closed_path(&closed, PathRole::Subject),
        Ok(true)
    );
    let (paths, table) =
        clipper.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    assert!(table.is_empty());
    assert!(paths.iter().flatten().all(|point| point.z == 7));

    let outside = 0x4000_0000_0000_0000_i64;
    assert_eq!(
        Clipper::new(ClipperOptions::default()).add_z_closed_path(
            &path(&[(outside, 0), (outside, 1), (outside - 1, 1)], 3),
            PathRole::Subject,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
