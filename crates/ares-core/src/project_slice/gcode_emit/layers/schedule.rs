//! `GCode.cpp::collect_layers_to_print` object/support pairs and plate chunks.

use super::{object::ObjectLabels, object_order, raft_schedule};
use crate::project_slice::{
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

const EPSILON: f64 = 1e-4;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit) struct LayerToPrint {
    pub(super) object_index: usize,
    pub(super) object_layer: Option<usize>,
    pub(super) support_layer: Option<usize>,
    // Temporary raft-only adapter; ordinary support owns its raft layers.
    pub(super) raft_layer: Option<usize>,
}

pub(in crate::project_slice::gcode_emit) struct Schedule {
    pub(in crate::project_slice::gcode_emit) tool_ordering:
        super::super::tool_ordering::ToolOrdering,
    pub(in crate::project_slice::gcode_emit) merged: Vec<(f64, LayerToPrint)>,
    pub(super) raft: Option<raft_schedule::RaftSchedule>,
    pub(super) print_position: Vec<usize>,
    pub(super) labels: Vec<Option<ObjectLabels>>,
    pub(super) object_layer_counts: Vec<usize>,
    pub(super) last_entry: Option<(usize, usize)>,
    pub(super) layer_count: usize,
    pub(super) skirt_schedule: super::skirt_schedule::SkirtSchedule,
}

impl Schedule {
    pub(in crate::project_slice::gcode_emit) fn layer_count(&self) -> usize {
        self.layer_count
    }
}

