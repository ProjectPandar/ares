use super::helpers::polygon;
use crate::geometry::{
    ClipperError, JoinType, offset_paths, opening_path_configurations_for_test, opening_paths,
    opening_paths_with_interstage,
};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

#[test]
fn task22o26_path_opening_is_ordered_asymmetric_shrink_then_expand() {
    let paths = vec![polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)])];
    let shrunk = offset_paths(&paths, -7.0, JoinType::Miter, 5.0).unwrap();
    let expected = offset_paths(&shrunk, 18.0, JoinType::Miter, 5.0).unwrap();
    let observed = std::cell::RefCell::new(Vec::new());

    let actual =
        opening_paths_with_interstage(&paths, [7.0, 18.0], JoinType::Miter, 5.0, |interstage| {
            observed.replace(interstage.to_vec());
            Ok(())
        })
        .unwrap();

    assert_eq!(observed.into_inner(), shrunk);
    assert_eq!(actual, expected);
    assert_eq!(
        opening_paths(&paths, 7.0, 18.0, JoinType::Miter, 5.0),
        Ok(expected)
    );
}

#[test]
fn task22o26_empty_opening_still_reaches_the_exact_interstage_boundary() {
    let visits = std::cell::Cell::new(0);
    assert_eq!(
        opening_paths_with_interstage(&[], [4.0, 14.0], JoinType::Miter, 5.0, |paths| {
            visits.set(visits.get() + 1);
            assert!(paths.is_empty());
            Ok(())
        }),
        Ok(Vec::new())
    );
    assert_eq!(visits.get(), 1);
    assert_eq!(
        opening_paths_with_interstage(&[], [4.0, 14.0], JoinType::Miter, 5.0, |_| Err(
            ClipperError::CoordinateOutOfRange
        )),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o26_opening_preserves_disjoint_and_repeated_flat_path_behavior() {
    let first = polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]);
    let second = polygon(&[(300, 0), (400, 0), (400, 100), (300, 100)]);
    for paths in [vec![first.clone(), second], vec![first.clone(), first]] {
        let shrunk = offset_paths(&paths, -5.0, JoinType::Miter, 5.0).unwrap();
        let expected = offset_paths(&shrunk, 15.0, JoinType::Miter, 5.0).unwrap();
        assert_eq!(
            opening_paths(&paths, 5.0, 15.0, JoinType::Miter, 5.0),
            Ok(expected)
        );
    }
}

#[test]
fn task22o26_opening_preserves_flat_contour_hole_orientation_order() {
    let contour = polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]);
    let hole = polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]);
    let input = vec![contour, hole];
    let shrunk = offset_paths(&input, -5.0, JoinType::Miter, 5.0).unwrap();
    let expected = offset_paths(&shrunk, 15.0, JoinType::Miter, 5.0).unwrap();

    assert_eq!(
        opening_paths(&input, 5.0, 15.0, JoinType::Miter, 5.0),
        Ok(expected)
    );
}

#[test]
fn task22o26_opening_freezes_both_miter5_shortest_edge_configurations() {
    assert_eq!(
        opening_path_configurations_for_test(20.0, 30.0, JoinType::Miter, 5.0),
        [(5.0, 0.1), (5.0, 0.15)]
    );
}

#[test]
fn task22o26_opening_reports_first_and_second_stage_coordinate_failures() {
    let invalid = vec![polygon(&[
        (HI_RANGE + 1, 0),
        (HI_RANGE + 100, 0),
        (HI_RANGE + 100, 100),
        (HI_RANGE + 1, 100),
    ])];
    let reached_interstage = std::cell::Cell::new(false);
    assert_eq!(
        opening_paths_with_interstage(&invalid, [1.0, 2.0], JoinType::Miter, 5.0, |_| {
            reached_interstage.set(true);
            Ok(())
        },),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert!(!reached_interstage.get());

    let near_limit = vec![polygon(&[
        (HI_RANGE - 1_000, 0),
        (HI_RANGE - 500, 0),
        (HI_RANGE - 500, 500),
        (HI_RANGE - 1_000, 500),
    ])];
    assert_eq!(
        opening_paths(&near_limit, 1.0, 2_000.0, JoinType::Miter, 5.0),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
