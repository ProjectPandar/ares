use super::LayerTools;
use crate::project_slice::{
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

pub(super) fn layer_index(layers: &[LayerTools], z: f64) -> usize {
    let mut index = layers.partition_point(|layer| layer.print_z < z - 1e-4);
    let mut distance = (layers[index].print_z - z).abs();
    while index + 1 < layers.len() {
        let next = (layers[index + 1].print_z - z).abs();
        if next >= distance {
            break;
        }
        index += 1;
        distance = next;
    }
    index
}

pub(super) fn initialize_layers(
    traversal: &PreparedPostClassicTraversal,
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&[Vec<crate::project_slice::raft::RaftLayerPlan>]>,
) -> Vec<LayerTools> {
    let mut zs = traversal
        .objects
        .iter()
        .flat_map(|object| object.physical_layers().iter().map(|layer| layer.print_z))
        .chain(support.iter().flatten().map(|layer| layer.print_z))
        .collect::<Vec<_>>();
    if let Some(plans) = raft {
        zs.extend(plans.iter().flatten().map(|plan| plan.z.print_z));
    }
    initialize_layers_on_zs(zs)
}

/// `ToolOrdering.cpp:406-414`: the per-object ordering initializes the
/// layer tools from that object's own object/support/raft z heights only.
pub(super) fn initialize_layers_scoped(
    traversal: &PreparedPostClassicTraversal,
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&[Vec<crate::project_slice::raft::RaftLayerPlan>]>,
    object: usize,
) -> Vec<LayerTools> {
    let mut zs = traversal.objects[object]
        .physical_layers()
        .iter()
        .map(|layer| layer.print_z)
        .chain(support[object].iter().map(|layer| layer.print_z))
        .collect::<Vec<_>>();
    if let Some(plans) = raft {
        zs.extend(plans[object].iter().map(|plan| plan.z.print_z));
    }
    initialize_layers_on_zs(zs)
}

fn initialize_layers_on_zs(mut zs: Vec<f64>) -> Vec<LayerTools> {
    zs.sort_by(f64::total_cmp);
    zs.dedup();
    let mut layers = Vec::new();
    let mut begin = 0;
    while begin < zs.len() {
        let mut end = begin + 1;
        while end < zs.len() && zs[end] <= zs[begin] + 1e-4 {
            end += 1;
        }
        layers.push(LayerTools {
            print_z: 0.5 * (zs[begin] + zs[end - 1]),
            ..Default::default()
        });
        begin = end;
    }
    layers
}