pub(in crate::project_slice::gcode_emit) fn build(
    traversal: &PreparedPostClassicTraversal,
    objects: &[Vec<OrderedExtrusionLayer>],
    support: &mut [Vec<InstalledSupportLayer>],
    context: &mut crate::SlicerContext,
) -> Result<Schedule, crate::SliceError> {
    let object_count = objects.len();
    let settings = &traversal.resolved.views.full;
    let object_options = traversal
        .objects
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let source_index = source
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity()
                .0;
            let mut options = traversal
                .resolved
                .objects
                .iter()
                .find(|object| object.source_object_index == source_index)
                .expect("print object retains its resolved source")
                .object
                .clone();
            if !support[index].is_empty() {
                options.raft_layers.0 = 0;
            }
            options
        })
        .collect::<Vec<_>>();
    let first_layer_lslices = traversal
        .objects
        .iter()
        .map(|object| object.slices(0))
        .collect::<Vec<_>>();
    let slicing_parameters = traversal
        .objects
        .iter()
        .map(|object| object.slicing_parameters())
        .collect::<Vec<_>>();
    let raft = raft_schedule::build(
        settings,
        &object_options,
        &first_layer_lslices,
        &slicing_parameters,
        traversal.scale,
        context,
    )?;
    let mut tool_ordering =
        super::super::tool_ordering::build(traversal, objects, support, raft.as_ref())?;
    tool_ordering.install_tower_support_layers(support);
    let mut merged = Vec::new();
    let mut raw_zs = Vec::new();
    let mut skirt_layers = Vec::new();
    for (object_index, source) in traversal.objects.iter().enumerate() {
        let object_z = source
            .physical_layers()
            .iter()
            .map(|layer| layer.print_z)
            .collect::<Vec<_>>();
        let support_z = support[object_index]
            .iter()
            .map(|layer| layer.print_z)
            .collect::<Vec<_>>();
        raw_zs.extend_from_slice(&object_z);
        raw_zs.extend_from_slice(&support_z);
        skirt_layers.extend(
            object_z
                .iter()
                .zip(&objects[object_index])
                .map(|(&z, layer)| {
                    let has_object = layer
                        .islands
                        .iter()
                        .any(|island| !island.entities.is_empty());
                    super::skirt_schedule::Layer::new(z, has_object, has_object)
                }),
        );
        skirt_layers.extend(support[object_index].iter().map(|layer| {
            let has_extrusions = layer.support_fills.iter().any(|entity| {
                let role = entity.role();
                matches!(
                    role,
                    crate::ExtrusionRole::SupportMaterial
                        | crate::ExtrusionRole::SupportMaterialInterface
                        | crate::ExtrusionRole::SupportTransition
                )
            });
            super::skirt_schedule::Layer::new(layer.print_z, false, has_extrusions)
        }));
        pair_layers(&mut merged, object_index, &object_z, &support_z);
    }
    if let Some(raft) = &raft {
        raw_zs.extend(raft.entries.iter().map(|entry| entry.0));
        skirt_layers.extend(raft.entries.iter().map(|&(z, object, sentinel)| {
            let layer = sentinel - raft_schedule::RAFT_SENTINEL_BASE;
            let has_extrusions = raft.layers[object][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty());
            super::skirt_schedule::Layer::new(z, false, has_extrusions)
        }));
        merged.extend(raft.entries.iter().map(|&(z, object_index, sentinel)| {
            (
                z,
                LayerToPrint {
                    object_index,
                    object_layer: None,
                    support_layer: None,
                    raft_layer: Some(sentinel - raft_schedule::RAFT_SENTINEL_BASE),
                },
            )
        }));
    }
    merged.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Header/tail m_layer_count uses raw unique Z adjacency, not chunk groups.
    raw_zs.sort_by(f64::total_cmp);
    raw_zs.dedup();
    let layer_count = usize::from(!raw_zs.is_empty())
        + raw_zs
            .windows(2)
            .filter(|pair| (pair[1] - pair[0]).abs() >= EPSILON)
            .count();
    for object_index in 0..object_count {
        let first = &merged
            .iter()
            .find(|(_, entry)| entry.object_index == object_index)
            .ok_or_else(|| crate::SliceError::InvalidInput("no layers were detected".into()))?
            .1;
        let object_extrusions = first.object_layer.is_some_and(|layer| {
            objects[object_index][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty())
        });
        let support_extrusions = first
            .support_layer
            .is_some_and(|layer| !support[object_index][layer].support_fills.is_empty());
        let raft_extrusions = first.raft_layer.is_some_and(|layer| {
            raft.as_ref()
                .expect("raft entry retains its adapter")
                .layers[object_index][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty())
        });
        if !object_extrusions && !support_extrusions && !raft_extrusions {
            return Err(crate::SliceError::InvalidInput(
                "one object has an empty first layer and cannot be printed".into(),
            ));
        }
    }
    // Native grouping compares against the group's minimum, not its previous Z.
    let mut begin = 0;
    while begin < merged.len() {
        let minimum = merged[begin].0;
        let mut end = begin + 1;
        while end < merged.len() && merged[end].0 <= minimum + EPSILON {
            end += 1;
        }
        let z = 0.5 * (minimum + merged[end - 1].0);
        for entry in &mut merged[begin..end] {
            entry.0 = z;
        }
        begin = end;
    }
    let centers = traversal
        .objects
        .iter()
        .map(|source| {
            let source_index = source
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity()
                .0;
            super::footprint::object_center(traversal, source_index).unwrap_or((0.0, 0.0))
        })
        .collect::<Vec<_>>();
    let mut print_order = object_order::chain_instance_order(&centers, (15, 220));
    print_order.reverse();
    let mut print_position = vec![0; object_count];
    for (rank, &object_index) in print_order.iter().enumerate() {
        print_position[object_index] = rank;
    }
    let labels = (0..object_count)
        .map(|index| ObjectLabels::from_traversal(traversal, index))
        .collect();
    let object_layer_counts = objects.iter().map(Vec::len).collect();
    let last_entry = merged
        .last()
        .and_then(|(_, entry)| entry.object_layer.map(|layer| (entry.object_index, layer)));
    let max_layer_height = super::super::tool_ordering::max_layer_height(
        settings,
        object_options
            .iter()
            .map(|object| object.layer_height.0)
            .fold(0.0, f64::max),
    );
    let skirt_schedule = super::skirt_schedule::SkirtSchedule::new(skirt_layers, max_layer_height);
    Ok(Schedule {
        tool_ordering,
        merged,
        raft,
        print_position,
        labels,
        object_layer_counts,
        last_entry,
        layer_count,
        skirt_schedule,
    })
}

fn pair_layers(
    destination: &mut Vec<(f64, LayerToPrint)>,
    object_index: usize,
    object_z: &[f64],
    support_z: &[f64],
) {
    let (mut object, mut support) = (0, 0);
    while object < object_z.len() || support < support_z.len() {
        let minimum = object_z
            .get(object)
            .copied()
            .unwrap_or(f64::MAX)
            .min(support_z.get(support).copied().unwrap_or(f64::MAX));
        let object_layer = object_z
            .get(object)
            .filter(|&&z| z <= minimum + EPSILON)
            .map(|_| object);
        let support_layer = support_z
            .get(support)
            .filter(|&&z| z <= minimum + EPSILON)
            .map(|_| support);
        let z = match (object_layer, support_layer) {
            (Some(object), Some(support)) => 0.5 * (object_z[object] + support_z[support]),
            (Some(object), None) => object_z[object],
            (None, Some(support)) => support_z[support],
            (None, None) => unreachable!("at least one next layer is at the minimum Z"),
        };
        destination.push((
            z,
            LayerToPrint {
                object_index,
                object_layer,
                support_layer,
                raft_layer: None,
            },
        ));
        object += usize::from(object_layer.is_some());
        support += usize::from(support_layer.is_some());
    }
}
