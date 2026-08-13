use crate::geometry::{Line, Point};

use super::super::sections as production_sections;
use super::line;

#[test]
fn task22o53_scanlines_use_area_x_anchor_y_and_center_expression_order() {
    let minimum = 9_007_199_254_740_993;
    assert_eq!(
        production_sections::vertical_lines_for_test(
            minimum,
            minimum + 20,
            Point::new(-100, -30),
            Point::new(100, 40),
            10,
        ),
        vec![
            line(9_007_199_254_740_996, -40, 9_007_199_254_740_996, 50),
            line(9_007_199_254_741_008, -40, 9_007_199_254_741_008, 50),
        ]
    );
}

#[test]
fn task22o53_structural_hit_stream_visits_every_adjacent_window() {
    assert_eq!(
        production_sections::all_adjacent_sections_for_test(&[
            Point::new(5, 0),
            Point::new(5, 10),
            Point::new(5, 20),
            Point::new(5, 30),
        ]),
        vec![line(5, 0, 5, 10), line(5, 10, 5, 20), line(5, 20, 5, 30)]
    );
}

#[test]
fn task22o53_midpoint_adds_in_integer_space_before_division() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    assert_eq!(
        production_sections::midpoint_for_test(
            Point::new(high - 1_000, -5),
            Point::new(high - 500, -4),
        ),
        Point::new(high - 750, -4)
    );
}

#[test]
fn task22o53_anchor_bounds_skip_equal_endpoints_and_use_mixed_width_arithmetic() {
    let anchors = vec![
        (Point::new(0, 5), 0),
        (Point::new(0, 10), 1),
        (Point::new(0, 20), 2),
        (Point::new(0, 25), 3),
    ];
    let mut section = line(0, 10, 0, 20);
    production_sections::extend_to_anchors_for_test(&mut section, &anchors, 3);
    assert_eq!(section, line(0, 2, 0, 28));

    let high = 9_007_199_254_740_993;
    let mut large = Line::new(Point::new(0, high + 10), Point::new(0, high + 20));
    production_sections::extend_to_anchors_for_test(&mut large, &[(Point::new(0, high), 0)], 1);
    assert_eq!(large.a.y(), 9_007_199_254_740_991);
}

#[test]
fn task22o53_inclusive_overlap_merges_once_removes_zero_and_keeps_source_order() {
    let prepared = production_sections::prepare_order_for_test(vec![
        line(5, 0, 5, 10),
        line(5, 10, 5, 20),
        line(5, 30, 5, 40),
    ]);
    assert_eq!(prepared, vec![line(5, 0, 5, 20), line(5, 30, 5, 40)]);

    let reordered = production_sections::prepare_order_for_test(vec![
        line(5, 30, 5, 40),
        line(5, 0, 5, 10),
        line(5, 20, 5, 25),
    ]);
    assert_eq!(
        reordered,
        vec![line(5, 0, 5, 10), line(5, 20, 5, 25), line(5, 30, 5, 40)]
    );
}

#[test]
fn task22o53_structural_fixed_msvc_comparator_freezes_large_permutation() {
    let input = (0..40)
        .map(|index| line(index, 0, index, 100))
        .collect::<Vec<_>>();
    let actual = production_sections::sort_sections_for_test(input.clone());
    let actual_ids = actual
        .iter()
        .map(|section| section.a.x())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_ids,
        vec![
            19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 39, 20, 0, 38, 37,
            36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21,
        ]
    );
    assert_ne!(actual, input);
}
