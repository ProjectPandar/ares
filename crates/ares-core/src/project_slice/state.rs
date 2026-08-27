use crate::{
    Project, SliceError,
    geometry::CoordinateScale,
    load_project,
    options::write_config_block,
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

fn first_plate_id(project: &Project) -> u32 {
    project
        .plates()
        .iter()
        .map(|plate| plate.id())
        .min()
        .unwrap_or(1)
}

pub(super) fn prepare_project_slice(
    project: impl AsRef<[u8]>,
    plate: Option<u32>,
) -> Result<ProjectSliceState, SliceError> {
    let project = load_project(project)?;
    let plate_id = plate.unwrap_or_else(|| first_plate_id(&project));
    let project = project.select_plate(plate_id)?;
    let resolved = resolve_bounded_project_config(&project)?;
    // OrcaSlicer writes the config block into every sliced G-code and uses it
    // as the machine/layer template placeholder source (`GCode.cpp` config
    // export), not only for Bambu Lab printers.
    let mut config_block = Vec::new();
    write_config_block(&resolved.views, 0, &mut config_block)?;
    let config_block = (!config_block.is_empty()).then_some(config_block);
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
