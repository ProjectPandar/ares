use crate::{
    GenerationMetadata, SliceError, SlicerContext,
    project_slice::{ProfileInput, ProjectSource, slice_source},
};

use super::catalog::{Catalog, selection};

/// Model identity and selected-profile JSON supplied by the caller.
#[derive(Clone, Copy, Debug)]
pub struct ProfileSliceRequest<'a> {
    pub filename: &'a str,
    pub catalog_json: &'a str,
    pub selection_json: &'a str,
    pub plate: Option<u32>,
    /// M854 user edits; applied last over the selected profiles (GUI edit
    /// order, `Config.cpp:461`).
    pub overrides: &'a crate::ConfigOverrides,
}

/// Compose selected Orca profiles and slice through the project Print lifecycle.
pub async fn slice_with_profiles(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext,
) -> Result<Vec<u8>, SliceError> {
    slice_profile_source(input, request, metadata, false, context).map(|processed| processed.gcode)
}

/// The `slice_with_profiles` entry with processor move recording, mirroring
/// `slice_project_plate_processed`: the G-code is identical to the plain
/// entry and `ProcessedGCode::result` carries the toolpath moves.
pub async fn slice_with_profiles_processed(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    slice_profile_source(input, request, metadata, true, context)
}

fn slice_profile_source(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: SlicerContext,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    let ProfileSliceRequest {
        filename,
        catalog_json,
        selection_json,
        plate,
        overrides,
    } = request;
    if filename.is_empty() {
        return Err(SliceError::InvalidInput(
            "model filename is required".to_owned(),
        ));
    }
    let catalog = Catalog::from_json(catalog_json)?;
    let selection = selection(selection_json)?;
    let composed = catalog.compose(&selection)?;
    slice_source(
        ProjectSource {
            bytes: input.as_ref(),
            plate,
            profile: Some(ProfileInput {
                filename,
                composed: &composed.profile,
                raw: &composed.raw,
                filament_raws: &composed.filament_raws,
            }),
            overrides,
        },
        metadata,
        record_moves,
        context,
    )
}
