use super::helpers::polygon;
use crate::geometry::clipper::{
    ClipOperation, ClipperOptions, ClosedClipper, FillRule, PathRole, PolyNode, PolyTree, union_ex,
};
use crate::geometry::{ExPolygon, Polygon};

#[derive(Debug, Eq, PartialEq)]
struct NodeSnapshot {
    parent: Option<usize>,
    sibling: usize,
    hole: bool,
    contour: Polygon,
}

fn nested_input() -> Vec<Polygon> {
    vec![
        polygon(&[(0, 0), (100, 0), (100, 100), (0, 100)]),
        polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)]),
        polygon(&[(20, 20), (80, 20), (80, 80), (20, 80)]),
        polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]),
    ]
}

fn sibling_input() -> Vec<Polygon> {
    vec![
        polygon(&[(0, 0), (60, 0), (60, 60), (0, 60)]),
        polygon(&[(10, 10), (10, 50), (50, 50), (50, 10)]),
        polygon(&[(20, 20), (40, 20), (40, 40), (20, 40)]),
        polygon(&[(100, 0), (160, 0), (160, 60), (100, 60)]),
        polygon(&[(110, 10), (110, 50), (150, 50), (150, 10)]),
    ]
}

fn multi_hole_input() -> Vec<Polygon> {
    vec![
        polygon(&[(0, 0), (200, 0), (200, 200), (0, 200)]),
        polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)]),
        polygon(&[(20, 20), (80, 20), (80, 80), (20, 80)]),
        polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]),
        polygon(&[(110, 10), (110, 90), (190, 90), (190, 10)]),
        polygon(&[(120, 20), (180, 20), (180, 80), (120, 80)]),
        polygon(&[(300, 0), (360, 0), (360, 60), (300, 60)]),
    ]
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
            contour: node.contour().clone(),
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

fn execute_tree(input: &[Polygon]) -> PolyTree {
    execute_tree_with_fill(input, FillRule::NonZero)
}

fn execute_tree_with_fill(input: &[Polygon], fill: FillRule) -> PolyTree {
    let mut clipper = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(clipper.add_closed_paths(input, PathRole::Subject), Ok(true));
    clipper.execute_polytree(ClipOperation::Union, fill, fill)
}

#[test]
fn task22f_polytree_freezes_nested_parent_child_sibling_and_start_point_order() {
    let input = nested_input();
    let mut clipper = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(
        clipper.add_closed_paths(&input, PathRole::Subject),
        Ok(true)
    );

    let tree = clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);

    assert_eq!(
        snapshot_tree(&tree),
        vec![
            NodeSnapshot {
                parent: None,
                sibling: 0,
                hole: false,
                contour: polygon(&[(100, 100), (0, 100), (0, 0), (100, 0)]),
            },
            NodeSnapshot {
                parent: Some(0),
                sibling: 0,
                hole: true,
                contour: polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)]),
            },
            NodeSnapshot {
                parent: Some(1),
                sibling: 0,
                hole: false,
                contour: polygon(&[(80, 80), (20, 80), (20, 20), (80, 20)]),
            },
            NodeSnapshot {
                parent: Some(2),
                sibling: 0,
                hole: true,
                contour: polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]),
            },
        ]
    );
    assert!(
        clipper
            .execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero,)
            .children()
            .next()
            .is_none()
    );

    clipper.clear();
    assert_eq!(
        clipper.add_closed_paths(&input, PathRole::Subject),
        Ok(true)
    );
    assert_eq!(
        clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero,),
        tree
    );
}

