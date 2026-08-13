use crate::geometry::{LineDistanceTree, Point};

use super::line;

#[test]
fn task22o52_empty_crossings_and_fractional_hits_match_pinned_oracle() {
    let empty = LineDistanceTree::new(&[]);
    assert!(empty.intersections_sorted(line(0, -10, 0, 10)).is_empty());

    let lines = [line(-10, 0, 10, 0), line(-10, 5, 10, 5), line(-7, -8, 8, 7)];
    let before = lines;
    let tree = LineDistanceTree::new(&lines);
    let query = line(0, -20, 0, 20);
    let before_query = query;
    assert_eq!(
        tree.intersections_in_traversal_order_for_test(query),
        vec![
            (Point::new(0, 0), 0),
            (Point::new(0, 5), 1),
            (Point::new(0, -1), 2),
        ]
    );
    let expected = vec![
        (Point::new(0, -1), 2),
        (Point::new(0, 0), 0),
        (Point::new(0, 5), 1),
    ];
    assert_eq!(tree.intersections_sorted(query), expected);
    assert_eq!(tree.intersections_sorted(query), expected);
    assert_eq!(lines, before);
    assert_eq!(query, before_query);

    let fractional = [
        line(-10, -5, 10, 6),
        line(-9, 7, 8, -4),
        line(-20, -9, 20, -8),
    ];
    let tree = LineDistanceTree::new(&fractional);
    assert_eq!(
        tree.intersections_sorted(line(-7, -20, -2, 20)),
        vec![
            (Point::new(-5, -8), 2),
            (Point::new(-4, -2), 0),
            (Point::new(-4, 3), 1),
        ]
    );
}

#[test]
fn task22o52_endpoint_duplicates_and_integer_determinants_keep_source_rules() {
    let endpoint_lines = [
        line(-10, 0, 0, 0),
        line(0, 0, 10, 0),
        line(0, -10, 0, 10),
        line(-5, -5, 5, 5),
    ];
    let tree = LineDistanceTree::new(&endpoint_lines);
    assert_eq!(
        tree.intersections_sorted(line(-10, 0, 10, 0)),
        vec![(Point::new(0, 0), 3), (Point::new(0, 0), 2)]
    );

    let determinant_lines = [
        line(0, 0, 1, 0),
        line(0, 0, 1_000_000_001, 1_000_000_000),
        line(0, 0, 1_000_000_000, 999_999_999),
        line(0, 0, 1_000_000_000, -1),
        line(0, 0, 1, 1),
        line(0, 0, -1, -1),
    ];
    let tree = LineDistanceTree::new(&determinant_lines);
    let hits = tree.intersections_sorted(line(0, 0, 1_000_000_000, 999_999_999));
    assert!(hits.iter().all(|hit| hit.0 == Point::new(0, 0)));
    assert_eq!(indices(&hits), vec![0, 4, 5, 3]);
}

#[test]
fn task22o52_hirange_source_defined_intersections_preserve_cast_order() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let lines = [
        line(high - 1_000, high - 1_000, high - 100, high - 100),
        line(high - 900, high - 100, high - 100, high - 900),
    ];
    let tree = LineDistanceTree::new(&lines);
    assert_eq!(
        tree.intersections_sorted(line(high - 800, high - 1_200, high - 800, high)),
        vec![
            (
                Point::new(4_611_686_018_427_386_880, 4_611_686_018_427_387_392),
                0,
            ),
            (
                Point::new(4_611_686_018_427_386_880, 4_611_686_018_427_387_904),
                1,
            ),
        ]
    );
}

#[test]
fn task22o52_sort_key_subtracts_in_integer_space_before_f64_promotion() {
    let origin = Point::new(9_007_199_254_740_993, -9_007_199_254_740_993);
    let point = Point::new(9_007_199_254_740_994, -9_007_199_254_740_992);
    assert_eq!(
        LineDistanceTree::squared_intersection_sort_key_bits_for_test(origin, point),
        2.0_f64.to_bits()
    );
}

#[test]
fn task22o52_fixed_msvc_sort_freezes_large_equal_key_groups() {
    let lines = (0..40)
        .map(|index| line(-10, index % 5, 10, index % 5))
        .collect::<Vec<_>>();
    let tree = LineDistanceTree::new(&lines);
    let query = line(0, -10, 0, 10);
    let raw = tree.intersections_in_traversal_order_for_test(query);
    assert_eq!(
        raw,
        vec![
            (Point::new(0, 0), 0),
            (Point::new(0, 0), 25),
            (Point::new(0, 1), 26),
            (Point::new(0, 2), 27),
            (Point::new(0, 4), 24),
            (Point::new(0, 3), 23),
            (Point::new(0, 3), 28),
            (Point::new(0, 1), 21),
            (Point::new(0, 2), 22),
            (Point::new(0, 4), 29),
            (Point::new(0, 0), 30),
            (Point::new(0, 0), 35),
            (Point::new(0, 1), 36),
            (Point::new(0, 2), 37),
            (Point::new(0, 4), 34),
            (Point::new(0, 3), 33),
            (Point::new(0, 0), 20),
            (Point::new(0, 1), 31),
            (Point::new(0, 2), 32),
            (Point::new(0, 4), 19),
            (Point::new(0, 3), 18),
            (Point::new(0, 0), 5),
            (Point::new(0, 1), 6),
            (Point::new(0, 2), 7),
            (Point::new(0, 4), 4),
            (Point::new(0, 3), 3),
            (Point::new(0, 3), 8),
            (Point::new(0, 1), 1),
            (Point::new(0, 2), 2),
            (Point::new(0, 4), 9),
            (Point::new(0, 0), 10),
            (Point::new(0, 0), 15),
            (Point::new(0, 1), 16),
            (Point::new(0, 2), 17),
            (Point::new(0, 4), 14),
            (Point::new(0, 3), 13),
            (Point::new(0, 3), 38),
            (Point::new(0, 1), 11),
            (Point::new(0, 2), 12),
            (Point::new(0, 4), 39),
        ]
    );
    let sorted = indices(&tree.intersections_sorted(query));
    assert_eq!(
        sorted,
        vec![
            0, 25, 30, 35, 5, 20, 10, 15, 26, 1, 31, 21, 36, 6, 16, 11, 27, 32, 22, 7, 37, 2, 17,
            12, 28, 33, 18, 3, 13, 38, 8, 23, 34, 24, 14, 29, 9, 4, 19, 39,
        ]
    );
    assert_ne!(
        sorted,
        vec![
            0, 25, 30, 35, 20, 5, 10, 15, 26, 21, 36, 31, 6, 1, 16, 11, 27, 22, 37, 32, 7, 2, 17,
            12, 23, 28, 33, 18, 3, 8, 13, 38, 24, 29, 34, 19, 4, 9, 14, 39,
        ]
    );
}

fn indices(hits: &[(Point, usize)]) -> Vec<usize> {
    hits.iter().map(|hit| hit.1).collect()
}
