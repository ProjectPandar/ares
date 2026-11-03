//! Static ToolOrdering.cpp:1752–1776,1869–1908 → Print.cpp:4656–4745.
use crate::{
    SliceError,
    options::{
        NozzleVolumeType, OrcaInt, OrcaInts, ProjectSettings,
        project_config_views::ProjectConfigViews,
    },
    project::raw_settings::ProjectSettingsRaw,
};

use super::index;

pub(crate) fn update_static_filament_maps(
    checkpoint: &mut ProjectSettings,
    current: &ProjectSettings,
    raw: &ProjectSettingsRaw,
    used: &[usize],
) -> Result<Option<ProjectConfigViews>, SliceError> {
    let multiple_nozzles = raw
        .get("extruder_max_nozzle_count")
        .and_then(|v| v.token())
        .is_some_and(|wire| {
            wire.split(';')
                .any(|v| v == "nil" || v.parse::<i32>().is_ok_and(|count| count > 1))
        });
    let mixed = raw
        .get("filament_is_mixed")
        .and_then(|v| v.token())
        .is_some_and(|wire| wire.split(';').any(|v| matches!(v, "1" | "true")));
    if current.project.print.nozzle_diameter.0.len() != 1 || multiple_nozzles || mixed {
        return Ok(None);
    }
    let volume = match current
        .project
        .gcode
        .nozzle_volume_type
        .0
        .first()
        .copied()
        .unwrap_or(NozzleVolumeType::Standard)
    {
        NozzleVolumeType::Standard => 0,
        NozzleVolumeType::HighFlow => 1,
    };
    if index::resolve_activation(current)?.is_some() {
        return Ok(None);
    }
    // LayeredNozzleGroupResult::create stores the complete 0-based map.
    let assignments: Vec<i32> = match current.project.gcode.filament_map_mode {
        crate::ProjectFilamentMapMode::Manual => current
            .project
            .gcode
            .filament_map
            .0
            .iter()
            .map(|id| id.0 - 1)
            .collect(),
        crate::ProjectFilamentMapMode::AutoForFlush
        | crate::ProjectFilamentMapMode::AutoForMatch => {
            vec![
                current.printer.gcode.master_extruder_id.0 - 1;
                current.filament.gcode.filament_colour.0.len()
            ]
        }
    };
    if assignments.is_empty() {
        return Ok(None);
    }
    // MultiNozzleUtils.cpp:316–369: one nozzle, group_id/extruder_id0.
    let derived_extruders = OrcaInts(
        assignments
            .iter()
            .map(|id| OrcaInt(if *id == 0 { 1 } else { 0 }))
            .collect(),
    );
    let derived_volumes = OrcaInts(
        assignments
            .iter()
            .map(|id| OrcaInt(if *id == 0 { volume } else { -1 }))
            .collect(),
    );
    let nozzles = OrcaInts(
        assignments
            .iter()
            .map(|id| OrcaInt(if *id == 0 { 0 } else { -1 }))
            .collect(),
    );
    let count = assignments.len();
    let extruder_base = sized_base(&current.project.gcode.filament_map, count, 1);
    let volume_base = sized_base(&current.project.gcode.filament_volume_map, count, 0);
    let extruders = update_used_filament_values(extruder_base, &derived_extruders, used);
    let volumes = update_used_filament_values(volume_base, &derived_volumes, used);
    let mut refreshed = current.clone();
    // Print.cpp:4658 compares all three maps, not only the extruder assignment.
    if current.project.gcode.filament_map != extruders
        || current.project.gcode.filament_volume_map != volumes
        || current.project.gcode.filament_nozzle_map != nozzles
    {
        for config in [&mut checkpoint.project.gcode, &mut refreshed.project.gcode] {
            config.filament_map.clone_from(&extruders);
            config.filament_volume_map.clone_from(&volumes);
            config.filament_nozzle_map.clone_from(&nozzles);
        }
    }
    crate::options::project_config_views::resolve_project_config_views(refreshed).map(Some)
}

fn sized_base(values: &OrcaInts, count: usize, default: i32) -> OrcaInts {
    if values.0.len() == count {
        values.clone()
    } else {
        OrcaInts(vec![OrcaInt(default); count])
    }
}

// FilamentGroupUtils.cpp:346–358 preserves unused entries and source bounds.
fn update_used_filament_values(mut base: OrcaInts, derived: &OrcaInts, used: &[usize]) -> OrcaInts {
    for &index in used {
        if let (Some(value), Some(replacement)) = (base.0.get_mut(index), derived.0.get(index)) {
            *value = *replacement;
        }
    }
    base
}