#[test]
fn task22f_union_ex_uses_paths_then_fresh_polytree_and_exact_recursive_order() {
    let input = sibling_input();
    let direct = snapshot_tree(&execute_tree(&input));
    assert_eq!(
        direct,
        vec![
            NodeSnapshot {
                parent: None,
                sibling: 0,
                hole: false,
                contour: polygon(&[(160, 60), (100, 60), (100, 0), (160, 0)]),
            },
            NodeSnapshot {
                parent: Some(0),
                sibling: 0,
                hole: true,
                contour: polygon(&[(110, 10), (110, 50), (150, 50), (150, 10)]),
            },
            NodeSnapshot {
                parent: None,
                sibling: 1,
                hole: false,
                contour: polygon(&[(60, 60), (0, 60), (0, 0), (60, 0)]),
            },
            NodeSnapshot {
                parent: Some(2),
                sibling: 0,
                hole: true,
                contour: polygon(&[(10, 10), (10, 50), (50, 50), (50, 10)]),
            },
            NodeSnapshot {
                parent: Some(3),
                sibling: 0,
                hole: false,
                contour: polygon(&[(40, 40), (20, 40), (20, 20), (40, 20)]),
            },
        ]
    );

    let expected = vec![
        ExPolygon::new(
            polygon(&[(60, 60), (0, 60), (0, 0), (60, 0)]),
            vec![polygon(&[(10, 10), (10, 50), (50, 50), (50, 10)])],
        ),
        ExPolygon::new(
            polygon(&[(40, 40), (20, 40), (20, 20), (40, 20)]),
            Vec::new(),
        ),
        ExPolygon::new(
            polygon(&[(160, 60), (100, 60), (100, 0), (160, 0)]),
            vec![polygon(&[(110, 10), (110, 50), (150, 50), (150, 10)])],
        ),
    ];

    assert_eq!(union_ex(&input, FillRule::NonZero), Ok(expected.clone()));
    assert_eq!(union_ex(&input, FillRule::NonZero), Ok(expected));
}

#[test]
fn task22f_polytree_freezes_multiple_hole_and_island_sibling_topology() {
    assert_eq!(
        snapshot_tree(&execute_tree(&multi_hole_input())),
        vec![
            NodeSnapshot {
                parent: None,
                sibling: 0,
                hole: false,
                contour: polygon(&[(200, 200), (0, 200), (0, 0), (200, 0)]),
            },
            NodeSnapshot {
                parent: Some(0),
                sibling: 0,
                hole: true,
                contour: polygon(&[(110, 10), (110, 90), (190, 90), (190, 10)]),
            },
            NodeSnapshot {
                parent: Some(1),
                sibling: 0,
                hole: false,
                contour: polygon(&[(180, 80), (120, 80), (120, 20), (180, 20)]),
            },
            NodeSnapshot {
                parent: Some(0),
                sibling: 1,
                hole: true,
                contour: polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)]),
            },
            NodeSnapshot {
                parent: Some(3),
                sibling: 0,
                hole: false,
                contour: polygon(&[(80, 80), (20, 80), (20, 20), (80, 20)]),
            },
            NodeSnapshot {
                parent: Some(4),
                sibling: 0,
                hole: true,
                contour: polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)]),
            },
            NodeSnapshot {
                parent: None,
                sibling: 1,
                hole: false,
                contour: polygon(&[(360, 60), (300, 60), (300, 0), (360, 0)]),
            },
        ]
    );
}

#[test]
fn task22f_union_ex_preserves_hole_siblings_and_recursive_island_order() {
    let input = multi_hole_input();
    let expected = vec![
        ExPolygon::new(
            polygon(&[(200, 200), (0, 200), (0, 0), (200, 0)]),
            vec![
                polygon(&[(10, 10), (10, 90), (90, 90), (90, 10)]),
                polygon(&[(110, 10), (110, 90), (190, 90), (190, 10)]),
            ],
        ),
        ExPolygon::new(
            polygon(&[(80, 80), (20, 80), (20, 20), (80, 20)]),
            vec![polygon(&[(30, 30), (30, 70), (70, 70), (70, 30)])],
        ),
        ExPolygon::new(
            polygon(&[(180, 80), (120, 80), (120, 20), (180, 20)]),
            Vec::new(),
        ),
        ExPolygon::new(
            polygon(&[(360, 60), (300, 60), (300, 0), (360, 0)]),
            Vec::new(),
        ),
    ];

    assert_eq!(union_ex(&input, FillRule::NonZero), Ok(expected));
}

#[test]
fn task22f_union_ex_forwards_negative_fill_to_the_fresh_second_pass() {
    let input = vec![polygon(&[(0, 0), (0, 40), (40, 40), (40, 0)])];
    let direct = execute_tree_with_fill(&input, FillRule::Negative);
    assert_eq!(direct.total(), 1);
    assert_eq!(
        direct.children().next().expect("one direct root").contour(),
        &polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)])
    );

    assert_eq!(union_ex(&input, FillRule::Negative), Ok(Vec::new()));
}

#[test]
fn task22f_polytree_and_union_ex_empty_input_are_empty() {
    let mut clipper = ClosedClipper::new(ClipperOptions::default());
    assert_eq!(clipper.add_closed_paths(&[], PathRole::Subject), Ok(false));
    let tree = clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    assert_eq!(tree.total(), 0);
    assert!(tree.children().next().is_none());
    assert_eq!(union_ex(&[], FillRule::NonZero), Ok(Vec::new()));
}
