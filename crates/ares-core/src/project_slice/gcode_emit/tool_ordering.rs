//! `ToolOrdering.cpp` materialized layer tools and writer-used IDs.

mod collect;
mod first_layer;
mod reorder;

use super::layers::RaftSchedule;

use crate::project_slice::{
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

pub(super) struct LayerTools {
    print_z: f64,
    extruders: Vec<usize>,
    has_object: bool,
}

pub(super) struct ToolOrdering {
    layers: Vec<LayerTools>,
    all_extruders: Vec<usize>,
}

impl ToolOrdering {
    pub(super) fn all_extruders(&self) -> &[usize] {
        &self.all_extruders
    }

    pub(super) fn ordered_tools(&self) -> impl Iterator<Item = usize> + '_ {
        self.layers
            .iter()
            .flat_map(|layer| layer.extruders.iter().copied())
    }

    pub(super) fn tools_for_layer(&self, print_z: f64) -> &[usize] {
        let index = layer_index(&self.layers, print_z);
        &self.layers[index].extruders
    }
}

pub(super) fn build(
    traversal: &PreparedPostClassicTraversal,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&RaftSchedule>,
) -> Result<ToolOrdering, crate::SliceError> {
    let mut layers = initialize_layers(traversal, support, raft.map(|raft| raft.plans.as_slice()));
    let candidates = collect::print_extruders(traversal);
    for object in 0..objects.len() {
        let options = collect::object_options(traversal, object);
        for (physical, layer) in traversal.objects[object]
            .physical_layers()
            .iter()
            .zip(&objects[object])
        {
            let target = layer_index(&layers, physical.print_z);
            collect::object(&mut layers[target], layer, traversal, object);
        }
        for layer in &support[object] {
            let target = layer_index(&layers, layer.print_z);
            collect::support(
                &mut layers[target],
                layer.support_fills.iter().map(|entity| entity.role()),
                options,
                traversal,
                &candidates,
            );
        }
        if let Some(raft) = raft {
            for (index, layer) in raft.layers[object].iter().enumerate() {
                let target = layer_index(&layers, raft.plans[object][index].z.print_z);
                collect::support(
                    &mut layers[target],
                    collect::raft_roles(layer),
                    options,
                    traversal,
                    &candidates,
                );
            }
        }
        for layer in &mut layers {
            layer.extruders.sort_unstable();
            layer.extruders.dedup();
            if layer.extruders.is_empty() && layer.has_object {
                layer.extruders.push(0);
            }
        }
    }
    let first_order = first_layer::generate(traversal)?;
    first_layer::handle_dontcare(&mut layers, &first_order, traversal);
    reorder::apply(&mut layers, traversal);
    let mut all_extruders = layers
        .iter()
        .flat_map(|layer| layer.extruders.iter().copied())
        .collect::<Vec<_>>();
    all_extruders.sort_unstable();
    all_extruders.dedup();
    Ok(ToolOrdering {
        layers,
        all_extruders,
    })
}

fn layer_index(layers: &[LayerTools], z: f64) -> usize {
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

fn initialize_layers(
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
            extruders: Vec::new(),
            has_object: false,
        });
        begin = end;
    }
    layers
}
