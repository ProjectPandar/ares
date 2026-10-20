//! `PrintApply.cpp:1148-1158`: one owner for the effective scarf flag.

use crate::{OrcaBool, ProcessSeamScarfType, ProjectObject, ProjectSettings};

pub(crate) fn apply(settings: &mut ProjectSettings, objects: &[ProjectObject]) {
    let non_none = |value| value != ProcessSeamScarfType::None;
    let enabled = settings.project.gcode.has_scarf_joint_seam.0
        || objects.iter().any(|object| {
            non_none(
                object
                    .region_overrides()
                    .seam_slope_type
                    .unwrap_or(settings.process.region.seam_slope_type),
            ) || object.volumes().iter().any(|volume| {
                volume
                    .region_overrides()
                    .seam_slope_type
                    .is_some_and(non_none)
            }) || object.layer_config_ranges().iter().any(|range| {
                range
                    .region_overrides()
                    .seam_slope_type
                    .is_some_and(non_none)
            })
        });
    settings.project.gcode.has_scarf_joint_seam = OrcaBool(enabled);
}
