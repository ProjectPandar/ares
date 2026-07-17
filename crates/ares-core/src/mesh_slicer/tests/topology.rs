use crate::SliceError;

use super::super::{
    index_mesh_edges,
    topology::{checked_edge_id, sorted_edge_uses},
};

#[test]
fn task22b_topology_pairs_opposite_oriented_neighbors() {
    let topology = index_mesh_edges(&[[2, 5, 0], [5, 2, 3]]).unwrap();

    assert_eq!(topology.edge_count(), 5);
    assert_eq!(topology.face_edge_ids(), &[[3, 1, 0], [3, 2, 4]]);
}

#[test]
fn task22b_topology_assigns_one_id_to_a_boundary_edge() {
    let topology = index_mesh_edges(&[[4, 1, 9]]).unwrap();

    assert_eq!(topology.edge_count(), 3);
    assert_eq!(topology.face_edge_ids(), &[[0, 1, 2]]);
}

#[test]
fn task22b_topology_pairs_two_same_oriented_uses() {
    let topology = index_mesh_edges(&[[2, 5, 0], [2, 5, 3]]).unwrap();

    assert_eq!(topology.edge_count(), 5);
    assert_eq!(topology.face_edge_ids(), &[[3, 1, 0], [3, 4, 2]]);
}

#[test]
fn task22b_topology_rejects_more_than_two_uses_before_intersection() {
    let expected = || SliceError::UnsupportedProjectFeature("mesh_topology".to_owned());

    assert_eq!(
        index_mesh_edges(&[[0, 1, 2], [1, 0, 3], [0, 1, 4]]).unwrap_err(),
        expected()
    );
    assert_eq!(index_mesh_edges(&[[0, 0, 0]]).unwrap_err(), expected());
}

#[test]
fn task22b_topology_edge_id_range_is_checked_without_large_allocation() {
    assert_eq!(checked_edge_id(u64::from(u32::MAX)), Ok(u32::MAX));
    assert_eq!(
        checked_edge_id(u64::from(u32::MAX) + 1),
        Err(SliceError::InvalidInput(
            "project mesh edge count exceeds supported range".to_owned()
        ))
    );
}

#[test]
fn task22b_topology_indexing_is_deterministic() {
    let triangles = [[3, 0, 1], [1, 3, 2]];
    let first = index_mesh_edges(&triangles).unwrap();
    let second = index_mesh_edges(&triangles).unwrap();
    let uses = sorted_edge_uses(&triangles);

    assert_eq!(first, second);
    assert_eq!(first.face_edge_ids(), &[[1, 0, 3], [3, 4, 2]]);
    assert_eq!(
        uses.iter()
            .map(|edge_use| {
                (
                    edge_use.low,
                    edge_use.high,
                    edge_use.face,
                    edge_use.local_edge,
                    edge_use.reversed,
                )
            })
            .collect::<Vec<_>>(),
        [
            (0, 1, 0, 1, false),
            (0, 3, 0, 0, true),
            (1, 2, 1, 2, true),
            (1, 3, 0, 2, false),
            (1, 3, 1, 0, false),
            (2, 3, 1, 1, true),
        ]
    );
}
