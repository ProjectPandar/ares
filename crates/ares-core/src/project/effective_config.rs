pub(crate) mod candidates;
mod cardinality;
pub(crate) mod grouping;
pub(crate) mod layers;
pub(crate) mod occupancy;
pub(crate) mod phases;
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
    let validated = cardinality::validate(settings)?;
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
