use super::apply_physical_map;
use crate::{OrcaInt, OrcaInts};

#[test]
fn physical_map_permutes_filament_slots_unconditionally() {
    // `GCode.cpp:2834-2842`: the remap applies regardless of toolhead
    // homogeneity — the old homogeneous skip was removed upstream.
    let swapped = OrcaInts(vec![OrcaInt(1), OrcaInt(0)]);
    assert_eq!(apply_physical_map(vec![0, -1], &swapped), vec![-1, 0]);

    let identity = OrcaInts(vec![OrcaInt(0), OrcaInt(1)]);
    assert_eq!(apply_physical_map(vec![0, -1], &identity), vec![0, -1]);
}
