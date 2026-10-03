//! `ToolOrdering::collect_extruders` region roles and support selectors.

use super::LayerTools;
use crate::project_slice::{
    extrusion_islands::fill_role,
    island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};
use crate::{ExtrusionRole, ObjectOptions};

pub(super) fn object_options(
    traversal: &PreparedPostClassicTraversal,
    object: usize,
) -> &ObjectOptions {
    let source = traversal.objects[object]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity()
        .0;
    &traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == source)
        .expect("print object retains resolved source")
        .object
}

pub(super) fn object(
    target: &mut LayerTools,
    layer: &OrderedExtrusionLayer,
    traversal: &PreparedPostClassicTraversal,
    object: usize,
) {
    let regions = traversal.objects[object]
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
    for (usage, region) in layer.tool_regions.iter().zip(regions) {
        let options = &region.options;
        if usage.perimeters {
            target
                .extruders
                .push(options.outer_wall_filament_id.0 as usize);
            if options.wall_loops.0 > 1 {
                target
                    .extruders
                    .push(options.inner_wall_filament_id.0 as usize);
            }
        }
        for (present, id) in [
            (usage.internal_solid, options.internal_solid_filament_id.0),
            (usage.top, options.top_surface_filament_id.0),
            (usage.bottom, options.bottom_surface_filament_id.0),
            (usage.infill, options.sparse_infill_filament_id.0),
        ] {
            if present {
                target.extruders.push(id as usize);
            }
        }
        target.has_object |=
            usage.perimeters || usage.internal_solid || usage.top || usage.bottom || usage.infill;
    }
}

pub(super) fn support(
    target: &mut LayerTools,
    roles: impl Iterator<Item = ExtrusionRole>,
    options: &ObjectOptions,
    traversal: &PreparedPostClassicTraversal,
    candidates: &[usize],
) {
    let mut base = false;
    let mut interface = false;
    for role in roles {
        base |= matches!(
            role,
            ExtrusionRole::SupportMaterial | ExtrusionRole::SupportTransition
        );
        interface |= role == ExtrusionRole::SupportMaterialInterface;
    }
    let base_id = options.support_filament.0 as usize;
    let interface_id = options.support_interface_filament.0 as usize;
    if base {
        if base_id > 0 || !interface || interface_id == 0 || target.has_object {
            target.extruders.push(base_id);
        } else {
            let current = options
                .support_interface_not_for_body
                .0
                .then_some(interface_id - 1);
            let next = next_support_extruder(traversal, interface_id, current, candidates);
            target.extruders.push(next.map_or(0, |id| id + 1));
        }
    }
    if interface {
        target.extruders.push(interface_id);
    }
}

fn next_support_extruder(
    traversal: &PreparedPostClassicTraversal,
    interface_id: usize,
    current: Option<usize>,
    candidates: &[usize],
) -> Option<usize> {
    let runtime = &traversal.resolved.views.runtime_gcode;
    let print = &traversal.resolved.views.full.project.print;
    let matrix = &print.flush_volumes_matrix.0;
    let nozzle_width = matrix.len() / print.nozzle_diameter.0.len();
    let matrix = &matrix[..nozzle_width];
    let width = ((matrix.len() as f64).sqrt() + 1e-4) as usize;
    let mut next = current;
    let mut minimum = f32::MAX;
    for &id in candidates {
        let soluble = runtime
            .filament_soluble
            .0
            .get(id)
            .unwrap_or(&runtime.filament_soluble.0[0])
            .0;
        if soluble || Some(id) == current {
            continue;
        }
        let cost = matrix[(interface_id - 1) * width + id] as f32;
        if cost < minimum {
            next = Some(id);
            minimum = cost;
        }
    }
    next
}

pub(super) fn raft_roles(
    layer: &OrderedExtrusionLayer,
) -> impl Iterator<Item = ExtrusionRole> + '_ {
    layer
        .islands
        .iter()
        .flat_map(|island| island.entities.iter())
        .map(|entity| match entity {
            IslandPrintEntity::Fill(entity) => fill_role(entity),
            IslandPrintEntity::FillCollection(collection) => collection
                .entities
                .first()
                .map_or(ExtrusionRole::None, fill_role),
            IslandPrintEntity::Perimeter(_) | IslandPrintEntity::Thin(_) => {
                unreachable!("raft adapter materializes support fill paths")
            }
        })
}

/// `Print::extruders` candidate set for choosing a nonsoluble support tool.
pub(super) fn print_extruders(traversal: &PreparedPostClassicTraversal) -> Vec<usize> {
    let mut tools = crate::project_slice::extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    )
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for object in &traversal.resolved.objects {
        let options = &object.object;
        if !options.enable_support.0
            && options.enforce_support_layers.0 <= 0
            && options.raft_layers.0 <= 0
        {
            continue;
        }
        for id in [
            options.support_filament.0,
            options.support_interface_filament.0,
        ]
        .into_iter()
        .filter(|&id| id > 0)
        {
            tools.push(if id as usize > traversal.resolved.logical_filament_count {
                0
            } else {
                id as usize - 1
            });
        }
    }
    tools.sort_unstable();
    tools.dedup();
    tools
}
