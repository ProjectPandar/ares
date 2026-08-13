mod intersections;
mod outside;

use crate::geometry::{
    Line, LineDistanceTree, NearestLine, Point,
    line_distance_tree::{NodeSnapshotForTest, NodeStateForTest},
};

#[test]
fn task22o50_empty_and_single_line_freeze_layout_and_projection_branches() {
    let empty = LineDistanceTree::new(&[]);
    assert!(empty.node_snapshots().is_empty());
    assert_eq!(empty.nearest(Point::new(0, 0)), None);

    let single_line = [line(0, 0, 10, 0)];
    let single = LineDistanceTree::new(&single_line);
    assert_eq!(single.node_snapshots(), vec![leaf(0, 0, 0, 10, 0)]);
    let named_result: Option<NearestLine> = single.nearest(Point::new(4, 2));
    assert_eq!(named_result.unwrap().line_index, 0);

    let lines = [line(0, 0, 10, 0), line(5, 5, 5, 5)];
    let before = lines;
    let first = LineDistanceTree::new(&lines);
    let second = LineDistanceTree::new(&lines);
    for (point, index, squared, nearest) in [
        (Point::new(-3, 4), 0, 25.0_f64, [0.0, 0.0]),
        (Point::new(13, 4), 0, 25.0, [10.0, 0.0]),
        (Point::new(5, 4), 1, 1.0, [5.0, 5.0]),
        (Point::new(4, 2), 0, 4.0, [4.0, 0.0]),
    ] {
        let actual = first.nearest(point).unwrap();
        assert_eq!(actual.line_index, index);
        assert_eq!(actual.squared_distance.to_bits(), squared.to_bits());
        assert_eq!(
            actual.nearest_point.map(f64::to_bits),
            nearest.map(f64::to_bits)
        );
        assert_eq!(second.nearest(point), Some(actual));
    }
    assert_eq!(lines, before);
}

#[test]
fn task22o50_three_line_tree_matches_pinned_layout_and_queries() {
    let lines = [line(0, 0, 10, 0), line(0, 20, 10, 20), line(30, -5, 30, 5)];
    let tree = LineDistanceTree::new(&lines);
    assert_eq!(
        tree.node_snapshots(),
        vec![
            inner(0, -5, 30, 20),
            inner(0, 0, 10, 20),
            leaf(2, 30, -5, 30, 5),
            leaf(0, 0, 0, 10, 0),
            leaf(1, 0, 20, 10, 20),
            unused(),
            unused(),
        ]
    );
    assert_query(&tree, Point::new(5, 10), 1, 100.0, [5.0, 20.0]);
    assert_query(&tree, Point::new(30, 0), 2, 0.0, [30.0, 0.0]);
    assert_query(&tree, Point::new(20, 0), 2, 100.0, [30.0, 0.0]);
}

#[test]
fn task22o50_containment_and_bbox_ties_keep_source_tree_visit_ownership() {
    let containing = [line(-10, 0, 10, 0), line(0, -10, 0, 10)];
    let tree = LineDistanceTree::new(&containing);
    assert_query(&tree, Point::new(0, 0), 0, 0.0, [0.0, 0.0]);
    assert_query(&tree, Point::new(1, 1), 1, 1.0, [0.0, 1.0]);

    let equidistant = [line(-20, -10, -20, 10), line(20, -10, 20, 10)];
    let tree = LineDistanceTree::new(&equidistant);
    assert_query(&tree, Point::new(0, 0), 1, 400.0, [20.0, 0.0]);
}

#[test]
fn task22o50_equal_centroids_preserve_quickselect_permutation_and_unused_slots() {
    let lines = [
        line(-8, 0, 8, 0),
        line(0, -8, 0, 8),
        line(-6, -6, 6, 6),
        line(-6, 6, 6, -6),
        line(-4, 0, 4, 0),
    ];
    let tree = LineDistanceTree::new(&lines);
    assert_eq!(
        tree.node_snapshots(),
        vec![
            inner(-8, -8, 8, 8),
            inner(-8, -6, 8, 6),
            inner(-4, -8, 4, 8),
            inner(-8, -6, 8, 6),
            leaf(2, -6, -6, 6, 6),
            leaf(1, 0, -8, 0, 8),
            leaf(4, -4, 0, 4, 0),
            leaf(0, -8, 0, 8, 0),
            leaf(3, -6, -6, 6, 6),
            unused(),
            unused(),
            unused(),
            unused(),
            unused(),
            unused(),
        ]
    );
    assert_query(&tree, Point::new(0, 0), 0, 0.0, [0.0, 0.0]);
    assert_query(&tree, Point::new(9, 1), 0, 2.0, [8.0, 0.0]);
}

#[test]
fn task22o50_quickselect_swap_branches_and_multi_round_layout_match_pinned_orca() {
    for (values, expected) in [
        (&[30, 0, 10, 20, 40][..], &[3, 0, 4, 1, 2][..]),
        (&[30, 20, 40, 10, 0], &[1, 0, 2, 4, 3]),
        (&[0, 30, 40, 20, 10], &[3, 1, 2, 0, 4]),
        (&[70, 10, 60, 20, 50, 30, 40], &[0, 1, 3, 5, 6, 4, 2]),
    ] {
        let lines = values
            .iter()
            .map(|&value| line(value, 0, value, 0))
            .collect::<Vec<_>>();
        let tree = LineDistanceTree::new(&lines);
        assert_eq!(leaf_order(&tree), expected);
    }
}

