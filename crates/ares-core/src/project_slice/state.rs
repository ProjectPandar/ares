use crate::{
    Project, SliceError, load_project,
    options::{is_bambu_project, write_config_block},
    project::effective_config::{
        resolve_bounded_project_config, types::BoundedResolvedProjectConfig,
    },
};

use super::{layers::PlannedPrintObject, plan_project};

pub(super) struct ProjectSliceState {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) planned_objects: Vec<PlannedPrintObject>,
}

pub(super) fn prepare_project_slice(
    project: impl AsRef<[u8]>,
) -> Result<ProjectSliceState, SliceError> {
    let project = load_project(project)?;
    let resolved = resolve_bounded_project_config(&project)?;
    let config_block = if is_bambu_project(&resolved.views.full) {
        let mut block = Vec::new();
        write_config_block(&resolved.views, 0, &mut block)?;
        Some(block)
    } else {
        None
    };
    let planned_objects = plan_project(&project, &resolved)?;
    Ok(ProjectSliceState {
        project,
        resolved,
        config_block,
        planned_objects,
    })
}
