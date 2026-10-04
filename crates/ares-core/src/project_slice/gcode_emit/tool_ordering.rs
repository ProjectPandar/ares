//! `ToolOrdering.cpp` materialized layer tools and writer-used IDs.

mod collect;
mod first_layer;
mod partitions;
mod reorder;
mod statistics;

pub(super) use collect::object_options;
pub(super) use partitions::max_layer_height;

use super::layers::RaftSchedule;

use crate::project_slice::{
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

#[derive(Default)]
pub(super) struct LayerTools {
    print_z: f64,
    extruders: Vec<usize>,
    has_object: bool,
    has_support: bool,
    wipe_tower_partitions: usize,
    has_wipe_tower: bool,
    wipe_tower_layer_height: f64,
}

pub(super) struct ToolOrdering {
    layers: Vec<LayerTools>,
    all_extruders: Vec<usize>,
    first_extruder: Option<usize>,
    type2_tower: bool,
}

impl ToolOrdering {
    /// `Print.cpp:3409–3443`: install the first contiguous tower-only layer run.
    pub(super) fn install_tower_support_layers(
        &mut self,
        support: &mut [Vec<InstalledSupportLayer>],
    ) {
        if !self.type2_tower {
            return;
        }
        let tower_only =
            |layer: &LayerTools| layer.has_wipe_tower && !layer.has_object && !layer.has_support;
        let Some(begin) = self.layers.iter().position(tower_only) else {
            return;
        };
        let end = begin
            + self.layers[begin..]
                .iter()
                .take_while(|layer| tower_only(layer))
                .count();
        // Upstream deliberately installs these in Print::objects().front().
        let first = support.first_mut().expect("active print has an object");
        for index in begin..end {
            let previous_z = if index == 0 {
                0.0
            } else {
                self.layers[index - 1].print_z
            };
            let layer = &mut self.layers[index];
            layer.has_support = true;
            let insert = first.partition_point(|support| support.print_z < layer.print_z - 1e-4);
            first.insert(
                insert,
                InstalledSupportLayer::for_wipe_tower(layer.print_z, layer.print_z - previous_z),
            );
        }
    }

    pub(super) fn initial_extruder(&self, settings: &crate::ProjectSettings) -> usize {
        if settings.process.print.print_sequence == crate::ProcessPrintSequence::ByObject {
            self.ordered_tools().next().unwrap_or(0)
        } else if self.type2_tower
            && !settings
                .process
                .gcode
                .single_extruder_multi_material_priming
                .0
        {
            *self
                .all_extruders
                .last()
                .expect("active tower has a printing tool")
        } else {
            self.first_extruder.unwrap_or(0)
        }
    }
    /// `DoExport::total_toolchanges_from_ordering` (`GCode.cpp:2289–2300`).
    pub(super) fn total_toolchanges(&self) -> usize {
        let mut last = None;
        let mut changes = 0;
        for filament in self.ordered_tools() {
            if last.is_some_and(|last| last != filament) {
                changes += 1;
            }
            last = Some(filament);
        }
        changes
    }

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
    let mut object_bottom_z = 0.0;
    let mut max_object_layer_height = 0.0_f64;
    for object in 0..objects.len() {
        let options = collect::object_options(traversal, object);
        max_object_layer_height = max_object_layer_height.max(options.layer_height.0);
        // `sort_and_build_data(Print)` retains the last object's first extrusion bottom.
        if let Some((physical, _)) = traversal.objects[object]
            .physical_layers()
            .iter()
            .zip(&objects[object])
            .find(|(_, layer)| {
                layer
                    .islands
                    .iter()
                    .any(|island| !island.entities.is_empty())
            })
        {
            object_bottom_z = physical.print_z - physical.height;
        }
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
    let settings = &traversal.resolved.views.full;
    let max_height = max_layer_height(settings, max_object_layer_height);
    partitions::fill(&mut layers, settings, object_bottom_z, max_height);
    if partitions::insert_extruder(&mut layers, settings) {
        reorder::apply(&mut layers, traversal);
        partitions::fill(&mut layers, settings, object_bottom_z, max_height);
    }
    let prime =
        super::wipe_tower::type2_requested(settings, super::tags::Tags::of(traversal).is_bbl());
    let (all_extruders, first_extruder) = statistics::collect(&layers, prime);
    let type2_tower = prime
        && first_extruder.is_some()
        && layers.first().is_some_and(|layer| layer.has_wipe_tower);
    Ok(ToolOrdering {
        layers,
        all_extruders,
        first_extruder,
        type2_tower,
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
            ..Default::default()
        });
        begin = end;
    }
    layers
}
