//! `TreeSupport.cpp:3251-3400::plan_layer_heights`.

use std::collections::BTreeMap;

use super::nodes::{NodeArena, NodeId};
use crate::project_slice::{layers::PlannedLayer, parameters::SlicingParameters};

const EPSILON: f64 = 0.0001;

pub(super) struct LayerHeightData {
    pub(super) print_z: f64,
    pub(super) height: f64,
    pub(super) object_layer: usize,
}

pub(super) struct HeightInput<'a> {
    pub(super) layers: &'a [PlannedLayer],
    pub(super) slicing: &'a SlicingParameters,
    pub(super) independent: bool,
    pub(super) top_z_distance: f64,
    pub(super) raft_layers: usize,
}

pub(super) fn plan_layer_heights(
    input: &HeightInput<'_>,
    arena: &mut NodeArena,
    contact_nodes: &mut Vec<Vec<NodeId>>,
) -> Vec<LayerHeightData> {
    let heights = if input.independent {
        independent_heights(input, arena, contact_nodes)
    } else {
        input
            .layers
            .iter()
            .enumerate()
            .map(|(object_layer, layer)| LayerHeightData {
                print_z: layer.print_z,
                height: layer.height,
                object_layer,
            })
            .collect()
    };
    let mut redistributed = vec![Vec::new(); input.raft_layers + heights.len()];
    for nodes in contact_nodes.iter() {
        if nodes.is_empty() {
            continue;
        }
        let print_z = arena.nodes[nodes[0]].print_z;
        let mut closest = 0;
        for index in 1..heights.len() {
            if (heights[index].print_z - print_z).abs() < (heights[closest].print_z - print_z).abs()
            {
                closest = index;
            }
        }
        redistributed[closest].extend_from_slice(nodes);
    }
    *contact_nodes = redistributed;
    adjust_gaps(input, &heights, arena, contact_nodes);
    heights
}

fn independent_heights(
    input: &HeightInput<'_>,
    arena: &NodeArena,
    contacts: &[Vec<NodeId>],
) -> Vec<LayerHeightData> {
    let first = &input.layers[0];
    // Finite positive print-Z IEEE bits have the same order as native map<double>.
    let mut bounds = BTreeMap::from([(first.print_z.to_bits(), first.height)]);
    let mut z_heights = bounds.clone();
    let object_z = input
        .layers
        .iter()
        .map(|layer| layer.print_z as f32)
        .collect::<Vec<_>>();
    for nodes in contacts.iter().skip(1).filter(|nodes| !nodes.is_empty()) {
        let node = &arena.nodes[nodes[0]];
        bounds.entry(node.print_z.to_bits()).or_insert(node.height);
        bounds
            .entry((node.print_z - node.height).max(first.print_z).to_bits())
            .or_insert(0.0);
    }
    let mut previous_z = first.print_z;
    for (&bits, _) in bounds.iter().skip(1) {
        let next_z = f64::from_bits(bits);
        let distance = next_z - previous_z;
        if distance < input.slicing.min_layer_height - EPSILON {
            continue;
        }
        let count = (distance / input.slicing.max_support_layer_height).ceil() as usize;
        let step = distance / count as f64;
        let mut print_z = previous_z + step;
        for _ in 0..count {
            z_heights.insert(print_z.to_bits(), step);
            print_z += step;
        }
        previous_z = next_z;
    }
    let mut object_layer = 0;
    z_heights
        .into_iter()
        .map(|(bits, height)| {
            let print_z = f64::from_bits(bits);
            while object_layer < object_z.len()
                && f64::from(object_z[object_layer]) < print_z - height / 2.0
            {
                object_layer += 1;
            }
            LayerHeightData {
                print_z,
                height,
                object_layer,
            }
        })
        .collect()
}

fn adjust_gaps(
    input: &HeightInput<'_>,
    heights: &[LayerHeightData],
    arena: &mut NodeArena,
    contacts: &[Vec<NodeId>],
) {
    for (layer, nodes) in contacts.iter().enumerate() {
        if nodes.is_empty() {
            continue;
        }
        let original_height = arena.nodes[nodes[0]].height;
        let height = heights[layer].height;
        if (original_height - height).abs() < EPSILON
            || (input.top_z_distance < EPSILON && original_height < EPSILON)
        {
            continue;
        }
        let mut accumulated = 0.0;
        let mut count = 0;
        for lower in heights[..=layer].iter().rev() {
            if lower.height <= EPSILON {
                continue;
            }
            accumulated += lower.height;
            count += 1;
            if accumulated > original_height - EPSILON {
                break;
            }
        }
        for &id in nodes {
            arena.nodes[id].height = height;
            arena.nodes[id].distance_to_top = -count;
        }
    }
}
