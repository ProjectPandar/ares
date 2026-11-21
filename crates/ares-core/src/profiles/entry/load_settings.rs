use super::Placement;
use crate::{
    GenerationMetadata, SliceError, SlicerContext,
    profiles::{
        ProjectPresetPolicy,
        load_settings::{LoadSettingsFile, LoadSettingsRequest, compose_project_load_settings},
    },
    project_slice::{ProfileInput, ProjectInput, ProjectSource, slice_source},
};

/// Model identity and preset files supplied by the native
/// `--load-settings`/`--load-filaments` CLI lists.
pub struct LoadSettingsSliceRequest<'a> {
    pub filename: &'a str,
    /// Explicit catalog bytes for retained kinds (OrcaSlicer.cpp:3164–3200).
    pub catalog_json: Option<&'a str>,
    pub loaded_sources: Option<&'a crate::LoadedPresetSources>,
    pub uptodate: bool,
    pub settings: &'a [LoadSettingsFile<'a>],
    pub filaments: &'a [LoadSettingsFile<'a>],
    pub plate: Option<u32>,
    pub overrides: &'a crate::ConfigOverrides,
    pub placement: Placement,
    pub orient: bool,
    /// Filled with the plate's success report values (`crate::SlicedPlate`).
    pub sliced_plate: &'a mut crate::SlicedPlate,
}

/// Slices through the project Print lifecycle with the loaded preset
/// files composed by the same single merge as the profile entry
/// (`OrcaSlicer.cpp:1953-3495`).
pub async fn slice_with_load_settings(
    input: impl AsRef<[u8]>,
    request: LoadSettingsSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    with_load_settings_source(input.as_ref(), request, |source| {
        slice_source(source, metadata, false, context)
    })
    .map(|processed| processed.gcode)
}

fn with_load_settings_source<T>(
    input: &[u8],
    request: LoadSettingsSliceRequest<'_>,
    body: impl FnOnce(ProjectSource<'_>) -> Result<T, SliceError>,
) -> Result<T, SliceError> {
    let LoadSettingsSliceRequest {
        filename,
        catalog_json,
        loaded_sources,
        uptodate,
        settings,
        filaments,
        plate,
        overrides,
        placement,
        orient,
        sliced_plate,
    } = request;
    if filename.is_empty() {
        return Err(SliceError::InvalidInput(
            "model filename is required".to_owned(),
        ));
    }
    let request = LoadSettingsRequest {
        loaded_sources,
        settings,
        filaments,
    };
    let (composed, load_names) = compose_project_load_settings(input, &request)?;
    let catalog = catalog_json
        .map(crate::profiles::project_selection::loaded_catalog)
        .transpose()?;
    // `OrcaSlicer.cpp:1682-1690`: the embedded `post_process` check lives
    // in the 3MF load block, so the composite entry validates 3MF inputs
    // only — `from_3mf_with_profiles` checks the pre-overlay embedded raw.
    let validate_print = crate::InputFormat::detect(input) == crate::InputFormat::ThreeMf;
    body(ProjectSource {
        input: ProjectInput::Bytes(input),
        plate,
        profile: Some(ProfileInput {
            filename,
            composed: &composed.profile,
            raw: &composed.raw,
            filament_raws: &composed.filament_raws,
            load_names: Some(&load_names),
            retained_refresh: catalog.as_ref().map(|catalog| {
                crate::profiles::ProjectPresetRefresh {
                    catalog,
                    policy: ProjectPresetPolicy::Cli { uptodate },
                    filename,
                    placement,
                }
            }),
            placement,
            validate_print,
        }),
        refresh: None,
        system_catalog: None,
        overrides,
        validate_print,
        validate_config: true,
        skip_modified_gcodes: false,
        orient,
        upward: None,
        sliced_plate: Some(sliced_plate),
        edits: &crate::plater::NO_EDITS,
    })
}
