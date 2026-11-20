//! Headless `PartPlateList::store_to_3mf_structure` plate facts.

use crate::project::raw_settings::ProjectSettingsRaw;
use crate::{GenerationMetadata, ProcessedGCode, SliceError, SlicerContext};

use super::{ProjectSource, perimeters, slice_plate};

pub(crate) struct ArchivePlate {
    pub(crate) object_name: String,
    pub(crate) identify_id: u32,
    pub(crate) bounds: [f64; 4],
    pub(crate) area: f32,
    pub(crate) layer_height: f32,
    pub(crate) layer_count: usize,
    pub(crate) project_settings: ProjectSettingsRaw,
}

#[inline(never)]
pub(crate) fn slice_source(
    project: ProjectSource<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_source_with_archive(project, metadata, record_moves, context, None)
}

pub(crate) fn slice_source_with_archive(
    mut project: ProjectSource<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    mut context: SlicerContext<'_>,
    archive: Option<&mut Option<ArchivePlate>>,
) -> Result<ProcessedGCode, SliceError> {
    let sliced_plate = project.sliced_plate.take();
    slice_plate(
        (metadata, record_moves),
        &mut context,
        sliced_plate,
        archive,
        |report, context| {
            if let Some(profile) = project.profile.as_mut() {
                profile.enable_timelapse = context.enable_timelapse;
            }
            perimeters::prepare_post_layer_region_perimeters(
                ProjectSource {
                    sliced_plate: report,
                    max_triangle_count: context.max_triangle_count,
                    custom_gcodes: context.custom_gcodes.take(),
                    ..project
                },
                context,
            )
        },
    )
}
