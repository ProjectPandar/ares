//! Adapts typed model sources to the existing PrintApply scarf-flag owner.

use crate::print_apply::apply_scarf_joint_seam_state::{
    StagedApplyScarfJointSeamObject, StagedSeamScarfType, staged_apply_scarf_joint_seam,
};
use crate::{OrcaBool, ProcessSeamScarfType, ProjectObject, ProjectSettings};

pub(crate) fn apply(settings: &mut ProjectSettings, objects: &[ProjectObject]) {
    let staged = objects
        .iter()
        .map(|object| {
            StagedApplyScarfJointSeamObject::new(
                staged_type(
                    object
                        .region_overrides()
                        .seam_slope_type
                        .unwrap_or(settings.process.region.seam_slope_type),
                ),
                object
                    .volumes()
                    .iter()
                    .map(|volume| volume.region_overrides().seam_slope_type.map(staged_type))
                    .collect(),
                object
                    .layer_config_ranges()
                    .iter()
                    .map(|range| range.region_overrides().seam_slope_type.map(staged_type))
                    .collect(),
            )
        })
        .collect::<Vec<_>>();
    if let Some(set) = staged_apply_scarf_joint_seam(&staged).config_set() {
        settings.project.gcode.has_scarf_joint_seam = OrcaBool(set.value());
    }
}

fn staged_type(value: ProcessSeamScarfType) -> StagedSeamScarfType {
    match value {
        ProcessSeamScarfType::None => StagedSeamScarfType::None,
        ProcessSeamScarfType::External => StagedSeamScarfType::External,
        ProcessSeamScarfType::All => StagedSeamScarfType::All,
    }
}
