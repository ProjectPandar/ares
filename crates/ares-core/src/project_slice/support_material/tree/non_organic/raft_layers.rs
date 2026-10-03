//! `TreeSupport.cpp:1157-1201::create_tree_support_layers`.

use super::drawing::areas::LayerAreas;
use crate::project_slice::parameters::SlicingParameters;

pub(super) fn create(slicing: &SlicingParameters) -> Vec<LayerAreas> {
    if slicing.base_raft_layers + slicing.interface_raft_layers == 0 {
        return Vec::new();
    }
    let mut layers = Vec::new();
    let mut print_z = 0.0;
    let heights = std::iter::once(slicing.first_print_layer_height)
        .chain((1..slicing.base_raft_layers).map(|_| slicing.base_raft_layer_height))
        .chain((0..slicing.interface_raft_layers).map(|_| slicing.interface_raft_layer_height));
    for height in heights {
        print_z += height;
        let mut layer = LayerAreas::default();
        layer.print_z = print_z;
        layer.height = height;
        layers.push(layer);
    }
    let distance = slicing.object_print_z_min - print_z;
    if distance > 0.0001 {
        let steps = ((distance - 0.0001) / slicing.max_support_layer_height).ceil() as usize;
        let height = distance / steps as f64;
        for _ in 0..steps {
            print_z += height;
            let mut layer = LayerAreas::default();
            layer.print_z = print_z;
            layer.height = height;
            layers.push(layer);
        }
    }
    layers
}
