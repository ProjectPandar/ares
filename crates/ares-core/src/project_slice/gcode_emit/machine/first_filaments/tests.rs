use super::apply_physical_map;
use crate::{OrcaInt, OrcaInts};

#[test]
fn homogeneous_hotends_keep_logical_order() {
    let physical_map = OrcaInts(vec![OrcaInt(1), OrcaInt(0)]);

    assert_eq!(
        apply_physical_map(vec![0, -1], &physical_map, false),
        vec![0, -1]
    );
}

#[test]
fn heterogeneous_hotends_apply_physical_permutation() {
    let physical_map = OrcaInts(vec![OrcaInt(1), OrcaInt(0)]);

    assert_eq!(
        apply_physical_map(vec![0, -1], &physical_map, true),
        vec![-1, 0]
    );
}
