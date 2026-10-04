//! `GCode.cpp:4864–5130`: tool → object → island → region ownership.

mod object;
mod support;

use std::collections::BTreeMap;

use super::schedule::LayerToPrint;
use crate::{
    ExtrusionRole,
    project_slice::{
        island_print_order::OrderedExtrusionLayer,
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(super) struct Entry {
    pub(super) source: LayerToPrint,
    pub(super) instance: super::instances::Instance,
    pub(super) body: Option<OrderedExtrusionLayer>,
    pub(super) support_role: Option<ExtrusionRole>,
}

pub(super) struct Input<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) entries: &'a [LayerToPrint],
    pub(super) instances: &'a [Vec<super::instances::Instance>],
    pub(super) tools: &'a [usize],
    pub(super) objects: &'a mut [Vec<OrderedExtrusionLayer>],
    pub(super) support: &'a [Vec<InstalledSupportLayer>],
    pub(super) raft: &'a mut [Vec<OrderedExtrusionLayer>],
}

pub(super) fn build(input: Input<'_>) -> Vec<(usize, Vec<Entry>)> {
    let mut buckets = BTreeMap::<usize, BTreeMap<usize, Entry>>::new();
    for (source, instance) in input.entries.iter().flat_map(|&source| {
        input.instances[source.object_index]
            .iter()
            .map(move |&instance| (source, instance))
    }) {
        let position = instance.position;
        let replay = instance.copy + 1 < input.instances[source.object_index].len();
        let options =
            super::super::tool_ordering::object_options(input.traversal, source.object_index);
        if let Some(index) = source.support_layer {
            for (tool, role) in support::selectors(
                input.support[source.object_index][index]
                    .support_fills
                    .iter()
                    .map(|entity| entity.role()),
                options,
                input.traversal,
                input.tools,
            ) {
                buckets
                    .entry(tool)
                    .or_default()
                    .entry(position)
                    .or_insert_with(|| Entry {
                        source,
                        instance,
                        body: None,
                        support_role: None,
                    })
                    .support_role = Some(role);
            }
        }
        let bodies = if let Some(index) = source.raft_layer {
            let layer = &mut input.raft[source.object_index][index];
            let mut repeated;
            let layer = if replay {
                repeated = layer.clone();
                &mut repeated
            } else {
                layer
            };
            object::raft(layer, options, input.traversal, input.tools)
        } else if let Some(index) = source.object_layer {
            let regions = input.traversal.objects[source.object_index]
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .as_parts()
                .0
                .as_parts()
                .0
                .as_parts()
                .2;
            let layer = &mut input.objects[source.object_index][index];
            // The consuming adapter replays additional copies, moving the last
            // one. A single instance retains its allocation-free move path.
            let mut repeated;
            let layer = if replay {
                repeated = layer.clone();
                &mut repeated
            } else {
                layer
            };
            object::split(layer, input.tools, |region| &regions[region].options)
        } else {
            BTreeMap::new()
        };
        for (tool, body) in bodies {
            buckets
                .entry(tool)
                .or_default()
                .entry(position)
                .or_insert_with(|| Entry {
                    source,
                    instance,
                    body: None,
                    support_role: None,
                })
                .body = Some(body);
        }
    }
    input
        .tools
        .iter()
        .filter_map(|&tool| {
            buckets.remove(&tool).map(|entries| {
                (
                    tool,
                    super::instances::order(entries.into_values().collect(), input.traversal),
                )
            })
        })
        .collect()
}
