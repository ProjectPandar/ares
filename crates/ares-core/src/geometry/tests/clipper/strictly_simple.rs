use super::helpers::{execute, polygon, polygons};
use crate::geometry::clipper::{
    ClipOperation, Clipper, ClipperOptions, FillRule, MaximaCursor, PathRole,
};

mod simple_ownership;
mod simple_polygons;

fn trace_cursor(
    maxima: &[i64],
    mut cursor: MaximaCursor,
    crossings: &[(i64, bool, i32)],
) -> (Vec<i64>, Vec<i64>) {
    let mut consumed = Vec::new();
    let mut inserted = Vec::new();
    for &(crossing_x, assigned, wind_delta) in crossings {
        while let Some(x) = cursor.pop_before(maxima, crossing_x) {
            consumed.push(x);
            if assigned && wind_delta != 0 {
                inserted.push(x);
            }
        }
    }
    (consumed, inserted)
}

fn strict_union(subject: &[&[(i64, i64)]]) -> Vec<crate::geometry::Polygon> {
    strict_union_with_collected(subject).0
}

fn strict_union_with_collected(
    subject: &[&[(i64, i64)]],
) -> (Vec<crate::geometry::Polygon>, Vec<i64>) {
    let mut clipper = Clipper::new(ClipperOptions {
        strictly_simple: true,
        ..ClipperOptions::default()
    });
    clipper
        .add_closed_paths(&polygons(subject), PathRole::Subject)
        .expect("fixed strict coordinates are in range");
    let output = clipper
        .execute_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
        .expect("closed Clipper execution accepts flat output");
    (output, clipper.collected_strict_maxima_for_test().to_vec())
}

fn strict_intersection(
    subject: &[&[(i64, i64)]],
    clip: &[&[(i64, i64)]],
) -> Vec<crate::geometry::Polygon> {
    execute(
        polygons(subject),
        polygons(clip),
        ClipOperation::Intersection,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions {
            strictly_simple: true,
            ..ClipperOptions::default()
        },
    )
}

#[test]
fn task22i_strict_type3_top_touch_matches_fixed_oracle() {
    let actual = strict_union(&[
        &[(0, 20), (10, 0), (20, 20)],
        &[(-5, 10), (0, 5), (5, 10), (0, 15)],
    ]);

    assert_eq!(
        actual,
        polygons(&[
            &[(20, 20), (0, 20), (5, 10), (10, 0)],
            &[(5, 10), (0, 15), (-5, 10), (0, 5)],
        ])
    );
}

#[test]
fn task22i_strict_horizontal_maxima_excludes_horizontal_pair_and_inserts_control() {
    let (actual, collected) = strict_union_with_collected(&[
        &[(5, 0), (0, 10), (10, 10)],
        &[(0, 0), (10, 0), (10, 5), (0, 5)],
    ]);

    assert_eq!(collected, [5]);
    assert_eq!(
        actual,
        polygons(&[
            &[(3, 5), (0, 5), (0, 0), (5, 0)],
            &[(10, 10), (0, 10), (3, 5), (8, 5)],
            &[(5, 0), (10, 0), (10, 5), (8, 5)],
        ])
    );
}

#[test]
fn task22i_strict_maxima_advance_across_unassigned_to_assigned_horizontal() {
    let actual = strict_intersection(
        &[
            &[(0, 0), (10, 0), (10, 5), (0, 5)],
            &[(2, 0), (3, 10), (1, 10)],
            &[(6, 0), (7, 10), (5, 10)],
        ],
        &[&[(3, -5), (8, -5), (8, 6), (3, 6)]],
    );

    assert_eq!(
        actual,
        polygons(&[&[
            (8, 5),
            (7, 5),
            (7, 6),
            (5, 6),
            (6, 5),
            (3, 5),
            (3, 0),
            (6, 0),
            (8, 0),
        ]])
    );
}

#[test]
fn task22i_strict_rtl_maximum_is_inserted_before_out_of_range_crossing() {
    let actual = strict_union(&[
        &[(0, 0), (10, 0), (10, 10), (5, 10), (5, 5), (0, 5)],
        &[(2, 5), (0, 15), (4, 15)],
        &[(-10, 0), (-5, 0), (-5, 10), (-10, 10)],
    ]);

    assert_eq!(
        actual,
        polygons(&[
            &[(4, 15), (0, 15), (2, 5)],
            &[(-5, 10), (-10, 10), (-10, 0), (-5, 0)],
            &[(10, 10), (5, 10), (5, 5), (2, 5), (0, 5), (0, 0), (10, 0)],
        ])
    );
}

