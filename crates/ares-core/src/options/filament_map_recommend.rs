//! `ToolOrdering::get_recommended_filament_maps` — the auto filament-map
//! assignment for `filament_map_mode < Manual`
//! (`ToolOrdering.cpp:1107-1215`), applied to the resolved config the way
//! `Print::update_filament_maps_to_config` (`Print.cpp:3166`) updates
//! `m_config` before G-code export.
//!
//! The full `FilamentGroup::calc_filament_group` flush optimizer
//! (`FilamentGroup.cpp`) is deferred: this port covers the branches the
//! printer smoke corpus exercises — single-nozzle (every filament maps to
//! `master_extruder_id`), non-BBL multi-nozzle (filament i → extruder i),
//! and dual-nozzle BBL where the initial `master_extruder_id` vector is the
//! result for single-filament prints.

use crate::{OrcaInt, ProjectFilamentMapMode, ProjectSettings};

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
    let master = settings.printer.gcode.master_extruder_id.0;
    let map = if nozzle_count == 1 || (nozzle_count == 2 && is_bbl) {
        vec![OrcaInt(master); filament_count]
    } else if nozzle_count > 1 {
        (1..=filament_count as i32).map(OrcaInt).collect()
    } else {
        return;
    };
    settings.project.gcode.filament_map.0 = map;
}
