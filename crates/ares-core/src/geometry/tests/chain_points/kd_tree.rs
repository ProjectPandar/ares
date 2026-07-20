use crate::geometry::chain_points::kd_tree::KdTree;

#[test]
fn task22m_chain_points_kd_tree_builds_fixed_layout_and_visits_in_source_order() {
    let points = [
        [9.0, 4.0],
        [1.0, 7.0],
        [5.0, 2.0],
        [3.0, 8.0],
        [7.0, 0.0],
        [2.0, 6.0],
        [6.0, 3.0],
    ];
    let tree = KdTree::new(&points);
    assert_eq!(tree.nodes(), [2, 1, 6, 5, 3, 4, 0, usize::MAX]);

    let mut visited = Vec::new();
    assert_eq!(
        tree.closest(&points, [4.9, 2.1], |index| {
            visited.push(index);
            true
        }),
        Some(2)
    );
    assert_eq!(visited, [2, 1, 5, 6, 4]);

    visited.clear();
    assert_eq!(
        tree.closest(&points, [4.9, 2.1], |index| {
            visited.push(index);
            index != 2
        }),
        Some(6)
    );
    assert_eq!(visited, [2, 1, 5, 3, 6, 4, 0]);
}

#[test]
fn task22m_chain_points_kd_tree_preserves_equal_distance_later_visit() {
    let points = [[-1.0, 0.0], [1.0, 0.0]];
    let tree = KdTree::new(&points);
    let mut visited = Vec::new();

    assert_eq!(tree.nodes(), [0, usize::MAX, 1, usize::MAX]);
    assert_eq!(
        tree.closest(&points, [0.0, 0.0], |index| {
            visited.push(index);
            true
        }),
        Some(1)
    );
    assert_eq!(visited, [0, 1]);
}

#[test]
fn task22m_chain_points_kd_tree_handles_empty_negative_and_large_cast_points() {
    assert!(KdTree::new(&[]).nodes().is_empty());

    let negative = [[-10.0, -10.0], [-2.0, -3.0], [4.0, -4.0]];
    let tree = KdTree::new(&negative);
    assert_eq!(tree.nodes(), [1, 0, 2, usize::MAX]);
    assert_eq!(tree.closest(&negative, [-3.0, -2.0], |_| true), Some(1));

    let large = [
        [(1_i64 << 53) as f64, 0.0],
        [((1_i64 << 53) + 1) as f64, 0.0],
    ];
    assert_eq!(large[0], large[1]);
    let tree = KdTree::new(&large);
    assert_eq!(tree.closest(&large, large[0], |_| true), Some(1));
}
