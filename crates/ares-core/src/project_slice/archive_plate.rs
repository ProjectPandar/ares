//! Headless `PartPlateList::store_to_3mf_structure` plate facts.

use crate::project::raw_settings::ProjectSettingsRaw;
use crate::{GenerationMetadata, ProcessedGCode, SliceError, SlicerContext};

use super::{ProjectSource, perimeters, slice_plate};

#[derive(Clone, Copy)]
pub(crate) enum ArchiveSource {
    Saved,
    DirectStl,
}

pub(crate) struct ArchivePlate {
    pub(crate) objects: Vec<crate::SlicedArchiveObject>,
    pub(crate) layer_count: usize,
    pub(crate) filament_color: String,
    pub(crate) filament_type: String,
    pub(crate) filament_id: String,
    pub(crate) nozzle_diameter: f32,
    pub(crate) label_object_enabled: bool,
    pub(crate) project_settings: ProjectSettingsRaw,
    pub(crate) stl_model: Option<crate::project::export::SavedModel>,
}

#[inline(never)]
pub(crate) fn slice_source(
    project: ProjectSource<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_source_inner(project, (metadata, record_moves), context, None, None)
}

pub(crate) fn slice_source_with_custom_gcodes(
    project: ProjectSource<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: SlicerContext<'_>,
) -> Result<(ProcessedGCode, Vec<crate::gcode_processor::CustomGCodeItem>), SliceError> {
    let mut custom_gcodes = Vec::new();
    let output = slice_source_inner(
        project,
        (metadata, record_moves),
        context,
        None,
        Some(&mut custom_gcodes),
    )?;
    Ok((output, custom_gcodes))
}

pub(crate) fn slice_source_with_archive(
    project: ProjectSource<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: SlicerContext<'_>,
    archive: Option<ArchiveSource>,
) -> Result<ProcessedGCode, SliceError> {
    slice_source_inner(project, (metadata, record_moves), context, archive, None)
}

fn slice_source_inner(
    mut project: ProjectSource<'_>,
    generation: (GenerationMetadata, bool),
    mut context: SlicerContext<'_>,
    archive: Option<ArchiveSource>,
    plate_custom_gcodes: Option<&mut Vec<crate::gcode_processor::CustomGCodeItem>>,
) -> Result<ProcessedGCode, SliceError> {
    let sliced_plate = project.sliced_plate.take();
    slice_plate(
        generation,
        &mut context,
        (sliced_plate, plate_custom_gcodes),
        archive,
        |report, context| {
            if let Some(profile) = project.profile.as_mut() {
                profile.enable_timelapse = context.enable_timelapse;
            }
            perimeters::prepare_post_layer_region_perimeters(
                ProjectSource {
                    sliced_plate: report,
                    normative_check: context.normative_check(),
                    skip_modified_gcodes: context.skip_modified_gcodes(),
                    allow_mix_temp: context.allow_mix_temp(),
                    max_triangle_count: context.max_triangle_count,
                    custom_gcodes: context.custom_gcodes.take(),
                    downward: context.downward.take(),
                    ..project
                },
                context,
            )
        },
    )
}
