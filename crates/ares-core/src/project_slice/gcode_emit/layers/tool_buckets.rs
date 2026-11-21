//! `GCode.cpp:4864–5130`: tool → object → island → region ownership.

mod object;
pub(super) mod support;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Pass {
    Wiping,
    Normal,
}

pub(super) struct Entry {
    pub(super) source: LayerToPrint,
    pub(super) instance: super::instances::Instance,
    pub(super) body: Option<OrderedExtrusionLayer>,
    pub(super) wipe_body: Option<OrderedExtrusionLayer>,
    pub(super) support_role: Option<ExtrusionRole>,
    pub(super) support_wiping: bool,
}

impl Entry {
    fn new(source: LayerToPrint, instance: super::instances::Instance) -> Self {
        Self {
            source,
            instance,
            body: None,
            wipe_body: None,
            support_role: None,
            support_wiping: false,
        }
    }
}

pub(super) struct Input<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) entries: &'a [LayerToPrint],
    pub(super) instances: &'a [Vec<super::instances::Instance>],
    /// By object, the phase drives one instance: only that copy's buckets
    /// are built, so the consuming adapter's clone/replay path keeps the
    /// last copy's in-place move for the phase that owns it.
    pub(super) copy_filter: Option<usize>,
    pub(super) layer_tools: &'a super::super::tool_ordering::LayerTools,
    pub(super) objects: &'a mut [Vec<OrderedExtrusionLayer>],
    pub(super) support: &'a [Vec<InstalledSupportLayer>],
    pub(super) raft: &'a mut [Vec<OrderedExtrusionLayer>],
}

pub(super) fn build(input: Input<'_>) -> Vec<(usize, super::island_visits::InstanceTour)> {
    let tools = &input.layer_tools.extruders;
    let wiping = &input.layer_tools.wiping;
    let mut buckets = BTreeMap::<usize, BTreeMap<usize, Entry>>::new();
    for (source, instance) in input.entries.iter().flat_map(|&source| {
        input.instances[source.object_index]
            .iter()
            .filter(move |&instance| input.copy_filter.is_none_or(|copy| instance.copy == copy))
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
                tools,
                [
                    wiping.support_tool(source.object_index, false),
                    wiping.support_tool(source.object_index, true),
                ],
            ) {
                let entry = buckets
                    .entry(tool)
                    .or_default()
                    .entry(position)
                    .or_insert_with(|| Entry::new(source, instance));
                entry.support_role = Some(role);
                entry.support_wiping = wiping
                    .support_tool(
                        source.object_index,
                        role == ExtrusionRole::SupportMaterialInterface,
                    )
                    .is_some();
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
            object::raft(layer, options, input.traversal, tools)
                .into_iter()
                .map(|(tool, layer)| ((tool, false), layer))
                .collect()
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
            object::split(
                layer,
                (tools, input.layer_tools.extruder_override),
                |region| &regions[region].options,
                |collection| wiping.entity(source.object_index, collection, instance.copy),
            )
        } else {
            BTreeMap::new()
        };
        for ((tool, wiping), body) in bodies {
            let entry = buckets
                .entry(tool)
                .or_default()
                .entry(position)
                .or_insert_with(|| Entry::new(source, instance));
            if wiping {
                entry.wipe_body = Some(body);
            } else {
                entry.body = Some(body);
            }
        }
        if super::instances::as_object_list(input.traversal) {
            // `GCode.cpp:4101` sizes every filament's `objects_by_extruder` to
            // all objects of the layer, and the `ordering != nullptr` branch of
            // `sort_print_object_instances` (:4140-4160) only requires
            // `layers[layer_id].original_object`, never islands. An object with
            // nothing on this filament is therefore still visited, with an empty
            // island list, and emits only its enter/exit markers.
            for &tool in tools {
                buckets
                    .entry(tool)
                    .or_default()
                    .entry(position)
                    .or_insert_with(|| Entry::new(source, instance));
            }
        }
    }
    tools
        .iter()
        .map(|&tool| {
            let entries = buckets.remove(&tool).unwrap_or_default();
            (
                tool,
                super::island_visits::plan(
                    entries.into_values().collect(),
                    input.traversal,
                    input.copy_filter,
                ),
            )
        })
        .collect()
}
