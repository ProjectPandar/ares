//! `TreeSupportCommon.hpp::layer_idx_ceil/floor`, shared by smoothing/drawing.

use super::TreeSettings;
use crate::project_slice::parameters::SlicingParameters;

pub(super) fn ceil(slicing: &SlicingParameters, config: &TreeSettings, z: f64) -> usize {
    let first = slicing.object_print_z_min + slicing.first_object_layer_height;
    if !config.raft_layers.is_empty() && z < first - 1e-4 {
        config
            .raft_layers
            .partition_point(|&height| height < z - 1e-4)
    } else {
        config.raft_layers.len()
            + (((z - first) / slicing.layer_height).ceil() as i32).max(0) as usize
    }
}

pub(super) fn floor(slicing: &SlicingParameters, config: &TreeSettings, z: f64) -> usize {
    let first = slicing.object_print_z_min + slicing.first_object_layer_height;
    if !config.raft_layers.is_empty() && z < first - 1e-4 {
        let end = config
            .raft_layers
            .partition_point(|&height| height <= z + 1e-4);
        if end == 0 { 0 } else { end - 1 }
    } else {
        config.raft_layers.len()
            + (((z - first) / slicing.layer_height).floor() as i32).max(0) as usize
    }
}
