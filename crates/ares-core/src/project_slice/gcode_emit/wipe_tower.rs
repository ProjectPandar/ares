//! `Print::has_wipe_tower` and `Print::wipe_tower_type` activation.

pub(super) fn type2_requested(settings: &crate::ProjectSettings, bbl: bool) -> bool {
    let print = &settings.process.print;
    !bbl && settings.printer.gcode.wipe_tower_type == crate::WipeTowerType::Type2
        && print.enable_prime_tower.0
        && ((settings.process.gcode.enable_wrapping_detection.0
            && settings.printer.gcode.wrapping_exclude_area.0.len() > 2)
            || print.timelapse_type == crate::ProcessTimelapseType::Smooth
            || (!print.spiral_mode.0 && settings.filament.gcode.filament_diameter.0.len() > 1))
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
        assert!(type2_requested(&settings, false));
        assert!(!type2_requested(&settings, true));
        settings.printer.gcode.wipe_tower_type = crate::WipeTowerType::Type1;
        assert!(!type2_requested(&settings, false));
        settings.printer.gcode.wipe_tower_type = crate::WipeTowerType::Type2;
        settings.process.print.enable_prime_tower.0 = false;
        assert!(!type2_requested(&settings, false));
    }
}
