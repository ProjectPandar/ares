use crate::{
    Project, SliceError,
    geometry::CoordinateScale,
    options::write_config_block,
    project::effective_config::{
        resolve_bounded_project_config, types::BoundedResolvedProjectConfig,
    },
};

mod painted_fuzzy;
mod print_volume;
pub(crate) use print_volume::global_bed_points;

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

#[cfg(test)]
pub(super) fn prepare_project_slice(
    project: impl AsRef<[u8]>,
    plate: Option<u32>,
    overrides: &crate::ConfigOverrides,
) -> Result<ProjectSliceState, SliceError> {
    let mut project = crate::project::load_project_with(project, overrides)?;
    let filament_count = project.cli_filament_count();
    project.initialize_cli_loaded_project_config(filament_count);
    prepare_loaded_project_slice(project, plate)
}

pub(super) fn prepare_project_source(
    source: super::ProjectSource<'_>,
) -> Result<ProjectSliceState, SliceError> {
    let (mut project, preparation) =
        crate::project::load_project_with_preparation(source.bytes, source.overrides)?;
    if source.validate_print {
        super::validated::validate_post_process(&project)?;
    }
    // `OrcaSlicer.cpp:2653-2654`: the plain-3MF compatible-printers gate
    // fires after load and before parameter validation.
    super::compatibility::reject_incompatible_process(&project.documents().project_settings_raw)?;
    project.settings().validate_loaded_config(
        &project.documents().project_settings_raw,
        source.validate_print,
    )?;
    let filament_count = project.cli_filament_count();
    project.initialize_cli_loaded_project_config(filament_count);
    // CLI:2656 commits the original-machine report before plate selection
    // and Print::validate. Early load/config failures above cannot leak it.
    if let Some(upward) = source.upward {
        *upward.report = match upward.source {
            Some(original) => original.select(upward.input)?,
            None => Vec::new(),
        };
    }
    prepare_loaded_project_slice_with_validation(
        project,
        source.plate,
        None,
        source.validate_print,
        Some((preparation, source.overrides)),
    )
}

#[cfg(test)]
pub(super) fn prepare_loaded_project_slice(
    project: Project,
    plate: Option<u32>,
) -> Result<ProjectSliceState, SliceError> {
    prepare_loaded_project_slice_with_validation(project, plate, None, false, None)
}

pub(super) fn prepare_loaded_project_slice_with_validation(
    project: Project,
    plate: Option<u32>,
    selected_printer: Option<&str>,
    validate_print: bool,
    preparation: Option<(
        crate::options::flush_matrix::PreparationContext,
        &crate::ConfigOverrides,
    )>,
) -> Result<ProjectSliceState, SliceError> {
    let project = project.select_plate(plate)?;
    let plate_id = project.first_plate_id();
    // `OrcaSlicer.cpp:5710-5724`: unconditional — if no model-part instance
    // is fully inside the plate's print volume, reject with the upstream
    // -50 text (`Model.cpp:692-702 update_print_volume_state`).
    if print_volume::count_inside_instances(&project)? == 0 {
        return Err(SliceError::NoSuitableObjects(
            "One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading."
                .to_owned(),
        ));
    }
    let resolved = match preparation {
        Some((context, overrides)) => crate::project::effective_config::phases::resolve_bounded_project_config_with_preparation(&project, &context, overrides)?,
        None => resolve_bounded_project_config(&project)?,
    };
    painted_fuzzy::validate(&project, &resolved)?;
    if validate_print {
        crate::print::validate::validate_applied_project(
            &project,
            &resolved,
            plate_id,
            selected_printer,
        )?;
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
        (
            &resolved.views.full.filament.region,
            resolved.logical_filament_count,
        ),
    )?;
    Ok(ProjectSliceState {
        project,
        resolved,
        config_block,
        scale,
        intersected_objects,
    })
}
