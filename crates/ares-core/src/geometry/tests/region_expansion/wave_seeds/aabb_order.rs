use super::*;
use crate::geometry::region_expansion::{
    bbox_contains_for_test, centroid_for_test, longest_axis_for_test, partition_for_test,
    sample_for_test,
};

#[test]
fn bbox_inflation_uses_both_fixed_scales_inclusively() {
    let boundary = square(0, 100);
    assert!(bbox_contains_for_test(
        &boundary,
        Point::new(-100, 50),
        CoordinateScale::Normal
    ));
    assert!(!bbox_contains_for_test(
        &boundary,
        Point::new(-101, 50),
        CoordinateScale::Normal
    ));
    assert!(bbox_contains_for_test(
        &boundary,
        Point::new(-10, 50),
        CoordinateScale::LargeBed
    ));
    assert!(!bbox_contains_for_test(
        &boundary,
        Point::new(-11, 50),
        CoordinateScale::LargeBed
    ));
}

#[test]
fn hole_interior_is_rejected_but_hole_boundary_is_contained() {
    let hole = polygon(&[(40, 40), (60, 40), (60, 60), (40, 60)]);
    let boundary = expolygon(&[(0, 0), (100, 0), (100, 100), (0, 100)], vec![hole]);
    assert_eq!(
        sample_for_test(
            std::slice::from_ref(&boundary),
            Point::new(50, 50),
            CoordinateScale::Normal
        ),
        None
    );
    assert_eq!(
        sample_for_test(&[boundary], Point::new(40, 50), CoordinateScale::Normal),
        Some(0)
    );
}

#[test]
fn overlapping_candidates_stop_at_first_containing_leaf() {
    let with_hole = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![polygon(&[(40, 40), (60, 40), (60, 60), (40, 60)])],
    );
    assert_eq!(
        sample_for_test(
            &[with_hole, square(0, 100)],
            Point::new(50, 50),
            CoordinateScale::Normal,
        ),
        Some(1)
    );
}

#[test]
fn boundary_bbox_uses_contour_only_even_when_hole_extends_outside() {
    let boundary = expolygon(
        &[(0, 0), (100, 0), (100, 100), (0, 100)],
        vec![polygon(&[(500, 500), (600, 500), (600, 600), (500, 600)])],
    );
    assert!(!bbox_contains_for_test(
        &boundary,
        Point::new(500, 500),
        CoordinateScale::Normal
    ));
}

#[test]
fn centroid_uses_literal_min_plus_half_max_for_negative_ranges() {
    assert_eq!(
        centroid_for_test(Point::new(-9, 0), Point::new(-3, 0), 0),
        -10
    );
}

#[test]
fn equal_axis_lengths_choose_x_and_only_strictly_taller_bounds_choose_y() {
    assert_eq!(
        longest_axis_for_test(Point::new(0, 0), Point::new(20, 20)),
        0
    );
    assert_eq!(
        longest_axis_for_test(Point::new(0, 0), Point::new(20, 21)),
        1
    );
}

#[test]
fn quickselect_partition_has_literal_multi_round_identity_order() {
    let bounds = [8, 1, 6, 3, 7, 2, 5, 4, 0].map(|x| (Point::new(x, 0), Point::new(x, 0)));
    assert_eq!(
        partition_for_test(&bounds, 0, 4),
        vec![8, 1, 5, 3, 7, 6, 2, 4, 0]
    );
}

#[test]
fn x_ties_keep_quickselect_candidate_order_for_first_hit() {
    let first = square(0, 20);
    let second = square(0, 20);
    assert_eq!(
        sample_for_test(
            &[first, second],
            Point::new(10, 10),
            CoordinateScale::Normal
        ),
        Some(0)
    );
}
