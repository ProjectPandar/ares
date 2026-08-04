use super::helpers::{polygon, polygons};
use crate::geometry::clipper::{
    safety_offset_clip_paths_for_test, safety_offset_configuration_for_test,
};
use crate::geometry::{
    ClipperError, difference_polygons_paths, intersection_polygons_paths,
    intersection_polygons_paths_with_safety_offset, union_polygons_paths,
};

#[test]
fn task22o20_paths_adapters_freeze_empty_and_nonzero_union_order() {
    assert!(union_polygons_paths(&[]).unwrap().is_empty());
    let input = polygons(&[
        &[(0, 0), (40, 0), (40, 40), (0, 40)],
        &[(10, 10), (30, 10), (30, 30), (10, 30)],
        &[(15, 15), (15, 25), (25, 25), (25, 15)],
    ]);
    assert_eq!(
        union_polygons_paths(&input).unwrap(),
        vec![polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)])]
    );
}

#[test]
fn task22o20_paths_union_preserves_holed_repeated_disjoint_order() {
    let outer = polygon(&[(0, 0), (40, 0), (40, 40), (0, 40)]);
    let hole = polygon(&[(10, 10), (10, 30), (30, 30), (30, 10)]);
    let disjoint = polygon(&[(100, 0), (120, 0), (120, 20), (100, 20)]);
    assert_eq!(
        union_polygons_paths(&[outer, hole, disjoint.clone(), disjoint]).unwrap(),
        polygons(&[
            &[(40, 40), (0, 40), (0, 0), (40, 0)],
            &[(10, 10), (10, 30), (30, 30), (30, 10)],
            &[(120, 20), (100, 20), (100, 0), (120, 0)],
        ])
    );
}

#[test]
fn task22o20_paths_intersection_preserves_flat_paths_output() {
    let subject = vec![polygon(&[
        (0, 0),
        (30, 0),
        (30, 10),
        (10, 10),
        (10, 30),
        (0, 30),
    ])];
    let clip = vec![polygon(&[(5, -5), (25, -5), (25, 25), (5, 25)])];
    assert_eq!(
        intersection_polygons_paths(&subject, &clip).unwrap(),
        vec![polygon(&[
            (25, 10),
            (10, 10),
            (10, 25),
            (5, 25),
            (5, 0),
            (25, 0),
        ])]
    );
    assert!(intersection_polygons_paths(&[], &clip).unwrap().is_empty());
    assert!(
        intersection_polygons_paths(&subject, &[])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn task22o21_paths_difference_freezes_empty_and_flat_nonzero_order() {
    let outer = polygon(&[(0, 0), (40, 0), (40, 40), (0, 40)]);
    let inner = polygon(&[(10, 10), (30, 10), (30, 30), (10, 30)]);
    assert!(
        difference_polygons_paths(&[], std::slice::from_ref(&inner))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        difference_polygons_paths(std::slice::from_ref(&outer), &[]).unwrap(),
        vec![polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)])]
    );
    assert_eq!(
        difference_polygons_paths(&[outer], &[inner]).unwrap(),
        polygons(&[
            &[(40, 40), (0, 40), (0, 0), (40, 0)],
            &[(10, 10), (10, 30), (30, 30), (30, 10)],
        ])
    );
}

#[test]
fn task22o21_paths_difference_preserves_repeated_disjoint_nonzero_paths() {
    let repeated = polygon(&[(0, 0), (30, 0), (30, 30), (0, 30)]);
    let disjoint = polygon(&[(50, 0), (70, 0), (70, 20), (50, 20)]);
    let hole = polygon(&[(10, 10), (20, 10), (20, 20), (10, 20)]);
    assert_eq!(
        difference_polygons_paths(&[repeated.clone(), repeated, disjoint], &[hole]).unwrap(),
        polygons(&[
            &[(30, 30), (0, 30), (0, 0), (30, 0)],
            &[(10, 10), (10, 20), (20, 20), (20, 10)],
            &[(70, 20), (50, 20), (50, 0), (70, 0)],
        ])
    );
}

#[test]
fn task22o21_difference_contour_hole_and_near_touching_clip_have_no_safety() {
    let contour = polygon(&[(0, 0), (40, 0), (40, 40), (0, 40)]);
    let hole = polygon(&[(10, 10), (10, 30), (30, 30), (30, 10)]);
    let near_touching = polygon(&[(41, 0), (50, 0), (50, 40), (41, 40)]);
    assert_eq!(
        difference_polygons_paths(&[contour, hole], &[near_touching]).unwrap(),
        polygons(&[
            &[(40, 40), (0, 40), (0, 0), (40, 0)],
            &[(10, 10), (10, 30), (30, 30), (30, 10)],
        ])
    );
}

