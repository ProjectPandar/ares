use crate::geometry::clipper::z::{KernelPoint, ZPath};
use crate::geometry::region_expansion::{
    merge_path_for_test, reconcile_for_test, split_registry_for_test,
};

fn zpath(values: &[(i64, i64, i64)]) -> ZPath {
    values
        .iter()
        .map(|&(x, y, z)| KernelPoint::new(x, y, z))
        .collect()
}

#[test]
fn split_reconciliation_keeps_duplicate_junction_and_swap_pop_order() {
    let split = KernelPoint::new(0, 0, 7);
    let source = vec![vec![split, KernelPoint::new(5, 5, 7), split]];
    let mut fragments = vec![
        vec![split, KernelPoint::new(1, 0, -1)],
        vec![KernelPoint::new(-1, 0, -2), split],
        vec![KernelPoint::new(9, 9, 7), KernelPoint::new(10, 9, 7)],
    ];
    reconcile_for_test(&source, &mut fragments);
    assert_eq!(fragments.len(), 2);
    assert_eq!(
        xyz(&fragments[0]),
        vec![(-1, 0, -2), (0, 0, 7), (0, 0, 7), (1, 0, -1)]
    );
    assert_eq!(xyz(&fragments[1]), vec![(9, 9, 7), (10, 9, 7)]);
}

fn xyz(path: &[KernelPoint]) -> Vec<(i64, i64, i64)> {
    path.iter()
        .map(|point| (point.x(), point.y(), point.z))
        .collect()
}

#[test]
fn four_merge_directions_retain_duplicate_junction() {
    let destination = zpath(&[(0, 0, 1), (1, 0, 1)]);
    let source = zpath(&[(1, 0, 1), (2, 0, 1)]);
    let cases = [
        (
            false,
            true,
            vec![(0, 0, 1), (1, 0, 1), (1, 0, 1), (2, 0, 1)],
        ),
        (
            false,
            false,
            vec![(0, 0, 1), (1, 0, 1), (2, 0, 1), (1, 0, 1)],
        ),
        (true, true, vec![(1, 0, 1), (0, 0, 1), (1, 0, 1), (2, 0, 1)]),
        (
            true,
            false,
            vec![(1, 0, 1), (2, 0, 1), (0, 0, 1), (1, 0, 1)],
        ),
    ];
    for (destination_front, source_front, expected) in cases {
        let mut actual = destination.clone();
        merge_path_for_test(&mut actual, destination_front, source.clone(), source_front);
        assert_eq!(xyz(&actual), expected);
    }
}

#[test]
fn split_registry_uses_xyz_lower_bound_and_literal_order() {
    let paths = vec![
        zpath(&[(0, 0, 9), (1, 0, 9), (0, 0, 9)]),
        zpath(&[(0, 0, 2), (2, 0, 2), (0, 0, 2)]),
        zpath(&[(-1, 0, 7), (3, 0, 7), (-1, 0, 7)]),
    ];
    let registry = split_registry_for_test(&paths);
    assert_eq!(
        registry
            .iter()
            .map(|(point, owner)| (point.x(), point.y(), point.z, *owner))
            .collect::<Vec<_>>(),
        vec![(-1, 0, 7, None), (0, 0, 2, None), (0, 0, 9, None)]
    );
}

#[test]
fn front_split_precedes_matching_back_split() {
    let source = vec![
        zpath(&[(0, 0, 1), (5, 5, 1), (0, 0, 1)]),
        zpath(&[(10, 0, 2), (5, 5, 2), (10, 0, 2)]),
    ];
    let mut fragments = vec![
        zpath(&[(0, 0, 1), (4, 0, 1)]),
        zpath(&[(0, 0, 1), (10, 0, 2)]),
        zpath(&[(10, 0, 2), (11, 0, 2)]),
    ];
    reconcile_for_test(&source, &mut fragments);
    assert_eq!(fragments.len(), 2);
    assert_eq!(
        xyz(&fragments[0]),
        vec![(4, 0, 1), (0, 0, 1), (0, 0, 1), (10, 0, 2)]
    );
    assert_eq!(xyz(&fragments[1]), vec![(10, 0, 2), (11, 0, 2)]);
}

#[test]
fn closed_fragments_are_noop_and_keep_exact_order() {
    let source = vec![zpath(&[(0, 0, 3), (5, 5, 3), (0, 0, 3)])];
    let closed = zpath(&[(0, 0, 3), (2, 0, 3), (0, 0, 3)]);
    let open = zpath(&[(9, 9, 4), (10, 9, 4)]);
    let mut fragments = vec![closed.clone(), open.clone()];
    reconcile_for_test(&source, &mut fragments);
    assert_eq!(fragments, vec![closed, open]);
}

#[test]
fn moved_middle_fragment_is_reprocessed_and_merged() {
    let source = vec![
        zpath(&[(0, 0, 1), (5, 5, 1), (0, 0, 1)]),
        zpath(&[(10, 0, 2), (15, 5, 2), (10, 0, 2)]),
    ];
    let mut fragments = vec![
        zpath(&[(0, 0, 1), (1, 0, 1)]),
        zpath(&[(2, 0, 1), (0, 0, 1)]),
        zpath(&[(9, 0, 2), (10, 0, 2)]),
        zpath(&[(10, 0, 2), (11, 0, 2)]),
    ];
    reconcile_for_test(&source, &mut fragments);
    assert_eq!(fragments.len(), 2);
    assert_eq!(
        xyz(&fragments[0]),
        vec![(2, 0, 1), (0, 0, 1), (0, 0, 1), (1, 0, 1)]
    );
    assert_eq!(
        xyz(&fragments[1]),
        vec![(9, 0, 2), (10, 0, 2), (10, 0, 2), (11, 0, 2)]
    );
}
