//! Organic's raft-outline Z prefix (`TreeSupportCommon.hpp:329-357`).

use crate::project_slice::parameters::SlicingParameters;

const EPSILON: f64 = 1e-4;

pub(super) fn layer_z(slicing: &SlicingParameters) -> Vec<f64> {
    let count = slicing.base_raft_layers + slicing.interface_raft_layers;
    if count == 0 {
        return Vec::new();
    }
    let mut z = slicing.first_print_layer_height;
    let mut layers = vec![z];
    for _ in 1..slicing.base_raft_layers {
        z += slicing.base_raft_layer_height;
        layers.push(z);
    }
    for _ in 0..slicing.interface_raft_layers.saturating_sub(1) {
        z += slicing.interface_raft_layer_height;
        layers.push(z);
    }
    if count > 1 {
        z = slicing.raft_contact_top_z;
        layers.push(z);
    }
    let distance = slicing.object_print_z_min - z;
    if distance > EPSILON {
        let steps = ((distance - EPSILON) / slicing.max_support_layer_height).ceil() as i32;
        let step = distance / f64::from(steps);
        for _ in 0..steps {
            z += step;
            layers.push(z);
        }
    }
    layers
}
