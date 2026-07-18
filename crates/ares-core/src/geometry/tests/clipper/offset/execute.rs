use super::helpers::{coordinates, polygon};
use crate::geometry::Polygon;
use crate::geometry::clipper::{
    ClipOperation, ClipperError, ClipperOffset, ClipperOptions, ClosedClipper, FillRule, IntBounds,
    JoinType, PathRole, PolyNode, PolyTree, negative_outer, offset_paths, offset_paths_tree,
    raw_offset_paths,
};

#[derive(Debug, Eq, PartialEq)]
struct NodeSnapshot {
    parent: Option<usize>,
    sibling: usize,
    hole: bool,
    contour: Vec<(i64, i64)>,
}

fn execute(points: &[(i64, i64)], delta: f64) -> Result<Vec<Vec<(i64, i64)>>, ClipperError> {
    let mut offset = ClipperOffset::default();
    offset.add_closed_path(&polygon(points), JoinType::Miter);
    offset
        .execute_paths(delta)
        .map(|paths| paths.iter().map(coordinates).collect())
}

fn execute_many(paths: &[Polygon], delta: f64) -> Result<Vec<Vec<(i64, i64)>>, ClipperError> {
    let mut offset = ClipperOffset::default();
    offset.add_closed_paths(paths, JoinType::Miter);
    offset
        .execute_paths(delta)
        .map(|paths| paths.iter().map(coordinates).collect())
}

fn tree(paths: &[Polygon]) -> PolyTree {
    let mut clipper = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(clipper.add_closed_paths(paths, PathRole::Subject), Ok(true));
    clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
}

fn snapshot_tree(tree: &PolyTree) -> Vec<NodeSnapshot> {
    fn visit(
        node: PolyNode<'_>,
        parent: Option<usize>,
        sibling: usize,
        snapshots: &mut Vec<NodeSnapshot>,
    ) {
        let index = snapshots.len();
        snapshots.push(NodeSnapshot {
            parent,
            sibling,
            hole: node.is_hole(),
            contour: coordinates(node.contour()),
        });
        for (child_index, child) in node.children().enumerate() {
            visit(child, Some(index), child_index, snapshots);
        }
    }

    let mut snapshots = Vec::with_capacity(tree.total());
    for (root_index, root) in tree.children().enumerate() {
        visit(root, None, root_index, &mut snapshots);
    }
    snapshots
}

#[test]
fn task22g_internal_signed_zero_and_sub_tolerance_use_exact_cleanup_sign() {
    let square = [(0, 0), (100, 0), (100, 100), (0, 100)];
    let expected = Ok(vec![vec![(100, 100), (0, 100), (0, 0), (100, 0)]]);

    assert_eq!(execute(&square, 0.0), expected);
    assert_eq!(execute(&square, -0.0), expected);
    assert_eq!(execute(&square, 5.0e-21), expected);
    assert_eq!(execute(&square, -5.0e-21), expected);

    let mixed_winding = vec![
        polygon(&[(0, 100), (40, 100), (40, 140), (0, 140)]),
        polygon(&[(100, 0), (100, 40), (140, 40), (140, 0)]),
    ];
    let upper = Ok(vec![vec![(40, 140), (0, 140), (0, 100), (40, 100)]]);
    for delta in [5.0e-21, 0.0, -0.0, -5.0e-21] {
        assert_eq!(execute_many(&mixed_winding, delta), upper);
    }
}

#[test]
fn task22g_internal_execute_cleans_positive_negative_and_complete_erosion() {
    let square = [(0, 0), (100, 0), (100, 100), (0, 100)];

    assert_eq!(
        execute(&square, 10.0),
        Ok(vec![vec![(110, 110), (-10, 110), (-10, -10), (110, -10)]])
    );
    assert_eq!(
        execute(&square, -10.0),
        Ok(vec![vec![(90, 90), (10, 90), (10, 10), (90, 10)]])
    );
    assert_eq!(
        execute(&[(0, 0), (8, 0), (8, 8), (0, 8)], -5.0),
        Ok(Vec::new())
    );

    let mut empty = ClipperOffset::default();
    assert_eq!(empty.execute_paths(-5.0), Ok(Vec::new()));
    assert_eq!(empty.execute_polytree(-5.0), Ok(PolyTree::empty()));
}

