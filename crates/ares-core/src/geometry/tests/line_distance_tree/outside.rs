use crate::geometry::{Line, LineDistanceTree, Point};

use super::line;

#[test]
fn task22o52_empty_square_and_shared_vertices_match_pinned_classification() {
    assert_eq!(LineDistanceTree::new(&[]).outside(Point::new(0, 0)), 1);

    let lines = square();
    let before = lines;
    let tree = LineDistanceTree::new(&lines);
    for (point, x_counts, y_counts, expected) in [
        (Point::new(5, 5), (1, 1), (1, 1), -1),
        (Point::new(-1, 5), (0, 2), (0, 0), 1),
        (Point::new(11, 5), (2, 0), (0, 0), 1),
        (Point::new(5, 0), (1, 1), (-1, -1), -1),
        (Point::new(5, 10), (0, 0), (-1, -1), 1),
        (Point::new(0, 0), (-1, -1), (-1, -1), 0),
        (Point::new(10, 10), (0, 0), (0, 0), 1),
        (Point::new(5, 11), (0, 0), (2, 0), 1),
    ] {
        assert_eq!(tree.ray_counts_for_test(point, 0), x_counts);
        assert_eq!(tree.ray_counts_for_test(point, 1), y_counts);
        assert_eq!(tree.outside(point), expected);
    }
    assert_eq!(lines, before);
}

#[test]
fn task22o52_multiple_contours_preserve_hole_and_boundary_results() {
    let mut lines = square().to_vec();
    lines.extend([
        line(3, 3, 3, 7),
        line(3, 7, 7, 7),
        line(7, 7, 7, 3),
        line(7, 3, 3, 3),
    ]);
    let tree = LineDistanceTree::new(&lines);
    for (point, expected) in [
        (Point::new(1, 1), -1),
        (Point::new(5, 5), 1),
        (Point::new(3, 5), 0),
        (Point::new(8, 8), -1),
        (Point::new(12, 5), 1),
    ] {
        assert_eq!(tree.outside(point), expected);
    }
}

#[test]
fn task22o52_mixed_x_parity_retries_y_and_second_mismatch_returns_zero() {
    let vertical = [line(5, -5, 5, 5)];
    let tree = LineDistanceTree::new(&vertical);
    assert_eq!(tree.ray_counts_for_test(Point::new(0, 0), 0), (0, 1));
    assert_eq!(tree.ray_counts_for_test(Point::new(0, 0), 1), (0, 0));
    assert_eq!(tree.outside(Point::new(0, 0)), 1);
    assert_eq!(tree.outside(Point::new(5, 0)), 0);

    let diagonal = [line(5, -5, 8, 5)];
    let tree = LineDistanceTree::new(&diagonal);
    assert_eq!(tree.ray_counts_for_test(Point::new(6, 0), 0), (0, 1));
    assert_eq!(tree.ray_counts_for_test(Point::new(6, 0), 1), (1, 0));
    assert_eq!(tree.outside(Point::new(6, 0)), 0);
}

const fn square() -> [Line; 4] {
    [
        line(0, 0, 10, 0),
        line(10, 0, 10, 10),
        line(10, 10, 0, 10),
        line(0, 10, 0, 0),
    ]
}
