use super::super::helpers::{polygon, polygons};
use super::simple_polygons::{NodeSnapshot, snapshot_tree, strict_tree};

#[test]
fn task22i_strict_simple_old_inside_new_repairs_dependent_parent() {
    let input = polygons(&[&[
        (10, 10),
        (20, 10),
        (15, 5),
        (20, 0),
        (10, 20),
        (15, 20),
        (20, 10),
        (10, 20),
        (10, 0),
        (10, 30),
        (20, 0),
        (25, 0),
        (15, 5),
    ]]);

    assert_eq!(
        snapshot_tree(&strict_tree(&input)),
        vec![
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: polygon(&[(18, 3), (20, 0), (15, 5)]),
            },
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: polygon(&[(20, 0), (25, 0), (19, 3)]),
            },
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: polygon(&[
                    (15, 20),
                    (13, 20),
                    (10, 30),
                    (10, 20),
                    (15, 10),
                    (10, 10),
                    (15, 5),
                    (17, 7),
                    (18, 5),
                    (18, 8),
                    (20, 10),
                ]),
            },
            NodeSnapshot {
                parent: Some(2),
                hole: true,
                contour: polygon(&[(17, 10), (15, 15), (20, 10)]),
            },
            NodeSnapshot {
                parent: None,
                hole: false,
                contour: polygon(&[(18, 5), (18, 3), (19, 3)]),
            },
        ]
    );
}

#[test]
fn task22i_strict_simple_keeps_split_hole_orientation() {
    let input = polygons(&[&[
        (5, 15),
        (30, 20),
        (25, 20),
        (30, 10),
        (0, 0),
        (15, 10),
        (25, 20),
        (10, 10),
        (10, 0),
        (15, 20),
        (30, 15),
        (20, 5),
    ]]);

    let snapshots = snapshot_tree(&strict_tree(&input));
    let parent_contour = polygon(&[(30, 20), (25, 20), (25, 19)]);
    let parent = snapshots
        .iter()
        .position(|node| node.parent.is_none() && node.contour == parent_contour)
        .expect("fixed Orca vector must preserve the split parent");
    let child = snapshots
        .iter()
        .find(|node| node.parent == Some(parent))
        .expect("fixed Orca vector must preserve the split child");

    assert_eq!(
        child,
        &NodeSnapshot {
            parent: Some(parent),
            hole: true,
            contour: polygon(&[(24, 19), (25, 20), (23, 19)]),
        }
    );
}