#[test]
fn task22i_strict_ltr_endpoint_maximum_is_excluded() {
    let actual = strict_union(&[
        &[(10, 0), (5, 10), (15, 10)],
        &[(0, 0), (10, 0), (10, 5), (0, 5)],
    ]);

    assert_eq!(
        actual,
        polygons(&[
            &[(15, 10), (5, 10), (8, 5), (10, 5), (10, 0)],
            &[(8, 5), (0, 5), (0, 0), (10, 0)],
        ])
    );
}

#[test]
fn task22i_strict_rtl_endpoint_maximum_is_excluded() {
    let actual = strict_union(&[
        &[(0, 0), (10, 0), (10, 10), (5, 10), (5, 5), (0, 5)],
        &[(0, 5), (-5, 15), (5, 15)],
    ]);

    assert_eq!(
        actual,
        polygons(&[
            &[(5, 15), (-5, 15), (0, 5)],
            &[(10, 10), (5, 10), (5, 5), (0, 5), (0, 0), (10, 0)],
        ])
    );
}

#[test]
fn task22i_strict_maxima_cursor_consumes_left_to_right_in_fixed_order() {
    let maxima = [0, 2, 4, 6, 8];
    let cursor = MaximaCursor::left_to_right(&maxima, 0, 10);

    assert_eq!(
        trace_cursor(&maxima, cursor, &[(5, true, 1), (9, true, 1)]),
        (vec![2, 4, 6, 8], vec![2, 4, 6, 8])
    );
}

#[test]
fn task22i_strict_maxima_cursor_consumes_right_to_left_and_includes_bottom() {
    let maxima = [2, 4, 6, 8, 10, 11];
    let cursor = MaximaCursor::right_to_left(&maxima, 10, 0);

    assert_eq!(
        trace_cursor(
            &maxima,
            cursor,
            &[(7, true, 1), (3, true, 1), (-1, true, 1)],
        ),
        (vec![10, 8, 6, 4, 2], vec![10, 8, 6, 4, 2])
    );
}

#[test]
fn task22i_strict_maxima_cursor_excludes_endpoint_first_candidates() {
    let left_maxima = [0, 10, 11];
    let right_maxima = [-1, 0, 11];

    assert_eq!(
        trace_cursor(
            &left_maxima,
            MaximaCursor::left_to_right(&left_maxima, 0, 10),
            &[(12, true, 1)],
        ),
        (Vec::new(), Vec::new())
    );
    assert_eq!(
        trace_cursor(
            &right_maxima,
            MaximaCursor::right_to_left(&right_maxima, 10, 0),
            &[(-2, true, 1)],
        ),
        (Vec::new(), Vec::new())
    );
}

#[test]
fn task22i_strict_maxima_cursor_consumes_before_the_horizontal_range_break() {
    let maxima = [0, 2, 10, 11, 20];
    let cursor = MaximaCursor::left_to_right(&maxima, 0, 10);

    assert_eq!(
        trace_cursor(&maxima, cursor, &[(12, true, 1)]),
        (vec![2, 10, 11], vec![2, 10, 11])
    );
}

#[test]
fn task22i_strict_maxima_cursor_advances_while_output_is_unassigned() {
    let maxima = [2, 4, 6];
    let cursor = MaximaCursor::left_to_right(&maxima, 0, 8);
    assert_eq!(
        trace_cursor(
            &maxima,
            cursor,
            &[(3, false, 1), (5, true, 1), (9, true, 1)],
        ),
        (vec![2, 4, 6], vec![4, 6])
    );

    let cursor = MaximaCursor::left_to_right(&maxima, 0, 8);
    assert_eq!(
        trace_cursor(
            &maxima,
            cursor,
            &[(3, false, 1), (5, false, 1), (9, false, 1)],
        ),
        (vec![2, 4, 6], Vec::new())
    );

    let cursor = MaximaCursor::left_to_right(&maxima, 0, 8);
    assert_eq!(
        trace_cursor(&maxima, cursor, &[(9, true, 0)]),
        (vec![2, 4, 6], Vec::new())
    );
}

#[test]
fn task22i_strict_maxima_state_is_empty_after_repeated_execution_and_clear() {
    let input = polygon(&[(0, 10), (5, 0), (10, 10)]);
    let mut clipper = Clipper::new(ClipperOptions {
        strictly_simple: true,
        ..ClipperOptions::default()
    });

    let mut outputs = Vec::new();
    for _ in 0..2 {
        clipper
            .add_closed_path(&input, PathRole::Subject)
            .expect("fixed triangle is in range");
        outputs.push(clipper.execute_paths(
            ClipOperation::Union,
            FillRule::NonZero,
            FillRule::NonZero,
        ));
        assert!(clipper.strict_maxima_for_test().is_empty());
    }
    assert_eq!(outputs[0], outputs[1]);

    clipper.seed_strict_maxima_for_test(&[7, 3]);
    clipper.clear();
    assert!(clipper.strict_maxima_for_test().is_empty());
}
