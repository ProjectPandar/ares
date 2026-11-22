//! `ToolOrdering.cpp` materialized layer tools and writer-used IDs.

mod collect;
mod custom_gcodes;
mod first_layer;
mod initialize;
use initialize::{initialize_layers, initialize_layers_scoped, layer_index};
mod partitions;
mod recommend;
pub(super) use recommend::build_recommended;
mod reorder;
mod statistics;

mod nozzle_group;
pub(super) use nozzle_group::{NozzleGroup, nozzle_count};
pub(super) mod wiping;

pub(super) use collect::{object_options, print_extruders};
pub(super) use partitions::max_layer_height;

use super::layers::RaftSchedule;

use crate::project_slice::{
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

#[derive(Default)]
pub(super) struct LayerTools {
    pub(super) print_z: f64,
    pub(super) extruders: Vec<usize>,
    /// `LayerTools::extruder_override` (`ToolOrdering.cpp:837`), zero-based for export.
    pub(super) extruder_override: Option<usize>,
    has_object: bool,
    has_support: bool,
    pub(super) wipe_tower_partitions: usize,
    pub(super) has_wipe_tower: bool,
    pub(super) wipe_tower_layer_height: f64,
    pub(super) wiping: wiping::WipingExtrusions,
    /// `LayerTools::custom_gcode` (`ToolOrdering.hpp:168`).
    pub(super) custom_gcode: Option<crate::gcode_processor::CustomGCodeItem>,
}

pub(super) struct ToolOrdering {
    pub(super) layers: Vec<LayerTools>,
    pub(super) nozzle_groups: Vec<NozzleGroup>,
    all_extruders: Vec<usize>,
    last_layer_per_extruder: Vec<Option<f64>>,
    pub(super) first_extruder: Option<usize>,
    pub(super) tower_type: Option<crate::WipeTowerType>,
}

/// `ToolOrdering.cpp:962-972`: count each distinct extruder of the layer
/// once (dedup on a reused scratch buffer, then bump the presence counts).
fn count_layer_extruders(distinct: &mut Vec<usize>, counts: &mut [usize]) {
    distinct.sort_unstable();
    distinct.dedup();
    for extruder in distinct.iter().copied() {
        counts[extruder] += 1;
    }
}

impl ToolOrdering {
    /// `Print.cpp:3409–3443`: install the first contiguous tower-only layer run.
    pub(super) fn install_tower_support_layers(
        &mut self,
        support: &mut [Vec<InstalledSupportLayer>],
    ) {
        if self.tower_type.is_none() {
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

    /// `ToolOrdering::last_extruder` (`ToolOrdering.cpp:932-935`): the
    /// trailing extruder of the last non-empty layer.
    pub(super) fn last_extruder(&self) -> Option<usize> {
        self.layers
            .iter()
            .rev()
            .find(|layer| !layer.extruders.is_empty())
            .and_then(|layer| layer.extruders.last().copied())
    }

    pub(super) fn initial_extruder(&self, settings: &crate::ProjectSettings) -> usize {
        if settings.process.print.print_sequence == crate::ProcessPrintSequence::ByObject {
            self.ordered_tools().next().unwrap_or(0)
        } else if self.tower_type == Some(crate::WipeTowerType::Type2)
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
    /// `ToolOrdering::cal_most_used_extruder` (`ToolOrdering.cpp:955-981`):
    /// per physical extruder (`filament_map[f] - 1`), count the layers it
    /// appears on (distinct per layer); the highest index wins ties.
    pub(super) fn most_used_extruder(
        &self,
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    ) -> usize {
        let views = &traversal.resolved.views.full;
        let map = &views.project.gcode.filament_map.0;
        let extruder_of = |filament: usize| -> usize { (map[filament].0 - 1) as usize };
        let mut counts = vec![0usize; views.project.print.nozzle_diameter.0.len()];
        let mut distinct = Vec::new();
        for layer in &self.layers {
            distinct.extend(layer.extruders.iter().map(|f| extruder_of(*f)));
            count_layer_extruders(&mut distinct, &mut counts);
        }
        let mut most_used = 0;
        for (extruder, count) in counts.iter().enumerate().skip(1) {
            if *count >= counts[most_used] {
                most_used = extruder;
            }
        }
        most_used
    }

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

    pub(super) fn is_last_extrusion_layer(&self, print_z: f64, extruder_id: usize) -> bool {
        self.last_layer_per_extruder
            .get(extruder_id)
            .and_then(|last| *last)
            .is_none_or(|last| print_z >= last - 1e-4)
    }

    pub(super) fn ordered_tools(&self) -> impl Iterator<Item = usize> + '_ {
        self.layers
            .iter()
            .flat_map(|layer| layer.extruders.iter().copied())
    }

    pub(super) fn for_layer(&self, print_z: f64) -> &LayerTools {
        &self.layers[layer_index(&self.layers, print_z)]
    }

    pub(super) fn tools_for_layer(&self, print_z: f64) -> &[usize] {
        let index = layer_index(&self.layers, print_z);
        &self.layers[index].extruders
    }
}

/// Which objects an ordering pass covers: the by-layer pass over every
/// object, or the sequential per-object pass
/// (`ToolOrdering(object, final_extruder_id)`, `ToolOrdering.cpp:396-436`)
/// seeded with the previous object's final extruder through the dontcare
/// handling (`ToolOrdering.cpp:253-300`).
pub(super) enum OrderingScope {
    All,
    Object {
        object: usize,
        seed_extruder: Option<usize>,
    },
}

pub(super) fn build(
    traversal: &PreparedPostClassicTraversal,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&RaftSchedule>,
    scope: OrderingScope,
) -> Result<ToolOrdering, crate::SliceError> {
    let scoped = matches!(scope, OrderingScope::Object { .. });
    let (layers, bottom_z, max_height) = collect_layers(traversal, objects, support, raft, scope)?;
    finish_layers(traversal, layers, bottom_z, max_height, scoped)
}

fn collect_layers(
    traversal: &PreparedPostClassicTraversal,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &[Vec<InstalledSupportLayer>],
    raft: Option<&RaftSchedule>,
    scope: OrderingScope,
) -> Result<(Vec<LayerTools>, f64, f64), crate::SliceError> {
    let (scoped_object, seed_extruder) = match scope {
        OrderingScope::All => (None, None),
        OrderingScope::Object {
            object,
            seed_extruder,
        } => (Some(object), seed_extruder),
    };
    let mut layers = match scoped_object {
        Some(object) => initialize_layers_scoped(
            traversal,
            support,
            raft.map(|raft| raft.plans.as_slice()),
            object,
        ),
        None => initialize_layers(traversal, support, raft.map(|raft| raft.plans.as_slice())),
    };
    let candidates = collect::print_extruders(traversal);
    let per_layer_extruder_switches = if scoped_object.is_some() {
        Vec::new()
    } else {
        custom_gcodes::multi_as_single_switches(traversal)
    };
    let mut object_bottom_z = 0.0;
    let mut max_object_layer_height = 0.0_f64;
    let object_range = match scoped_object {
        Some(object) => object..object + 1,
        None => 0..objects.len(),
    };
    for object in object_range {
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
        let mut switch_index = 0;
        let mut extruder_override = None;
        for (physical, layer) in traversal.objects[object]
            .physical_layers()
            .iter()
            .zip(&objects[object])
        {
            while switch_index < per_layer_extruder_switches.len()
                && per_layer_extruder_switches[switch_index].0 < physical.print_z + 1e-4
            {
                extruder_override = (per_layer_extruder_switches[switch_index].1 != 0)
                    .then_some(per_layer_extruder_switches[switch_index].1);
                switch_index += 1;
            }
            let target = layer_index(&layers, physical.print_z);
            layers[target].extruder_override = extruder_override.map(|id| id - 1);
            collect::object(
                &mut layers[target],
                layer,
                traversal,
                object,
                extruder_override,
            );
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
    match seed_extruder {
        // `ToolOrdering.cpp:429`: the seed fills the dontcare handling
        // instead of the generated first-layer tool order. The first
        // layer's region options still carry the raw one-based ids that
        // the global pass rewrites through `handle_dontcare`; apply the
        // same rewrite before leading with the seed.
        Some(seed) => {
            first_layer::handle_dontcare(&mut layers, &first_order, traversal);
            first_layer::handle_dontcare_seed(&mut layers, Some(seed));
        }
        None => first_layer::handle_dontcare(&mut layers, &first_order, traversal),
    }
    Ok((layers, object_bottom_z, max_object_layer_height))
}

fn finish_layers(
    traversal: &PreparedPostClassicTraversal,
    mut layers: Vec<LayerTools>,
    object_bottom_z: f64,
    max_object_layer_height: f64,
    scoped: bool,
) -> Result<ToolOrdering, crate::SliceError> {
    reorder::apply(&mut layers, traversal, scoped);
    let settings = &traversal.resolved.views.full;
    let max_height = max_layer_height(settings, max_object_layer_height);
    partitions::fill(&mut layers, settings, object_bottom_z, max_height);
    if partitions::insert_extruder(&mut layers, settings) {
        reorder::apply(&mut layers, traversal, scoped);
        partitions::fill(&mut layers, settings, object_bottom_z, max_height);
    }
    let requested =
        super::wipe_tower::requested(settings, super::tags::Tags::of(traversal).is_bbl());
    let prime = requested == Some(crate::WipeTowerType::Type2);
    let (all_extruders, first_extruder) = statistics::collect(&layers, prime);
    let last_layer_per_extruder =
        statistics::collect_last_layer_per_extruder(&layers, &all_extruders);
    let tower_type = requested.filter(|_| {
        first_extruder.is_some() && layers.first().is_some_and(|layer| layer.has_wipe_tower)
    });
    Ok(ToolOrdering {
        layers,
        nozzle_groups: nozzle_group::static_groups(settings),
        all_extruders,
        last_layer_per_extruder,
        first_extruder,
        tower_type,
    })
}
