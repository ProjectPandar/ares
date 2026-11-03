//! `Print::has_wipe_tower` and `Print::wipe_tower_type` activation.

mod data;
mod emit;
pub(crate) mod estimate;
mod extents;
mod fake;
pub(super) use data::Data;
pub(super) use extents::Print as PrintExtents;
mod filament;
mod finish_layer;
mod generate;
mod generator;
mod integration;
mod interface_temperature;
pub(super) use integration::Integration;
mod geometry;
mod occupancy;
mod plan;
mod planning;
mod prepare;
pub(super) use prepare::prepare;
mod postprocess;
mod prime;
mod result;
mod route;
mod runtime;
mod type1;
pub(super) use runtime::{finalize, initialize, tool_change};
mod tool_change;
mod tower_gap;
mod unload;
mod wipe;
mod writer;

/// `wipe_tower_x.get_at(plate_idx)`, `wipe_tower_y.get_at(plate_idx)`: the sliced plate's stored
/// tower position (`GCode.cpp:7270`, `GCode/WipeTower2.cpp:1046` at v2.5.0-beta `345efec64`);
/// `get_at` takes entry 0 past the end.
pub(super) fn plate_position(
    traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
) -> [f64; 2] {
    let print = &traversal.resolved.views.full.project.print;
    let plate = traversal.project.current_plate_index();
    let at = |values: &[crate::OrcaFloat]| values.get(plate).unwrap_or(&values[0]).0;
    [at(&print.wipe_tower_x.0), at(&print.wipe_tower_y.0)]
}

pub(super) fn requested(
    settings: &crate::ProjectSettings,
    bbl: bool,
) -> Option<crate::WipeTowerType> {
    let print = &settings.process.print;
    let enabled = print.enable_prime_tower.0
        && ((settings.process.gcode.enable_wrapping_detection.0
            && settings.printer.gcode.wrapping_exclude_area.0.len() > 2)
            || print.timelapse_type == crate::ProcessTimelapseType::Smooth
            || (!print.spiral_mode.0 && settings.filament.gcode.filament_diameter.0.len() > 1));
    enabled.then_some(if bbl {
        crate::WipeTowerType::Type1
    } else {
        settings.printer.gcode.wipe_tower_type
    })
}