#[test]
fn task22g_internal_execute_cleans_concave_positive_and_negative_offsets() {
    let concave = [(0, 0), (100, 0), (100, 40), (40, 40), (40, 100), (0, 100)];

    assert_eq!(
        execute(&concave, 10.0),
        Ok(vec![vec![
            (110, 50),
            (50, 50),
            (50, 110),
            (-10, 110),
            (-10, -10),
            (110, -10),
        ]])
    );
    assert_eq!(
        execute(&concave, -10.0),
        Ok(vec![vec![
            (90, 30),
            (30, 30),
            (30, 90),
            (10, 90),
            (10, 10),
            (90, 10),
        ]])
    );
}

#[test]
fn task22g_normalized_bounds_and_negative_outer_match_fixed_clipper() {
    let mut empty = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(empty.bounds(), IntBounds::default());
    assert_eq!(
        negative_outer(empty.bounds()),
        polygon(&[(-10, 10), (10, 10), (10, -10), (-10, -10)])
    );

    assert_eq!(
        empty.add_closed_path(
            &polygon(&[(1_000_000, 1_000_000), (2_000_000, 2_000_000)]),
            PathRole::Subject,
        ),
        Ok(false)
    );
    let bounds_input = polygon(&[(-4, 7), (13, 7), (13, -2), (-4, -2)]);
    assert_eq!(
        empty.add_closed_path(&bounds_input, PathRole::Subject),
        Ok(true)
    );
    let bounds = IntBounds {
        left: -4,
        top: -2,
        right: 13,
        bottom: 7,
    };
    assert_eq!(empty.bounds(), bounds);
    assert_eq!(
        negative_outer(bounds),
        polygon(&[(-14, 17), (23, 17), (23, -12), (-14, -12)])
    );
}

#[test]
fn task22g_polytree_outer_removal_promotes_children_and_preserves_order() {
    let input = vec![
        polygon(&[(0, 0), (200, 0), (200, 200), (0, 200)]),
        polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)]),
        polygon(&[(20, 20), (80, 20), (80, 80), (20, 80)]),
        polygon(&[(110, 10), (110, 90), (190, 90), (190, 10)]),
        polygon(&[(120, 20), (180, 20), (180, 80), (120, 80)]),
    ];
    let mut output = tree(&input);

    output.remove_outermost_polygon();

    assert_eq!(
        snapshot_tree(&output),
        vec![
            NodeSnapshot {
                parent: None,
                sibling: 0,
                hole: false,
                contour: vec![(110, 10), (110, 90), (190, 90), (190, 10)],
            },
            NodeSnapshot {
                parent: Some(0),
                sibling: 0,
                hole: true,
                contour: vec![(180, 80), (120, 80), (120, 20), (180, 20)],
            },
            NodeSnapshot {
                parent: None,
                sibling: 1,
                hole: false,
                contour: vec![(10, 10), (10, 90), (90, 90), (90, 10)],
            },
            NodeSnapshot {
                parent: Some(2),
                sibling: 0,
                hole: true,
                contour: vec![(80, 80), (20, 80), (20, 20), (80, 20)],
            },
        ]
    );
    assert_eq!(output.total(), 4);
}

#[test]
fn task22g_polytree_outer_removal_clears_non_promotable_shapes() {
    let mut empty = PolyTree::empty();
    empty.remove_outermost_polygon();
    assert_eq!(empty.total(), 0);

    let mut single = tree(&[polygon(&[(0, 0), (20, 0), (20, 20), (0, 20)])]);
    single.remove_outermost_polygon();
    assert_eq!(single.total(), 0);

    let mut disjoint = tree(&[
        polygon(&[(0, 0), (20, 0), (20, 20), (0, 20)]),
        polygon(&[(40, 0), (60, 0), (60, 20), (40, 20)]),
    ]);
    disjoint.remove_outermost_polygon();
    assert_eq!(disjoint.total(), 0);
    assert_eq!(disjoint.children().count(), 0);
}

#[test]
fn task22g_wrapper_empty_shrink_short_circuits_and_generated_range_is_rejected() {
    assert_eq!(
        offset_paths(&[], -5.0, JoinType::Miter, 3.0),
        Ok(Vec::new())
    );

    const HIGH: i64 = 0x3fff_ffff_ffff_ffff;
    assert_eq!(
        execute(
            &[
                (HIGH - 16_384, 0),
                (HIGH - 8_192, 0),
                (HIGH - 8_192, 8_192),
                (HIGH - 16_384, 8_192),
            ],
            16_384.0,
        ),
        Err(ClipperError::CoordinateOutOfRange)
    );
}

