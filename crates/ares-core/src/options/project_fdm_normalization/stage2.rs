use super::{ProjectFdmNormalizationKey, ProjectSettings};
use crate::options::{OrcaBool, ProcessPrintSequence, ProcessTimelapseType};

pub(super) fn normalize(
    settings: &mut ProjectSettings,
    num_objects: usize,
    used_filaments: usize,
) -> Vec<ProjectFdmNormalizationKey> {
    if used_filaments == 0 {
        return Vec::new();
    }

    let wrapping_enabled = settings.process.gcode.enable_wrapping_detection.0;
    let print = &mut settings.process.print;
    let disable_tower = print.timelapse_type != ProcessTimelapseType::Smooth
        && !wrapping_enabled
        && (used_filaments == 1
            || (print.print_sequence == ProcessPrintSequence::ByObject && num_objects > 1));

    if disable_tower && print.enable_prime_tower.0 {
        print.enable_prime_tower = OrcaBool(false);
        return vec![ProjectFdmNormalizationKey::EnablePrimeTower];
    }

    if print.enable_prime_tower.0 && print.independent_support_layer_height.0 {
        print.independent_support_layer_height = OrcaBool(false);
        return vec![ProjectFdmNormalizationKey::IndependentSupportLayerHeight];
    }

    Vec::new()
}
