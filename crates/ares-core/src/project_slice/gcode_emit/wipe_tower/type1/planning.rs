//! `Print.cpp:3444–3537`: Type1 nozzle-aware prime/purge planning.
use super::super::{
    super::tool_ordering::{
        ToolOrdering,
        wiping::{self, Layer, WipingExtrusions},
    },
    filament::at,
};
use super::{filament::Filament, plan::Plan};
use crate::project_slice::{
    extrusion_islands::PreparedPostExtrusionIslands,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

pub(in crate::project_slice::gcode_emit) struct Planned {
    pub(super) plan: Plan,
    pub(super) filaments: Vec<Filament>,
    pub(super) first_tool: usize,
    pub(super) used: Vec<usize>,
}

pub(in crate::project_slice::gcode_emit) fn prepare(
    traversal: &PreparedPostClassicTraversal,
    original: &PreparedPostExtrusionIslands,
    support: &[Vec<InstalledSupportLayer>],
    ordering: &mut ToolOrdering,
) -> Planned {
    let settings = &traversal.resolved.views.full;
    let config = &settings.filament.gcode;
    let count = config.filament_colour.0.len();
    let nozzle_count = settings.project.print.nozzle_diameter.0.len();
    let raw_matrix = &settings.project.print.flush_volumes_matrix.0;
    // PrintConfig.hpp:2133–2140: contiguous matrix per physical nozzle.
    let matrix_size = raw_matrix.len() / nozzle_count;
    let matrices = raw_matrix
        .chunks_exact(matrix_size)
        .map(|matrix| {
            matrix
                .iter()
                .map(|&volume| volume as f32)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let soluble = (0..count)
        .map(|i| at(&config.filament_soluble.0, i).0)
        .collect::<Vec<_>>();
    let support_filaments = (0..count)
        .map(|i| at(&config.filament_is_support.0, i).0)
        .collect::<Vec<_>>();
    let first_tool = ordering
        .first_extruder
        .expect("active tower has a first tool");
    let mut plan = Plan::new(settings, ordering.all_extruders());
    let mut used = std::collections::BTreeSet::new();
    let mut current = first_tool;
    let mut nozzle_filaments = vec![None; nozzle_count];
    nozzle_filaments[plan.nozzle_map[current]] = Some(current);
    let mut layers = ordering.layers.iter_mut().peekable();
    while let Some(tools) = layers.next() {
        if !tools.has_wipe_tower {
            continue;
        }
        let position = (tools.print_z as f32, tools.wipe_tower_layer_height as f32);
        plan.add(position, (current, current), 0.0, 0.0);
        used.extend(tools.extruders.iter().copied());
        let objects = wiping::objects(traversal, original, support, tools.print_z);
        let layer = Layer {
            objects: &objects,
            tools: &tools.extruders,
            soluble: &soluble,
            support_filaments: &support_filaments,
        };
        let mut wiping = WipingExtrusions::new(&layer);
        for &next in &tools.extruders {
            if next == current {
                continue;
            }
            let nozzle = plan.nozzle_map[next];
            let mut purge = 0.0;
            if let Some(previous) = nozzle_filaments[nozzle]
                && previous != next
            {
                purge = (f64::from(matrices[nozzle][previous * count + next])
                    * at(&settings.project.print.flush_multiplier.0, nozzle).0)
                    as f32;
                purge = wiping.mark(&layer, (current, next), purge);
            }
            let grab = (at(&settings.printer.remaining.grab_length.0, nozzle).0 * 2.4) as f32;
            purge = (purge - grab).max(0.0);
            let prime = at(&config.filament_prime_volume.0, next).0 as f32;
            plan.add(position, (current, next), prime, purge);
            current = next;
            nozzle_filaments[nozzle] = Some(current);
        }
        wiping.ensure_order(&layer);
        tools.wiping = wiping;
        if plan.wrapping || plan.smooth {
            if tools.wipe_tower_partitions == 0 {
                plan.layers.last_mut().unwrap().extruder_fill = false;
            }
            continue;
        }
        if layers
            .peek()
            .is_none_or(|next| next.wipe_tower_partitions == 0)
        {
            break;
        }
    }
    let runtime = &traversal.resolved.views.runtime.project.gcode;
    let filaments = (0..count)
        .map(|index| {
            Filament::new(
                settings,
                index,
                (
                    at(&runtime.retraction_length.0, index).0 as f32,
                    at(&runtime.retraction_speed.0, index).0 as f32,
                ),
            )
        })
        .collect();
    Planned {
        plan,
        filaments,
        first_tool,
        used: used.into_iter().collect(),
    }
}
