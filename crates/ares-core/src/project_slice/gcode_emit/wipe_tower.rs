//! `Print::has_wipe_tower` and `Print::wipe_tower_type` activation.

mod data;
mod emit;
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
mod runtime;
mod type1;
pub(super) use runtime::{finalize, initialize, tool_change};
mod tool_change;
mod unload;
mod wipe;
mod writer;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smooth_activation_preserves_source_type_dispatch_and_enable_flag() {
        let mut settings = crate::ProjectSettings::default();
        settings.process.print.enable_prime_tower.0 = true;
        settings.process.print.timelapse_type = crate::ProcessTimelapseType::Smooth;
        settings.printer.gcode.wipe_tower_type = crate::WipeTowerType::Type2;
        assert_eq!(
            requested(&settings, false),
            Some(crate::WipeTowerType::Type2)
        );
        assert_eq!(
            requested(&settings, true),
            Some(crate::WipeTowerType::Type1)
        );
        settings.printer.gcode.wipe_tower_type = crate::WipeTowerType::Type1;
        assert_eq!(
            requested(&settings, false),
            Some(crate::WipeTowerType::Type1)
        );
        settings.printer.gcode.wipe_tower_type = crate::WipeTowerType::Type2;
        settings.process.print.enable_prime_tower.0 = false;
        assert_eq!(requested(&settings, false), None);
    }
}
