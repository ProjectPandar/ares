use super::helpers::{coordinates, polygon};
use crate::geometry::clipper::{
    FillRule, JoinType, offset_expolygon, offset_expolygon_refs_paths, offset_expolygons,
    offset_expolygons_paths, offset_expolygons_raw, offset2_ex, offset2_ex_with_interstage,
    opening_ex, union_ex,
};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;
use crate::geometry::{ExPolygon, Polygon};

fn fixed_input() -> ExPolygon {
    ExPolygon::new(
        polygon(&[(200, 100), (200, 200), (100, 200), (100, 100)]),
        vec![polygon(&[(160, 140), (140, 140), (140, 160), (160, 160)])],
    )
}

fn holeless(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(polygon(points), Vec::new())
}

fn path_coordinates(paths: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    paths.iter().map(coordinates).collect()
}

#[test]
fn task22g_fixed_expolygon_offset_and_offset2_match_orca_ordered_vectors() {
    assert_eq!(
        offset_expolygon(&fixed_input(), 5.0, JoinType::Miter, 3.0),
        Ok(vec![ExPolygon::new(
            polygon(&[(205, 205), (95, 205), (95, 95), (205, 95)]),
            vec![polygon(&[(145, 145), (145, 155), (155, 155), (155, 145)])],
        )])
    );
    assert_eq!(
        offset2_ex(&[fixed_input()], 5.0, -2.0, JoinType::Miter, 3.0),
        Ok(vec![ExPolygon::new(
            polygon(&[(203, 203), (97, 203), (97, 97), (203, 97)]),
            vec![polygon(&[(143, 143), (143, 157), (157, 157), (157, 143)])],
        )])
    );
}

#[test]
fn task22o22_offset2_interstage_observer_preserves_output_and_can_stop_before_second_stage() {
    let observed = std::cell::Cell::new(0);
    let expected = offset2_ex(&[fixed_input()], -5.0, 8.0, JoinType::Square, 3.0);
    assert_eq!(
        offset2_ex_with_interstage(&[fixed_input()], (-5.0, 8.0, JoinType::Square, 3.0), || {
            observed.set(observed.get() + 1);
            Ok(())
        },),
        expected
    );
    assert_eq!(observed.get(), 1);
    assert_eq!(
        offset2_ex_with_interstage(
            &[fixed_input()],
            (-5.0, 8.0, JoinType::Square, 3.0),
            || Err(crate::geometry::ClipperError::CoordinateOutOfRange),
        ),
        Err(crate::geometry::ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22o22_each_morphology_primitive_reports_actual_coordinate_failure() {
    let invalid = holeless(&[
        (HI_RANGE + 1, 0),
        (HI_RANGE + 2, 0),
        (HI_RANGE + 2, 10),
        (HI_RANGE + 1, 10),
    ]);
    assert_eq!(
        union_ex(&[invalid.contour().clone()], FillRule::NonZero),
        Err(crate::geometry::ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(
        offset2_ex_with_interstage(&[invalid], (-1.0, 2.0, JoinType::Square, 3.0), || Ok(()),),
        Err(crate::geometry::ClipperError::CoordinateOutOfRange)
    );

    let near_limit = holeless(&[
        (HI_RANGE - 100, 0),
        (HI_RANGE - 1, 0),
        (HI_RANGE - 1, 100),
        (HI_RANGE - 100, 100),
    ]);
    let near_second = holeless(&[
        (HI_RANGE - 1_000, 0),
        (HI_RANGE - 500, 0),
        (HI_RANGE - 500, 500),
        (HI_RANGE - 1_000, 500),
    ]);
    let reached_second = std::cell::Cell::new(false);
    assert_eq!(
        offset2_ex_with_interstage(
            &[near_second],
            (1.0, 2_000.0, JoinType::Square, 3.0),
            || {
                reached_second.set(true);
                Ok(())
            },
        ),
        Err(crate::geometry::ClipperError::CoordinateOutOfRange)
    );
    assert!(reached_second.get());
    assert_eq!(
        offset_expolygons(&[near_limit], -1.0, JoinType::Square, 3.0),
        Err(crate::geometry::ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22g_positive_offset_drops_a_fully_eroded_tiny_hole() {
    let input = ExPolygon::new(
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        vec![polygon(&[(48, 48), (48, 52), (52, 52), (52, 48)])],
    );

    assert_eq!(
        offset_expolygon(&input, 5.0, JoinType::Miter, 3.0),
        Ok(vec![holeless(&[
            (105, 105),
            (-5, 105),
            (-5, -5),
            (105, -5),
        ])])
    );
}

#[test]
fn task22g_single_expolygon_recovery_uses_even_odd() {
    let duplicate_hole = polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]);
    let input = ExPolygon::new(
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        vec![duplicate_hole.clone(), duplicate_hole],
    );

    assert_eq!(
        offset_expolygon(&input, 5.0, JoinType::Miter, 3.0),
        Ok(vec![holeless(&[
            (105, 105),
            (-5, 105),
            (-5, -5),
            (105, -5),
        ])])
    );
}

#[test]
fn task22g_negative_zero_uses_positive_hole_collection_branch() {
    let duplicate_hole = polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]);
    let input = ExPolygon::new(
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        vec![duplicate_hole.clone(), duplicate_hole],
    );

    assert_eq!(
        offset_expolygon(&input, -0.0, JoinType::Miter, 3.0),
        Ok(vec![holeless(&[(100, 100), (0, 100), (0, 0), (100, 0),])])
    );
}

#[test]
fn task22g_multi_expolygon_recovery_uses_non_zero() {
    let input = holeless(&[(0, 0), (100, 0), (100, 100), (0, 100)]);

    assert_eq!(
        offset_expolygons(&[input.clone(), input], -5.0, JoinType::Miter, 3.0),
        Ok(vec![holeless(&[(95, 95), (5, 95), (5, 5), (95, 5)])])
    );
}

#[test]
fn task22g_negative_hole_difference_uses_non_zero_and_can_consume_contour() {
    let duplicate_hole = polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]);
    let input = ExPolygon::new(
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        vec![duplicate_hole.clone(), duplicate_hole],
    );
    assert_eq!(
        offset_expolygon(&input, -5.0, JoinType::Miter, 3.0),
        Ok(vec![ExPolygon::new(
            polygon(&[(95, 95), (5, 95), (5, 5), (95, 5)]),
            vec![polygon(&[(25, 25), (25, 75), (75, 75), (75, 25)])],
        )])
    );

    let consumed = ExPolygon::new(
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        vec![polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)])],
    );
    assert_eq!(
        offset_expolygon(&consumed, -20.0, JoinType::Miter, 3.0),
        Ok(Vec::new())
    );
}

