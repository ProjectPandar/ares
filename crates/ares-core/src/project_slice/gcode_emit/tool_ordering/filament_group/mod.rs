//! Static ToolOrdering → FilamentGroup → Print::update_filament_maps_to_config.
mod context;
mod enumeration;
mod sequence;

use super::super::LayerTools;
use crate::options::{project_config_views::ProjectConfigViews, refresh_filament_map};
use crate::project::raw_settings::ProjectSettingsRaw;
use crate::project_slice::gcode_emit::wipe_tower_estimate::config::DynamicPrintConfig;
use crate::{OrcaInt, OrcaInts, ProjectFilamentMapMode, ProjectSettings, SliceError};
use std::collections::BTreeSet;

pub(super) fn check_scope(
    settings: &ProjectSettings,
    is_bbl: bool,
    used: &[usize],
    objects: usize,
    multiple_nozzles: bool,
) -> Result<bool, SliceError> {
    let automatic = (is_bbl || multiple_nozzles)
        && settings.project.print.nozzle_diameter.0.len() > 1
        && (settings.project.gcode.filament_map_mode != ProjectFilamentMapMode::Manual
            || multiple_nozzles)
        && !used.iter().any(|&id| {
            settings
                .filament
                .gcode
                .filament_type
                .0
                .get(id)
                .unwrap_or(&settings.filament.gcode.filament_type.0[0])
                == "TPU"
        });
    let sequential = settings.process.print.print_sequence == crate::ProcessPrintSequence::ByObject;
    if automatic
        && ((sequential && objects > 1) || settings.project.print.nozzle_diameter.0.len() != 2)
    {
        return Err(context::unsupported());
    }
    Ok(automatic)
}

pub(super) fn apply(
    checkpoint: &mut ProjectSettings,
    input: (&ProjectSettings, &ProjectSettingsRaw),
    layers: &[LayerTools],
    geometry: Vec<BTreeSet<usize>>,
    config: DynamicPrintConfig,
) -> Result<ProjectConfigViews, SliceError> {
    let (current, raw) = input;
    let context = context::Context::build((current, checkpoint, raw), layers, geometry, config)?;
    let mut result = if context.manual_multi_nozzle {
        current
            .project
            .gcode
            .filament_map
            .0
            .iter()
            .map(|id| usize::try_from(id.0 - 1).expect("validated filament map"))
            .collect()
    } else {
        enumeration::solve(&context)
    };
    if !context.manual_multi_nozzle {
        // ToolOrdering.cpp:1681–1747: post-AMS master preference is not a solver tie-break.
        let master = context.master;
        let other = 1 - master;
        let mut occupancy = [0; 2];
        for &id in &context.used {
            occupancy[result[id]] += 1;
        }
        let printable_swap = context
            .used
            .iter()
            .all(|id| !context.limits[1 - result[*id]].contains(id));
        let capacity_swap = !(occupancy[0] <= context.capacity[0]
            && occupancy[1] <= context.capacity[1]
            && (occupancy[1] > context.capacity[0] || occupancy[0] > context.capacity[1]));
        if occupancy[other] > occupancy[master] && printable_swap && capacity_swap {
            for &id in &context.used {
                result[id] = 1 - result[id];
            }
        }
    }
    let mut refreshed = current.clone();
    let mut heads = sized_seed(&current.project.gcode.filament_map, context.total, 1);
    let mut volumes = sized_seed(&current.project.gcode.filament_volume_map, context.total, 0);
    for &id in &context.used {
        heads.0[id] = OrcaInt(result[id] as i32 + 1);
        volumes.0[id] = OrcaInt(context.volumes[result[id]]);
    }
    let nozzle_offsets = context
        .nozzle_counts
        .iter()
        .scan(0usize, |offset, count| {
            let current = *offset;
            *offset += *count;
            Some(current)
        })
        .collect::<Vec<_>>();
    let nozzles = OrcaInts(
        result
            .iter()
            .map(|head| {
                OrcaInt(if context.manual_multi_nozzle {
                    nozzle_offsets[*head] as i32
                } else {
                    *head as i32
                })
            })
            .collect(),
    );
    if current.project.gcode.filament_map != heads
        || current.project.gcode.filament_volume_map != volumes
        || current.project.gcode.filament_nozzle_map != nozzles
    {
        for config in [&mut checkpoint.project.gcode, &mut refreshed.project.gcode] {
            config.filament_map.clone_from(&heads);
            config.filament_volume_map.clone_from(&volumes);
            config.filament_nozzle_map.clone_from(&nozzles);
        }
    }
    let mut views = refresh_filament_map(
        checkpoint,
        &refreshed.project.gcode.filament_map,
        &refreshed,
    )?;
    if context.manual_multi_nozzle {
        views.full.project.gcode.filament_nozzle_map = nozzles.clone();
        views.runtime.project.gcode.filament_nozzle_map = nozzles.clone();
        views.runtime_gcode.filament_nozzle_map = nozzles;
    }
    Ok(views)
}

fn sized_seed(seed: &OrcaInts, count: usize, default: i32) -> OrcaInts {
    if seed.0.len() == count {
        seed.clone()
    } else {
        OrcaInts(vec![OrcaInt(default); count])
    }
}
