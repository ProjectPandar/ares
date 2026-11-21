//! `ToolOrdering::collect_extruders` region roles and support selectors.

use super::LayerTools;
use crate::project_slice::{
    extrusion_islands::fill_role,
    island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
    perimeters::classic::traversal::PreparedPostClassicTraversal,
};
use crate::{ExtrusionRole, ObjectOptions};

pub(in crate::project_slice::gcode_emit) fn object_options(
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
    extruder_override: Option<usize>,
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
    let options = object_options(traversal, object);
    // `ToolOrdering.cpp:686-698,714-734`: on by-layer prints
    // (`m_print_config_ptr` set) an extruder only joins `LayerTools`
    // when its region has a non-overriddable entity —
    // `is_overriddable_and_mark` admits every role under
    // flush_into_objects and only internal infill under
    // flush_into_infill (`ToolOrdering.cpp:1586-1596`), so a fully
    // overriddable region adds no extruder and the print can collapse
    // to a single tool. ByObject ordering never gates
    // (`m_print_config_ptr` null).
    let by_layer = traversal.resolved.views.full.process.print.print_sequence
        != crate::ProcessPrintSequence::ByObject;
    let runtime = &traversal.resolved.views.runtime_gcode;
    let soluble = |id: i32| {
        usize::try_from(id).is_ok_and(|id| {
            runtime
                .filament_soluble
                .0
                .get(id)
                .is_some_and(|value| value.0)
        })
    };
    for (usage, region) in layer.tool_regions.iter().zip(regions) {
        let region_options = &region.options;
        // Walls are overriddable only under flush_into_objects (they are
        // never `erInternalInfill`); a soluble wall filament keeps its
        // extruder (`is_overriddable`'s first check).
        let walls_nonoverriddable = !options.flush_into_objects.0
            || soluble(region_options.outer_wall_filament_id.0.max(1) - 1)
            || (region_options.wall_loops.0 > 1
                && soluble(region_options.inner_wall_filament_id.0.max(1) - 1));
        if usage.perimeters && (!by_layer || walls_nonoverriddable) {
            target.extruders.push(
                extruder_override.unwrap_or(region_options.outer_wall_filament_id.0 as usize),
            );
            if extruder_override.is_none() && region_options.wall_loops.0 > 1 {
                target
                    .extruders
                    .push(region_options.inner_wall_filament_id.0 as usize);
            }
        }
        let categories = [
            (
                usage.internal_solid,
                options.flush_into_objects.0,
                region_options.internal_solid_filament_id.0,
            ),
            (
                usage.top,
                options.flush_into_objects.0,
                region_options.top_surface_filament_id.0,
            ),
            (
                usage.bottom,
                options.flush_into_objects.0,
                region_options.bottom_surface_filament_id.0,
            ),
            (
                usage.infill,
                options.flush_into_objects.0 || options.flush_into_infill.0,
                region_options.sparse_infill_filament_id.0,
            ),
            // `ToolOrdering.cpp:160–163,3394–3405`: standalone GapFill uses wall gating.
            (
                usage.outer_wall_fill,
                options.flush_into_objects.0,
                region_options.outer_wall_filament_id.0,
            ),
        ];
        // Upstream pushes the whole category block when ANY present fill
        // is non-overriddable (`something_nonoverriddable`, :718-733);
        // soluble filaments are never overriddable.
        let fills_nonoverriddable = categories
            .iter()
            .any(|&(present, overridable, id)| present && (!overridable || soluble(id.max(1) - 1)));
        let has_fill = categories.iter().any(|&(present, _, _)| present);
        match extruder_override {
            Some(extruder) if has_fill && (fills_nonoverriddable || !by_layer) => {
                target.extruders.push(extruder);
            }
            None if fills_nonoverriddable || !by_layer => target.extruders.extend(
                categories
                    .into_iter()
                    .filter(|&(present, _, _)| present)
                    .map(|(_, _, id)| id as usize),
            ),
            _ => {}
        }
        target.has_object |= usage.perimeters
            || usage.outer_wall_fill
            || usage.internal_solid
            || usage.top
            || usage.bottom
            || usage.infill;
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
    target.has_support |= base || interface;
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

/// `Print::extruders(false)` candidate set, without custom G-code tools.
pub(in crate::project_slice::gcode_emit) fn print_extruders(
    traversal: &PreparedPostClassicTraversal,
) -> Vec<usize> {
    let mut object_tools = crate::project_slice::extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    )
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    object_tools.sort_unstable();
    object_tools.dedup();
    let mut support_tools = Vec::new();
    let mut support_uses_current_extruder = false;
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
        ] {
            if id == 0 {
                support_uses_current_extruder = true;
                continue;
            }
            support_tools.push(if id as usize > traversal.resolved.logical_filament_count {
                0
            } else {
                id as usize - 1
            });
        }
    }
    if support_uses_current_extruder {
        support_tools.extend_from_slice(&object_tools);
    }
    support_tools.sort_unstable();
    support_tools.dedup();
    let mut tools = object_tools;
    tools.extend(support_tools);
    // Print.cpp:540-546,3290-3301: explicit tower filament joins the
    // candidate set based on configuration, not materialized tower paths.
    let settings = &traversal.resolved.views.full;
    let print = &settings.process.print;
    let tower = &settings.printer.gcode;
    let has_tower = print.enable_prime_tower.0
        && ((settings.process.gcode.enable_wrapping_detection.0
            && tower.wrapping_exclude_area.0.len() > 2)
            || print.timelapse_type == crate::ProcessTimelapseType::Smooth
            || (!print.spiral_mode.0 && settings.filament.gcode.filament_diameter.0.len() > 1));
    if has_tower && print.wipe_tower_filament.0 != 0 && tools.len() > 1 {
        tools.push(print.wipe_tower_filament.0 as usize - 1);
    }
    tools.sort_unstable();
    tools.dedup();
    tools
}
