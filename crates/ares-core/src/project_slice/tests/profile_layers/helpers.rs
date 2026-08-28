use std::fmt::Debug;

use crate::SliceError;
use crate::project_slice::{
    layers::{LayerPair, PlannedLayer},
    parameters::SlicingParameters,
};

pub(super) fn parameters(first: f64, regular: f64, minimum: f64, top: f64) -> SlicingParameters {
    SlicingParameters {
        base_raft_layers: 0,
        interface_raft_layers: 0,
        base_raft_layer_height: 0.0,
        interface_raft_layer_height: 0.0,
        contact_raft_layer_height: 0.0,
        layer_height: regular,
        min_layer_height: minimum,
        max_layer_height: regular.max(minimum),
        first_print_layer_height: first,
        first_object_layer_height: first,
        first_object_layer_bridging: false,
        gap_raft_object: 0.0,
        gap_object_support: 0.0,
        gap_support_object: 0.0,
        raft_base_top_z: 0.0,
        raft_interface_top_z: 0.0,
        raft_contact_top_z: 0.0,
        object_print_z_min: 0.0,
        object_print_z_max: top,
        object_print_z_uncompensated_max: top,
        object_shrinkage_compensation_z: 1.0,
    }
}

pub(super) fn expected_layers(pairs: &[LayerPair]) -> Vec<PlannedLayer> {
    pairs
        .iter()
        .enumerate()
        .map(|(id, pair)| PlannedLayer {
            id,
            height: pair.hi - pair.lo,
            print_z: pair.hi,
            slice_z: 0.5 * (pair.lo + pair.hi),
        })
        .collect()
}

pub(super) fn assert_invalid<T: Debug>(result: Result<T, SliceError>, expected: &str) {
    let SliceError::InvalidInput(message) = result.unwrap_err() else {
        panic!("expected InvalidInput");
    };
    assert_eq!(message, expected);
}