#[test]
fn task22g_positive_multi_offset_unions_only_multiple_survivors() {
    let overlapping = vec![
        holeless(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        holeless(&[(80, 0), (180, 0), (180, 100), (80, 100)]),
    ];
    assert_eq!(
        offset_expolygons_paths(&overlapping, 10.0, JoinType::Miter, 3.0)
            .map(|paths| path_coordinates(&paths)),
        Ok(vec![vec![(-10, -10), (190, -10), (190, 110), (-10, 110),]])
    );

    let degenerate = holeless(&[(0, 0), (1, 0)]);
    let valid = holeless(&[(0, 0), (100, 0), (100, 100), (0, 100)]);
    let (one_paths, one_survivor) =
        offset_expolygons_raw(&[degenerate, valid.clone()], 5.0, JoinType::Miter, 3.0).unwrap();
    assert_eq!(one_survivor, 1);
    assert_eq!(one_paths.len(), 1);

    let (_, two_survivors) =
        offset_expolygons_raw(&[valid.clone(), valid], 5.0, JoinType::Miter, 3.0).unwrap();
    assert_eq!(two_survivors, 2);
}

#[test]
fn task22g_positive_disjoint_multi_preserves_raw_and_tree_order_contracts() {
    let input = vec![
        holeless(&[(0, 0), (40, 0), (40, 40), (0, 40)]),
        holeless(&[(100, 0), (140, 0), (140, 40), (100, 40)]),
    ];
    let (raw, collected) = offset_expolygons_raw(&input, 5.0, JoinType::Miter, 3.0).unwrap();
    assert_eq!(collected, 2);
    assert_eq!(
        path_coordinates(&raw),
        vec![
            vec![(45, 45), (-5, 45), (-5, -5), (45, -5)],
            vec![(145, 45), (95, 45), (95, -5), (145, -5)],
        ]
    );
    assert_eq!(
        offset_expolygons(&input, 5.0, JoinType::Miter, 3.0),
        Ok(vec![
            holeless(&[(145, 45), (95, 45), (95, -5), (145, -5)]),
            holeless(&[(45, 45), (-5, 45), (-5, -5), (45, -5)]),
        ])
    );
}

#[test]
fn task22g_negative_multi_offset_preserves_no_cross_union_input_order() {
    let overlapping = vec![
        holeless(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        holeless(&[(60, 0), (160, 0), (160, 100), (60, 100)]),
    ];

    let output = offset_expolygons_paths(&overlapping, -10.0, JoinType::Miter, 3.0).unwrap();

    assert_eq!(
        path_coordinates(&output),
        vec![
            vec![(90, 90), (10, 90), (10, 10), (90, 10)],
            vec![(150, 90), (70, 90), (70, 10), (150, 10)],
        ]
    );
}

#[test]
fn task22o19_borrowed_multi_offset_preserves_paths_contract() {
    let input = vec![
        holeless(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        holeless(&[(80, 0), (180, 0), (180, 100), (80, 100)]),
    ];
    let borrowed = input.iter().collect::<Vec<_>>();
    assert_eq!(
        offset_expolygon_refs_paths(&borrowed, 10.0, JoinType::Miter, 3.0),
        offset_expolygons_paths(&input, 10.0, JoinType::Miter, 3.0)
    );
}

#[test]
fn task22o19_borrowed_offset_ignores_out_of_range_degenerate_path_like_orca() {
    let input = vec![holeless(&[(i64::MAX, i64::MAX), (i64::MAX - 1, i64::MAX)])];
    let borrowed = input.iter().collect::<Vec<_>>();
    assert_eq!(
        offset_expolygon_refs_paths(&borrowed, 10.0, JoinType::Miter, 3.0),
        offset_expolygons_paths(&input, 10.0, JoinType::Miter, 3.0)
    );
    assert_eq!(
        offset_expolygon_refs_paths(&borrowed, 10.0, JoinType::Miter, 3.0),
        Ok(Vec::new())
    );
}

#[test]
fn task22o11_opening_is_the_exact_inward_then_outward_offset() {
    let input = vec![fixed_input()];
    assert_eq!(
        opening_ex(&input, 5.0, JoinType::Miter, 3.0),
        offset2_ex(&input, -5.0, 5.0, JoinType::Miter, 3.0)
    );
}

#[test]
fn task22o11_opening_preserves_total_collapse() {
    let input = vec![holeless(&[(0, 0), (8, 0), (8, 8), (0, 8)])];
    assert_eq!(
        opening_ex(&input, 5.0, JoinType::Miter, 3.0),
        Ok(Vec::new())
    );
}