#[test]
fn task22o21_safety_intersection_expands_only_clip_by_shared_source_constants() {
    let subject = vec![polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)])];
    let clip = vec![polygon(&[(19, 0), (29, 0), (29, 10), (19, 10)])];
    assert!(
        intersection_polygons_paths_with_safety_offset(&[], &clip)
            .unwrap()
            .is_empty()
    );
    assert!(
        intersection_polygons_paths_with_safety_offset(&subject, &[])
            .unwrap()
            .is_empty()
    );
    assert!(
        intersection_polygons_paths(&subject, &clip)
            .unwrap()
            .is_empty()
    );
    let expected = vec![polygon(&[(10, 10), (9, 10), (9, 0), (10, 0)])];
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(&subject, &clip).unwrap(),
        expected
    );
    let repeated_edge = vec![polygon(&[(19, 0), (19, 0), (29, 0), (29, 10), (19, 10)])];
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(&subject, &repeated_edge).unwrap(),
        expected
    );
}

#[test]
fn task22o21_safety_intersection_freezes_contour_hole_repetition_and_disjoint_order() {
    let subject = vec![polygon(&[(-50, -50), (250, -50), (250, 150), (-50, 150)])];
    let contour = polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]);
    let hole = polygon(&[(80, 20), (20, 20), (20, 80), (80, 80)]);
    let disjoint = polygon(&[(150, 0), (190, 0), (190, 40), (150, 40)]);
    let clip = vec![contour, hole, disjoint.clone(), disjoint];
    assert_eq!(
        safety_offset_clip_paths_for_test(&clip).unwrap(),
        polygons(&[
            &[(110, 110), (-10, 110), (-10, -10), (110, -10)],
            &[(70, 30), (30, 30), (30, 70), (70, 70)],
            &[(200, 50), (140, 50), (140, -10), (200, -10)],
            &[(200, 50), (140, 50), (140, -10), (200, -10)],
        ])
    );
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(&subject, &clip).unwrap(),
        polygons(&[
            &[(110, 110), (-10, 110), (-10, -10), (110, -10)],
            &[(30, 30), (30, 70), (70, 70), (70, 30)],
            &[(200, 50), (140, 50), (140, -10), (200, -10)],
        ])
    );
}

#[test]
fn task22o21_safety_intersection_freezes_miter_three_and_shortest_edge_configuration() {
    let (miter_limit, shortest_edge) = safety_offset_configuration_for_test();
    assert_eq!(miter_limit, 6.0 / 2.0);
    assert_eq!(shortest_edge, 1.0 / 20.0);

    let subject = vec![polygon(&[(-50, -50), (150, -50), (150, 100), (-50, 100)])];
    let acute = polygon(&[(0, 0), (100, 0), (50, 50)]);
    assert_eq!(
        safety_offset_clip_paths_for_test(std::slice::from_ref(&acute)).unwrap(),
        vec![polygon(&[(50, 64), (-24, -10), (124, -10)])]
    );
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(&subject, &[acute]).unwrap(),
        vec![polygon(&[(50, 64), (-24, -10), (124, -10)])]
    );
}

#[test]
fn task22o21_safety_intersection_is_raw_path_by_path_not_preunioned() {
    let subject = vec![polygon(&[(-20, -20), (60, -20), (60, 60), (-20, 60)])];
    let contour = polygon(&[(0, 0), (40, 0), (40, 40), (0, 40)]);
    let mut reversed = contour.clone();
    reversed.reverse();
    let clip = vec![contour, reversed];
    assert_eq!(
        safety_offset_clip_paths_for_test(&clip).unwrap(),
        polygons(&[
            &[(50, 50), (-10, 50), (-10, -10), (50, -10)],
            &[(30, 10), (10, 10), (10, 30), (30, 30)],
        ])
    );
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(&subject, &clip).unwrap(),
        polygons(&[
            &[(50, 50), (-10, 50), (-10, -10), (50, -10)],
            &[(10, 10), (10, 30), (30, 30), (30, 10)],
        ])
    );
    let preunioned = union_polygons_paths(&clip).unwrap();
    assert!(preunioned.is_empty());
    assert!(
        intersection_polygons_paths_with_safety_offset(&subject, &preunioned)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn task22o21_paths_adapters_propagate_real_coordinate_failures() {
    let invalid = polygon(&[(i64::MAX, 0), (0, 1), (0, -1)]);
    let valid = polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)]);
    assert_eq!(
        difference_polygons_paths(std::slice::from_ref(&invalid), &[]),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(
            std::slice::from_ref(&invalid),
            std::slice::from_ref(&valid),
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        intersection_polygons_paths_with_safety_offset(&[valid], &[invalid]),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
