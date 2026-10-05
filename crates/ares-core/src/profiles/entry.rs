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
}

/// Compose selected Orca profiles and slice through the project Print lifecycle.
pub async fn slice_with_profiles(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext,
) -> Result<Vec<u8>, SliceError> {
    let ProfileSliceRequest {
        filename,
        catalog_json,
        selection_json,
        plate,
    } = request;
    if filename.is_empty() {
        return Err(SliceError::InvalidInput(
            "model filename is required".to_owned(),
        ));
    }
    let catalog = Catalog::from_json(catalog_json)?;
    let selection = selection(selection_json)?;
    let composed = catalog.compose(&selection)?;
    // Profile-sourced slices carry no user overrides (M854).
    let overrides = crate::ConfigOverrides::default();
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
            overrides: &overrides,
        },
        metadata,
        false,
        context,
    )
    .map(|processed| processed.gcode)
}