#[test]
fn task22g_wrapper_signed_zero_and_sub_tolerance_follow_exact_branch_predicate() {
    let mixed_winding = [
        polygon(&[(0, 100), (40, 100), (40, 140), (0, 140)]),
        polygon(&[(100, 0), (100, 40), (140, 40), (140, 0)]),
    ];
    let run = |delta| {
        offset_paths(&mixed_winding, delta, JoinType::Miter, 3.0)
            .map(|paths| paths.iter().map(coordinates).collect::<Vec<_>>())
    };
    let shrunken = Ok(vec![vec![(40, 140), (0, 140), (0, 100), (40, 100)]]);

    assert_eq!(run(0.0), shrunken);
    assert_eq!(run(-0.0), shrunken);
    assert_eq!(run(-5.0e-21_f32), shrunken);
    assert_eq!(
        run(5.0e-21_f32),
        Ok(vec![
            vec![(40, 140), (0, 140), (0, 100), (40, 100)],
            vec![(140, 40), (100, 40), (100, 0), (140, 0)],
        ])
    );
}

#[test]
fn task22g_raw_wrapper_runs_internal_cleanup_and_preserves_round_order() {
    let square = [polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)])];

    assert_eq!(
        raw_offset_paths(&square, 5.0, JoinType::Miter, 3.0)
            .map(|paths| paths.iter().map(coordinates).collect::<Vec<_>>()),
        Ok(vec![vec![(105, 105), (-5, 105), (-5, -5), (105, -5)]])
    );
    assert_eq!(
        raw_offset_paths(&square, 10.0, JoinType::Round, 0.25)
            .map(|paths| paths.iter().map(coordinates).collect::<Vec<_>>()),
        Ok(vec![vec![
            (104, -9),
            (108, -6),
            (110, -2),
            (110, 100),
            (109, 104),
            (106, 108),
            (102, 110),
            (0, 110),
            (-4, 109),
            (-8, 106),
            (-10, 102),
            (-10, 0),
            (-9, -4),
            (-6, -8),
            (-2, -10),
            (100, -10),
        ]])
    );
}

#[test]
fn task22g_raw_wrapper_negates_cw_delta_then_reverses_each_result() {
    let outer = polygon(&[(200, 100), (200, 200), (100, 200), (100, 100)]);
    let hole = polygon(&[(160, 140), (140, 140), (140, 160), (160, 160)]);

    assert_eq!(
        raw_offset_paths(&[outer, hole], 5.0, JoinType::Miter, 3.0)
            .map(|paths| paths.iter().map(coordinates).collect::<Vec<_>>()),
        Ok(vec![
            vec![(205, 205), (95, 205), (95, 95), (205, 95)],
            vec![(155, 145), (145, 145), (145, 155), (155, 155)],
        ])
    );
}

#[test]
fn task22g_wrapper_polytree_is_single_pass_and_preserves_direct_root_order() {
    let input = vec![
        polygon(&[(0, 0), (60, 0), (60, 60), (0, 60)]),
        polygon(&[(10, 10), (10, 50), (50, 50), (50, 10)]),
        polygon(&[(20, 20), (40, 20), (40, 40), (20, 40)]),
        polygon(&[(100, 0), (160, 0), (160, 60), (100, 60)]),
        polygon(&[(110, 10), (110, 50), (150, 50), (150, 10)]),
    ];

    let output = offset_paths_tree(&input, 5.0e-21_f32, JoinType::Miter, 3.0).unwrap();

    assert_eq!(
        snapshot_tree(&output),
        vec![
            NodeSnapshot {
                parent: None,
                sibling: 0,
                hole: false,
                contour: vec![(160, 60), (100, 60), (100, 0), (160, 0)],
            },
            NodeSnapshot {
                parent: Some(0),
                sibling: 0,
                hole: true,
                contour: vec![(110, 10), (110, 50), (150, 50), (150, 10)],
            },
            NodeSnapshot {
                parent: None,
                sibling: 1,
                hole: false,
                contour: vec![(60, 60), (0, 60), (0, 0), (60, 0)],
            },
            NodeSnapshot {
                parent: Some(2),
                sibling: 0,
                hole: true,
                contour: vec![(10, 10), (10, 50), (50, 50), (50, 10)],
            },
            NodeSnapshot {
                parent: Some(3),
                sibling: 0,
                hole: false,
                contour: vec![(40, 40), (20, 40), (20, 20), (40, 20)],
            },
        ]
    );
}
