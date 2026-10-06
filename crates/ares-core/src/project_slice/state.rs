use crate::{
    Project, SliceError,
    geometry::CoordinateScale,
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

/// `[min_x, max_x, min_y, max_y, max_z]` from the project's `printable_area`
/// and `printable_height` (`BuildVolume::bounding_volume`).
fn print_volume(project: &Project) -> [f64; 5] {
    let printer = &project.settings().printer.remaining;
    let area = &printer.printable_area.0;
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for point in area {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }
    [min_x, max_x, min_y, max_y, printer.printable_height.0]
}

fn vertex_inside(point: crate::Point3d, volume: &[f64; 5]) -> bool {
    point.x >= volume[0]
        && point.x <= volume[1]
        && point.y >= volume[2]
        && point.y <= volume[3]
        && point.z >= 0.0
        && point.z <= volume[4]
}

/// `Model.cpp:2370-2379`: any instance whose model-part volumes are all
/// fully inside the print volume.
fn instance_inside(object: &crate::ProjectObject, volume: &[f64; 5]) -> bool {
    object.instances().iter().any(|instance| {
        let transform = instance.transform();
        object
            .volumes()
            .iter()
            .filter(|v| v.volume_type() == crate::ProjectVolumeType::ModelPart)
            .any(|vol| {
                let matrix = transform.then(vol.transform());
                vol.mesh()
                    .vertices()
                    .iter()
                    .all(|vertex| vertex_inside(matrix.transform_point(*vertex), volume))
            })
    })
}

fn first_plate_id(project: &Project) -> u32 {
    project
        .plates()
        .iter()
        .map(|plate| plate.id())
        .min()
        .unwrap_or(1)
}

#[cfg(test)]
pub(super) fn prepare_project_slice(
    project: impl AsRef<[u8]>,
    plate: Option<u32>,
    overrides: &crate::ConfigOverrides,
) -> Result<ProjectSliceState, SliceError> {
    let mut project = crate::project::load_project_with(project, overrides)?;
    project.initialize_cli_loaded_project_config();
    prepare_loaded_project_slice(project, plate)
}

pub(super) fn prepare_project_source(
    source: super::ProjectSource<'_>,
) -> Result<ProjectSliceState, SliceError> {
    let mut project = crate::project::load_project_with(source.bytes, source.overrides)?;
    if source.validate_print {
        super::validated::validate_post_process(&project)?;
    }
    project.settings().validate_loaded_firmware_retraction()?;
    project.initialize_cli_loaded_project_config();
    prepare_loaded_project_slice_with_validation(project, source.plate, None, source.validate_print)
}

#[cfg(test)]
pub(super) fn prepare_loaded_project_slice(
    project: Project,
    plate: Option<u32>,
) -> Result<ProjectSliceState, SliceError> {
    prepare_loaded_project_slice_with_validation(project, plate, None, false)
}

pub(super) fn prepare_loaded_project_slice_with_validation(
    project: Project,
    plate: Option<u32>,
    selected_printer: Option<&str>,
    validate_print: bool,
) -> Result<ProjectSliceState, SliceError> {
    let plate_id = plate.unwrap_or_else(|| first_plate_id(&project));
    let project = project.select_plate(plate_id)?;
    // `OrcaSlicer.cpp:5710-5724`: if no model-part instance is fully
    // inside the plate's print volume, reject with the upstream -50 text
    // (`Model.cpp:692-702 update_print_volume_state`).
    // Only checked when unassigned instances are present: plate-assigned
    // instances were positioned by the 3MF's own plate metadata (upstream
    // always checks, but Ares profile overlays may change the bed after
    // the objects were placed for the original printer — a documented
    // bounded deviation for the profile entry path).
    let has_unassigned = project.objects().iter().any(|object| {
        object
            .instances()
            .iter()
            .any(|instance| instance.loaded_label_id().is_none())
    });
    let inside = !has_unassigned
        || project
            .objects()
            .iter()
            .any(|object| instance_inside(object, &print_volume(&project)));
    if !inside {
        return Err(SliceError::InvalidInput(
            "One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading."
                .to_owned(),
        ));
    }
    let resolved = resolve_bounded_project_config(&project)?;
    if validate_print {
        crate::print::validate::validate_applied_project(&project, &resolved, selected_printer)?;
    }
    // OrcaSlicer writes the config block into every sliced G-code and uses it
    // as the machine/layer template placeholder source (`GCode.cpp` config
    // export), not only for Bambu Lab printers.
    let mut config_block = Vec::new();
    write_config_block(
        &resolved.views,
        &project.documents().project_settings_raw,
        0,
        &mut config_block,
    )?;
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
