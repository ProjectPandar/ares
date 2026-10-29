pub(crate) mod candidates;
mod cardinality;
pub(crate) mod grouping;
pub(crate) mod layers;
pub(crate) mod occupancy;
pub(crate) mod phases;
pub(crate) mod scarf_joint_seam;
mod selector_validation;
pub(crate) mod types;
pub(crate) mod usage;

use crate::{ProjectSettings, SliceError};

use super::Project;
use types::BoundedResolvedProjectConfig;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ValidatedMaterializedProject {
    pub(crate) physical_extruder_count: usize,
    pub(crate) logical_filament_count: usize,
}

pub(crate) fn validate_materialized_project(
    settings: &ProjectSettings,
    project: &Project,
) -> Result<ValidatedMaterializedProject, SliceError> {
    // `OrcaSlicer.cpp:3619,3627-3633`: check resolved technology before FFF work.
    if settings.printer.remaining.printer_technology != crate::PrinterTechnology::Fff {
        return Err(SliceError::InvalidPrinterTechnology);
    }
    let validated = cardinality::validate(settings)?;
    // MMS.cpp:2202 uses colour.size()+1; PrintObjectSlice.cpp:887 consumes diameter.size().
    if project
        .objects()
        .iter()
        .any(|source| source.is_mm_painted(validated.logical_filament_count))
        && settings.filament.gcode.filament_colour.0.len() != validated.logical_filament_count
    {
        return Err(invalid_option("filament_colour"));
    }
    selector_validation::validate(settings, project, validated)?;
    Ok(validated)
}

pub(crate) fn resolve_bounded_project_config(
    project: &Project,
) -> Result<BoundedResolvedProjectConfig, SliceError> {
    phases::resolve_bounded_project_config(project)
}

fn invalid_option(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid Orca option {key}"))
}
