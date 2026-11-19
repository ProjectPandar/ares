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
pub(crate) use print_volume::{
    BedShape, Circle, PrintVolume, global_bed_points, inside_triangle_count,
    instance_partly_outside,
};

use super::{
    looped_intersections::LoopedPrintObject,
    planning::plan_project,
    raw_intersections::{prepare_projected_objects, slice_projected_objects},
};

pub(super) struct ProjectSliceState {
    pub(super) project: Project,
    pub(super) resolved: BoundedResolvedProjectConfig,
    pub(super) config_block: Option<Vec<u8>>,
    pub(super) scale: CoordinateScale,
    pub(super) looped_objects: Vec<LoopedPrintObject>,
}

pub(super) fn prepare_project_source(
    source: super::ProjectSource<'_>,
) -> Result<ProjectSliceState, SliceError> {
    let plate = source.plate;
    let validate_print = source.validate_print;
    let overrides = source.overrides;
    let (project, preparation) = load_project_source(source)?;
    prepare_loaded_project_slice_with_validation(
        project,
        plate,
        None,
        validate_print,
        Some((&preparation, overrides)),
    )
}

type Preparation = crate::options::flush_matrix::PreparationContext;

pub(super) fn load_project_source(
    source: super::ProjectSource<'_>,
) -> Result<(Project, Preparation), SliceError> {
    let (mut project, preparation) = source.input.load(
        source.overrides,
        source.refresh,
        source.system_catalog,
        source.validate_print,
    )?;
    project.apply_plater_edits(source.edits);
    if let Some(refresh) = source.refresh {
        refresh.place(&mut project);
    }
    if source.validate_print {
        super::validated::validate_post_process(&project)?;
    }
    // `OrcaSlicer.cpp:2653-2654`: the plain-3MF compatible-printers gate
    // fires after load and before parameter validation. It belongs to the
    // CLI main (`:2575-2620`); the plater's selection has no such check.
    if !source.refresh.is_some_and(|refresh| refresh.is_plater()) {
        super::compatibility::reject_incompatible_process(
            &project.documents().project_settings_raw,
        )?;
    }
    // OrcaSlicer.cpp:3005 commits the report before validate(true):4272–4278.
    // Earlier load/post-process/compatibility failures do not reach this snapshot.
    if let Some(upward) = source.upward {
        *upward.report = match upward.source {
            Some(original) => original.select(upward.input)?,
            None => Vec::new(),
        };
    }
    super::capabilities::validate_settings(project.settings())?;
    if source.validate_config {
        project.settings().validate_loaded_config(
            &project.documents().project_settings_raw,
            source.validate_print,
        )?;
    }
    if matches!(source.input, super::ProjectInput::Bytes(_)) {
        let filament_count = project.cli_filament_count();
        project.initialize_cli_loaded_project_config(filament_count);
    }
    if source.orient {
        crate::plater::apply_cli_orientation(&mut project, source.plate)?;
    }
    Ok((project, preparation))
}

pub(super) fn prepare_loaded_project_slice_with_validation(
    project: Project,
    plate: Option<u32>,
    selected_printer: Option<&str>,
    validate_print: bool,
    preparation: Option<(&Preparation, &crate::ConfigOverrides)>,
) -> Result<ProjectSliceState, SliceError> {
    super::capabilities::validate_settings(project.settings())?;
    let project = crate::plater::initialize_slice_plates(project);
    let project = crate::plater::fit_slice_wipe_tower(project, plate)?;
    apply_plate(
        project.select_plate(plate)?,
        selected_printer,
        validate_print,
        preparation,
    )?
    .prepare()
}

pub(super) struct AppliedPlate {
    project: Project,
    resolved: BoundedResolvedProjectConfig,
}

pub(super) fn apply_plate(
    mut project: Project,
    selected_printer: Option<&str>,
    validate_print: bool,
    preparation: Option<(&Preparation, &crate::ConfigOverrides)>,
) -> Result<AppliedPlate, SliceError> {
    let plate_id = project.first_plate_id();
    // `OrcaSlicer.cpp:6562-6573`: unconditional — if no model-part instance
    // is fully inside the plate's print volume, reject with the upstream
    // -50 text (`Model.cpp:839-852 update_print_volume_state`; text `OrcaSlicer.cpp:220`).
    if print_volume::update_print_volume_state(&mut project)? == 0 {
        return Err(no_printable_objects());
    }
    let resolved = match preparation {
        Some((context, overrides)) => crate::project::effective_config::phases::resolve_bounded_project_config_with_preparation(&project, context, overrides)?,
        None => resolve_bounded_project_config(&project)?,
    };
    super::capabilities::validate_settings(&resolved.views.full)?;
    painted_fuzzy::validate(&project, &resolved)?;
    super::seam_placement::validate_paint(&project, &resolved)?;
    if validate_print {
        crate::print::validate::validate_applied_project(
            &project,
            &resolved,
            plate_id,
            selected_printer,
        )?;
    }
    // OrcaSlicer.cpp:7047–7055 checks `Print::empty()` after apply and
    // validation, including in the all-plates precheck. Volume membership
    // alone can count an instance whose build item is not printable.
    if resolved
        .objects
        .iter()
        .all(|object| object.print_objects.is_empty())
    {
        return Err(no_printable_objects());
    }
    Ok(AppliedPlate { project, resolved })
}

fn no_printable_objects() -> SliceError {
    SliceError::NoSuitableObjects(
        "The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume."
            .to_owned(),
    )
}

impl AppliedPlate {
    pub(super) fn report(&self, report: &mut crate::SlicedPlate) -> Result<(), SliceError> {
        slice_report(&self.project, &self.resolved, report)
    }

    pub(super) fn prepare(self) -> Result<ProjectSliceState, SliceError> {
        let Self { project, resolved } = self;
        // OrcaSlicer writes the config block into every sliced G-code and uses it
        // as the machine/layer template placeholder source (`GCode.cpp` config
        // export), not only for Bambu Lab printers.
        let mut config_block = Vec::new();
        // `wipe_tower_x`/`_y` echo the sliced plate's entry: `get_at(print.get_plate_index())`
        // (`GCode.cpp:8342-8346` at v2.5.0-beta `345efec64`).
        write_config_block(
            &resolved.views,
            &project.documents().project_settings_raw,
            project.current_plate_index(),
            None,
            &mut config_block,
        )?;
        let config_block = (!config_block.is_empty()).then_some(config_block);
        let planned_objects = plan_project(&project, &resolved)?;
        let projected_objects =
            prepare_projected_objects(project.objects(), &resolved.objects, planned_objects)?;
        let scale = CoordinateScale::from_printable_area(
            &resolved.views.full.printer.remaining.printable_area,
        );
        let looped_objects = slice_projected_objects(
            project.objects(),
            &resolved.objects,
            projected_objects,
            scale,
            (
                &resolved.views.full.filament.region,
                resolved.logical_filament_count,
                resolved.process_variant_indices.as_slice(),
            ),
        )?;
        Ok(ProjectSliceState {
            project,
            resolved,
            config_block,
            scale,
            looped_objects,
        })
    }
}

pub(super) fn slice_report(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    report: &mut crate::SlicedPlate,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    report.triangle_count = inside_triangle_count(project)?;
    report.layer_height = f64::from(settings.process.object.layer_height.0 as f32);
    report.wall_loops = settings.process.region.wall_loops.0;
    report.sparse_infill_density =
        f64::from(settings.process.region.sparse_infill_density.0 as f32);
    Ok(())
}
