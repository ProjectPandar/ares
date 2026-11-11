//! `ToolOrdering::get_recommended_filament_maps` — the auto filament-map
//! assignment for `filament_map_mode < Manual`
//! (`ToolOrdering.cpp:1107-1215`), applied to the resolved config the way
//! `Print::update_filament_maps_to_config` (`Print.cpp:3166`) updates
//! `m_config` before G-code export.
//!
//! BBL multi-head grouping runs after actual layer usage is available in
//! `gcode_emit::tool_ordering::recommend`. This early phase preserves its
//! input map; it only resolves the independent single-head/non-BBL routes.

use crate::{OrcaInt, ProjectFilamentMapMode, ProjectSettings};

/// OrcaSlicer.cpp:6937–6969: CLI auto-map preparation replaces per-head AMS counts.
pub(crate) fn prepare_multi_extruder_cli_defaults(settings: &mut ProjectSettings) {
    let count = settings.project.print.nozzle_diameter.0.len();
    if count <= 1 || settings.project.gcode.filament_map_mode == ProjectFilamentMapMode::Manual {
        return;
    }
    settings.project.gcode.extruder_ams_count.0 = vec!["1#0|4#1".to_owned(); count];
}

pub(crate) fn apply_recommended_filament_map(settings: &mut ProjectSettings) {
    if settings.project.gcode.filament_map_mode == ProjectFilamentMapMode::Manual {
        return;
    }
    let filament_count = settings.filament.gcode.filament_colour.0.len().max(1);
    let nozzle_count = settings.project.print.nozzle_diameter.0.len();
    let is_bbl = settings
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab");
    if nozzle_count > 1 && is_bbl {
        return;
    }
    let master = settings.printer.gcode.master_extruder_id.0;
    let map = if nozzle_count == 1 {
        vec![OrcaInt(master); filament_count]
    } else if nozzle_count > 1 {
        (1..=filament_count as i32).map(OrcaInt).collect()
    } else {
        return;
    };
    settings.project.gcode.filament_map.0 = map;
}

/// ToolOrdering.cpp:1171-1173,1200-1206 and FilamentGroup.cpp:40-50.
/// Used IDs have already passed collect_extruders and handle_dontcare.
pub(crate) fn apply_used_tpu_map(
    settings: &mut ProjectSettings,
    used: &[usize],
    is_bbl: bool,
) -> bool {
    if settings.project.gcode.filament_map_mode == ProjectFilamentMapMode::Manual
        || settings.project.print.nozzle_diameter.0.len() != 2
        || !is_bbl
    {
        return false;
    }
    let types = &settings.filament.gcode.filament_type.0;
    let tpu = used
        .iter()
        .copied()
        .filter(|&id| types.get(id).unwrap_or(&types[0]) == "TPU")
        .collect::<Vec<_>>();
    if tpu.is_empty() {
        return false;
    }
    let master = settings.printer.gcode.master_extruder_id.0;
    let mut map = vec![OrcaInt(3 - master); settings.filament.gcode.filament_colour.0.len()];
    for id in tpu {
        map[id] = OrcaInt(master);
    }
    if settings.project.gcode.filament_map.0 == map {
        return false;
    }
    settings.project.gcode.filament_map.0 = map;
    true
}

/// ToolOrdering.cpp:67-82,1304-1308: check used filaments only, on BBL only.
pub(crate) fn check_filament_printable_after_group(
    settings: &ProjectSettings,
    used: &[usize],
    is_bbl: bool,
) -> Result<(), String> {
    if !is_bbl {
        return Ok(());
    }
    let filament = &settings.filament.gcode;
    for &id in used {
        let nozzle = settings.project.gcode.filament_map.0[id].0 - 1;
        // ConfigOptionVector::get_at falls back to the first value beyond its vector.
        let status = filament
            .filament_printable
            .0
            .get(id)
            .unwrap_or(&filament.filament_printable.0[0])
            .0;
        if status >> nozzle & 1 == 0 {
            let kind = filament
                .filament_type
                .0
                .get(id)
                .unwrap_or(&filament.filament_type.0[0]);
            let name = if nozzle == 0 { "left" } else { "right" };
            return Err(format!(
                "Grouping error: {kind} cannot be placed in the {name} nozzle"
            ));
        }
    }
    Ok(())
}
