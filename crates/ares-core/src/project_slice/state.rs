use crate::{
    Project, SliceError,
    geometry::CoordinateScale,
    load_project,
    options::{is_bambu_project, write_config_block},
    project::effective_config::{
        resolve_bounded_project_config, types::BoundedResolvedProjectConfig,
    },
};

use super::{
    planning::plan_project,
    raw_intersections::{
        IntersectedPrintObject, intersect_projected_objects, prepare_projected_objects,
    },
};

pub(super) struct ProjectSliceState {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) scale: CoordinateScale,
    pub(super) intersected_objects: Vec<IntersectedPrintObject>,
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
    let projected_objects =
        prepare_projected_objects(project.objects(), &resolved.objects, planned_objects)?;
    let scale =
        CoordinateScale::from_printable_area(&resolved.views.full.printer.remaining.printable_area);
    let intersected_objects = intersect_projected_objects(
        project.objects(),
        &resolved.objects,
        projected_objects,
        scale,
    )?;
    Ok(ProjectSliceState {
        project,
        resolved,
        config_block,
        scale,
        intersected_objects,
    })
}
