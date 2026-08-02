use super::super::helpers::{execute, polygon, polygons};
use crate::geometry::Polygon;
use crate::geometry::clipper::{
    ClipOperation, Clipper, ClipperError, ClipperOptions, FillRule, PathRole, PolyNode, PolyTree,
    SimpleRepair, simplify_polygons,
};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct NodeSnapshot {
    pub(super) parent: Option<usize>,
    pub(super) hole: bool,
    pub(super) contour: Polygon,
}

fn strict_clipper(input: &[Polygon]) -> Clipper {
    let mut clipper = Clipper::new(ClipperOptions {
        strictly_simple: true,
        ..ClipperOptions::default()
    });
    assert_eq!(clipper.add_closed_paths(input, PathRole::Subject), Ok(true));
    clipper
}

pub(super) fn strict_tree(input: &[Polygon]) -> PolyTree {
    let mut clipper = strict_clipper(input);
    clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
}

fn strict_repair_traces(input: &[Polygon]) -> (Vec<SimpleRepair>, Vec<SimpleRepair>) {
    let mut paths_clipper = strict_clipper(input);
    paths_clipper
        .execute_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
        .expect("closed Clipper execution accepts flat output");
    let paths = paths_clipper.simple_repairs_for_test().to_vec();

    let mut tree_clipper = strict_clipper(input);
    tree_clipper.execute_polytree(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    let tree = tree_clipper.simple_repairs_for_test().to_vec();
    (paths, tree)
}

pub(super) fn snapshot_tree(tree: &PolyTree) -> Vec<NodeSnapshot> {
    fn visit(node: PolyNode<'_>, parent: Option<usize>, result: &mut Vec<NodeSnapshot>) {
        let index = result.len();
        result.push(NodeSnapshot {
            parent,
            hole: node.is_hole(),
            contour: node.contour().clone(),
        });
        for child in node.children() {
            visit(child, Some(index), result);
        }
    }

    let mut result = Vec::with_capacity(tree.total());
    for root in tree.children() {
        visit(root, None, &mut result);
    }
    result
}

#[test]
fn task22i_strict_simple_paths_split_the_fixed_touching_kat() {
    let input = polygons(&[&[
        (0, 0),
        (10, 0),
        (10, 10),
        (20, 10),
        (20, 20),
        (10, 20),
        (10, 10),
        (0, 10),
    ]]);

    let non_strict = execute(
        input.clone(),
        Vec::new(),
        ClipOperation::Union,
        (FillRule::NonZero, FillRule::NonZero),
        ClipperOptions::default(),
    );
    assert_eq!(
        non_strict,
        polygons(&[&[
            (10, 10),
            (20, 10),
            (20, 20),
            (10, 20),
            (10, 10),
            (0, 10),
            (0, 0),
            (10, 0),
        ]])
    );

    assert_eq!(
        simplify_polygons(&input).expect("fixed KAT coordinates are in range"),
        polygons(&[
            &[(20, 10), (20, 20), (10, 20), (10, 10)],
            &[(0, 10), (0, 0), (10, 0), (10, 10)],
        ])
    );
}

#[test]
fn task22i_strict_simple_disjoint_split_repairs_dependent_hole() {
    let input = polygons(&[
        &[
            (0, 0),
            (10, 0),
            (10, 10),
            (20, 10),
            (20, 20),
            (10, 20),
            (10, 10),
            (0, 10),
        ],
        &[(2, 2), (2, 8), (8, 8), (8, 2)],
    ]);
    assert_eq!(
        strict_repair_traces(&input),
        (Vec::new(), vec![SimpleRepair::FirstLefts1])
    );

    assert_eq!(
        simplify_polygons(&input).expect("fixed disjoint coordinates are in range"),
        polygons(&[
            &[(20, 10), (20, 20), (10, 20), (10, 10)],
            &[(2, 2), (2, 8), (8, 8), (8, 2)],
            &[(0, 10), (0, 0), (10, 0), (10, 10)],
        ])
    );
    assert_eq!(
        snapshot_tree(&strict_tree(&input)),
        vec![
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: polygon(&[(20, 10), (20, 20), (10, 20), (10, 10)]),
            },
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: polygon(&[(0, 10), (0, 0), (10, 0), (10, 10)]),
            },
            NodeSnapshot {
                parent: Some(1),
                hole: true,
                contour: polygon(&[(2, 2), (2, 8), (8, 8), (8, 2)]),
            },
        ]
    );
}

#[test]
fn task22i_strict_simple_new_inside_old_repairs_nested_island_parent() {
    let input = polygons(&[
        &[
            (100, 0),
            (100, 100),
            (0, 100),
            (0, 0),
            (100, 0),
            (20, 20),
            (20, 80),
            (80, 80),
            (80, 20),
        ],
        &[(40, 40), (60, 40), (60, 60), (40, 60)],
    ]);
    assert_eq!(
        strict_repair_traces(&input),
        (Vec::new(), vec![SimpleRepair::FirstLefts2])
    );
    let outer = polygon(&[(100, 100), (0, 100), (0, 0), (100, 0)]);
    let island = polygon(&[(60, 60), (40, 60), (40, 40), (60, 40)]);
    let hole = polygon(&[(20, 20), (20, 80), (80, 80), (80, 20), (100, 0)]);

    assert_eq!(
        simplify_polygons(&input).expect("fixed contained coordinates are in range"),
        vec![outer.clone(), island.clone(), hole.clone()]
    );
    assert_eq!(
        snapshot_tree(&strict_tree(&input)),
        vec![
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: outer,
            },
            NodeSnapshot {
                parent: Some(0),
                hole: true,
                contour: hole,
            },
            NodeSnapshot {
                parent: Some(1),
                hole: false,
                contour: island,
            },
        ]
    );
}

