//! Active layer id and flush flag of `GCode.cpp::process_layer` LayerResult.

use super::schedule::LayerToPrint;
use crate::project_slice::{
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

pub(super) fn metadata(
    entries: &[LayerToPrint],
    traversal: &PreparedPostClassicTraversal,
    support: &[Vec<InstalledSupportLayer>],
    last_layer: bool,
) -> (usize, bool) {
    let object = entries.iter().find(|entry| entry.object_layer.is_some());
    let first_support = entries
        .iter()
        .find(|entry| entry.support_layer.is_some() || entry.raft_layer.is_some());
    let support_id = first_support.map(|entry| {
        entry
            .raft_layer
            .unwrap_or_else(|| support[entry.object_index][entry.support_layer.unwrap()].id)
    });
    let raft = first_support.is_some_and(|entry| {
        let parameters = traversal.objects[entry.object_index].slicing_parameters();
        support_id.unwrap() < parameters.base_raft_layers + parameters.interface_raft_layers
    });
    let layer_id = object.map_or_else(
        || support_id.expect("layer chunk has an active object or support layer"),
        |entry| {
            traversal.objects[entry.object_index].physical_layers()[entry.object_layer.unwrap()].id
        },
    );
    (layer_id, object.is_some() || raft || last_layer)
}
