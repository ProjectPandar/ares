//! `Print.cpp:3572–3654`: Type2 purge allocation and native depth-plan input.
use super::super::tool_ordering::{
    ToolOrdering,
    wiping::{self, Layer, WipingExtrusions},
};
use super::{
    filament::{Filament, at},
    plan::Plan,
};
use crate::project_slice::{
    extrusion_islands::PreparedPostExtrusionIslands,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};

pub(in crate::project_slice::gcode_emit) struct Planned {
    pub(super) plan: Plan,
    pub(super) filaments: Vec<Filament>,
    pub(super) matrix: Vec<Vec<f32>>,
    pub(super) first_tool: usize,
}

pub(in crate::project_slice::gcode_emit) fn prepare(
    traversal: &PreparedPostClassicTraversal,
    original: &PreparedPostExtrusionIslands,
    support: &[Vec<InstalledSupportLayer>],
    ordering: &mut ToolOrdering,
) -> Option<Planned> {
    if ordering.tower_type != Some(crate::WipeTowerType::Type2) {
        return None;
    }
    let settings = &traversal.resolved.views.full;
    let config = &settings.filament.gcode;
    let count = config.filament_colour.0.len();
    let retraction_speed = &traversal
        .resolved
        .views
        .runtime
        .project
        .gcode
        .retraction_speed
        .0;
    let filaments = (0..count)
        .map(|tool| Filament::from_config(settings, tool, at(retraction_speed, tool).0 as f32))
        .collect::<Vec<_>>();
    let purge = settings.printer.gcode.single_extruder_multi_material.0
        && settings.printer.gcode.purge_in_prime_tower.0;
    let prime = settings.process.print.prime_volume.0 as f32;
    let matrix = (0..count)
        .map(|row| {
            (0..count)
                .map(|column| {
                    let volume =
                        settings.project.print.flush_volumes_matrix.0[row * count + column] as f32;
                    if !purge && volume > 0.0 {
                        prime
                    } else {
                        volume
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    // Tower interface selection must not change the allocation's soluble flags.
    let soluble = (0..count)
        .map(|tool| at(&config.filament_soluble.0, tool).0)
        .collect::<Vec<_>>();
    let support_filaments = (0..count)
        .map(|tool| at(&config.filament_is_support.0, tool).0)
        .collect::<Vec<_>>();
    let first_tool = ordering
        .first_extruder
        .expect("active tower has a first tool");
    let primed_tool = *ordering
        .all_extruders()
        .last()
        .expect("active tower has printing tools");
    let mut current = primed_tool;
    let mut plan = Plan::new(settings, &filaments);
    let mut layers = ordering.layers.iter_mut().enumerate().peekable();
    while let Some((index, tools)) = layers.next() {
        if !tools.has_wipe_tower {
            continue;
        }
        let objects = wiping::objects(traversal, original, support, tools.print_z);
        let layer = Layer {
            objects: &objects,
            tools: &tools.extruders,
            soluble: &soluble,
            support_filaments: &support_filaments,
        };
        let mut wiping = WipingExtrusions::new(&layer);
        let position = (tools.print_z as f32, tools.wipe_tower_layer_height as f32);
        plan.add(position, (current, current), 0.0, &filaments);
        for &next in &tools.extruders {
            if next == current && !(index == 0 && next == primed_tool) {
                continue;
            }
            let mut volume = prime;
            if purge {
                volume = (f64::from(matrix[current][next])
                    * settings.project.print.flush_multiplier.0[0].0)
                    as f32;
                volume -= filaments[next].minimal_purge;
                volume = wiping.mark(&layer, (current, next), volume);
                volume += filaments[next].minimal_purge;
            }
            plan.add(position, (current, next), volume, &filaments);
            current = next;
        }
        wiping.ensure_order(&layer);
        tools.wiping = wiping;
        if layers
            .peek()
            .is_none_or(|(_, next)| next.wipe_tower_partitions == 0)
        {
            break;
        }
    }
    Some(Planned {
        plan,
        filaments,
        matrix,
        first_tool,
    })
}