#[test]
fn task22i_strict_simple_old_inside_new_transfers_parent_to_appended_outer() {
    let input = polygons(&[
        &[
            (0, 0),
            (100, 0),
            (100, 100),
            (0, 100),
            (0, 0),
            (20, 20),
            (20, 80),
            (80, 80),
            (80, 20),
        ],
        &[(40, 40), (60, 40), (60, 60), (40, 60)],
    ]);
    assert_eq!(
        strict_repair_traces(&input),
        (Vec::new(), vec![SimpleRepair::FirstLefts2])
    );
    let hole = polygon(&[(20, 20), (20, 80), (80, 80), (80, 20), (0, 0)]);
    let island = polygon(&[(60, 60), (40, 60), (40, 40), (60, 40)]);
    let outer = polygon(&[(100, 0), (100, 100), (0, 100), (0, 0)]);

    assert_eq!(
        simplify_polygons(&input).expect("fixed contained coordinates are in range"),
        vec![hole.clone(), island.clone(), outer.clone()]
    );
    assert_eq!(
        snapshot_tree(&strict_tree(&input)),
        vec![
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: outer,
            },
            NodeSnapshot {
                parent: Some(0),
                hole: true,
                contour: hole,
            },
            NodeSnapshot {
                parent: Some(1),
                hole: false,
                contour: island,
            },
        ]
    );
}

#[test]
fn task22i_strict_simple_repeated_duplicates_preserve_discovery_order() {
    let input = polygons(&[&[
        (0, 0),
        (10, 0),
        (10, 10),
        (20, 10),
        (20, 20),
        (30, 20),
        (30, 30),
        (20, 30),
        (20, 20),
        (10, 20),
        (10, 10),
        (0, 10),
    ]]);

    assert_eq!(
        simplify_polygons(&input).expect("fixed repeated coordinates are in range"),
        polygons(&[
            &[(30, 20), (30, 30), (20, 30), (20, 20)],
            &[(0, 10), (0, 0), (10, 0), (10, 10)],
            &[(10, 20), (10, 10), (20, 10), (20, 20)],
        ])
    );
}

#[test]
fn task22i_strict_simple_visits_a_newly_appended_record() {
    let input = polygons(&[&[
        (10, 0),
        (25, 10),
        (15, 5),
        (30, 0),
        (10, 5),
        (15, 0),
        (30, 20),
        (5, 0),
        (30, 20),
        (10, 5),
        (25, 10),
        (10, 15),
    ]]);

    assert_eq!(
        simplify_polygons(&input).expect("fixed appended-record coordinates are in range"),
        polygons(&[
            &[(15, 4), (17, 4), (18, 4), (17, 3)],
            &[(21, 8), (22, 9), (25, 10)],
            &[
                (23, 11),
                (30, 20),
                (19, 12),
                (10, 15),
                (10, 0),
                (13, 2),
                (15, 0),
                (17, 3),
                (30, 0),
                (18, 4),
                (20, 7),
                (25, 10),
            ],
        ])
    );
}

#[test]
fn task22i_strict_simple_keeps_two_point_paths_without_refixup() {
    let input = polygons(&[&[
        (10, 25),
        (5, 15),
        (0, 5),
        (30, 30),
        (25, 20),
        (5, 15),
        (0, 5),
        (20, 5),
    ]]);
    let first = polygon(&[(25, 20), (30, 30), (15, 18)]);
    let second = polygon(&[(10, 25), (0, 5), (20, 5), (14, 17)]);

    assert_eq!(
        simplify_polygons(&input).expect("fixed two-point coordinates are in range"),
        vec![
            first.clone(),
            second.clone(),
            polygon(&[(14, 17), (15, 18)]),
        ]
    );
    assert_eq!(
        snapshot_tree(&strict_tree(&input)),
        vec![
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: first,
            },
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: second,
            },
        ]
    );
}

#[test]
fn task22i_strict_simple_wrapper_uses_nonzero_and_propagates_input_contract() {
    let square = &[(0, 0), (10, 0), (10, 10), (0, 10)];
    assert_eq!(
        simplify_polygons(&polygons(&[square, square])),
        Ok(polygons(&[&[(10, 10), (0, 10), (0, 0), (10, 0)]]))
    );
    assert_eq!(simplify_polygons(&[]), Ok(Vec::new()));
    assert_eq!(
        simplify_polygons(&[polygon(&[]), polygon(&[(0, 0), (1, 1)])]),
        Ok(Vec::new())
    );
    assert_eq!(
        simplify_polygons(&[polygon(&[(i64::MAX, 0), (0, 1), (0, 0)])]),
        Err(ClipperError::CoordinateOutOfRange)
    );
}