#[test]
fn task22o50_centroids_preserve_source_sum_f64_half_and_truncation() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let lines = [line(high - 100, 0, high - 50, 0), line(-9, -1, -6, -4)];
    let tree = LineDistanceTree::new(&lines);
    assert_eq!(
        tree.centroids_for_test(),
        vec![Point::new(1_i64 << 62, 0), Point::new(-7, -2)]
    );
    assert_ne!(tree.centroids_for_test()[0].x(), high - 75);
}

#[test]
fn task22o50_bbox_distance_preserves_mixed_truncation_and_fixed_accumulation() {
    assert_eq!(
        LineDistanceTree::exterior_distance_squared_for_test(
            Point::new(10, 0),
            Point::new(20, 0),
            [8.75, 0.0],
        )
        .to_bits(),
        1.0_f64.to_bits()
    );

    let fixed = LineDistanceTree::exterior_distance_squared_for_test(
        Point::new(0, 0),
        Point::new(0, 0),
        [2_000_000_000.0, 1_000_000_024.0],
    );
    let pinned_fixed = 5_000_000_048_000_000_576_i64 as f64;
    let per_axis_f64 = 2_000_000_000.0_f64.powi(2) + 1_000_000_024.0_f64.powi(2);
    assert_eq!(fixed.to_bits(), pinned_fixed.to_bits());
    assert_ne!(fixed.to_bits(), per_axis_f64.to_bits());

    let extended = LineDistanceTree::exterior_distance_squared_for_test(
        Point::new(0, 0),
        Point::new(0, 0),
        [i64::MIN as f64, i64::MIN as f64],
    );
    let widened = (i128::from(i64::MAX) * i128::from(i64::MAX) * 2) as f64;
    assert_eq!(extended.to_bits(), widened.to_bits());
}

#[test]
fn task22o50_query_roundtrip_above_2pow53_and_hirange_matches_pinned_literals() {
    let base = (1_i64 << 53) + 3;
    let lines = [
        line(base, 0, base + 20, 0),
        line(base + 40, -10, base + 40, 10),
    ];
    let tree = LineDistanceTree::new(&lines);
    assert_query(
        &tree,
        Point::new(base + 11, 7),
        0,
        49.0,
        [9_007_199_254_741_008.0, 0.0],
    );
    assert_query(
        &tree,
        Point::new(base + 30, 0),
        0,
        81.0,
        [9_007_199_254_741_016.0, 0.0],
    );

    let high = 0x3fff_ffff_ffff_ffff_i64;
    let lines = [
        line(high - 100, 0, high - 50, 0),
        line(high - 20, -10, high - 20, 10),
    ];
    let tree = LineDistanceTree::new(&lines);
    assert_query(
        &tree,
        Point::new(high - 75, 3),
        1,
        441.0,
        [4_611_686_018_427_387_904.0, 3.0],
    );
    assert_query(
        &tree,
        Point::new(high - 30, 0),
        1,
        441.0,
        [4_611_686_018_427_387_904.0, 0.0],
    );
}

fn leaf_order(tree: &LineDistanceTree<'_>) -> Vec<usize> {
    tree.node_snapshots()
        .into_iter()
        .filter_map(|node| match node.state {
            NodeStateForTest::Leaf(index) => Some(index),
            NodeStateForTest::Unused | NodeStateForTest::Inner => None,
        })
        .collect()
}

fn assert_query(
    tree: &LineDistanceTree<'_>,
    point: Point,
    line_index: usize,
    squared_distance: f64,
    nearest_point: [f64; 2],
) {
    let actual = tree.nearest(point).unwrap();
    assert_eq!(actual.line_index, line_index);
    assert_eq!(
        actual.squared_distance.to_bits(),
        squared_distance.to_bits()
    );
    assert_eq!(
        actual.nearest_point.map(f64::to_bits),
        nearest_point.map(f64::to_bits)
    );
}

const fn line(ax: i64, ay: i64, bx: i64, by: i64) -> Line {
    Line::new(Point::new(ax, ay), Point::new(bx, by))
}

const fn inner(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> NodeSnapshotForTest {
    snapshot(NodeStateForTest::Inner, min_x, min_y, max_x, max_y)
}

const fn leaf(index: usize, min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> NodeSnapshotForTest {
    snapshot(NodeStateForTest::Leaf(index), min_x, min_y, max_x, max_y)
}

const fn unused() -> NodeSnapshotForTest {
    snapshot(
        NodeStateForTest::Unused,
        i64::MAX,
        i64::MAX,
        i64::MIN,
        i64::MIN,
    )
}

const fn snapshot(
    state: NodeStateForTest,
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
) -> NodeSnapshotForTest {
    NodeSnapshotForTest {
        state,
        min: Point::new(min_x, min_y),
        max: Point::new(max_x, max_y),
    }
}
