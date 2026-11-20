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
    slice_with_load_settings_and_refresh(input, request, None, metadata, context).await
}

/// Native `--load-*` replacements take precedence over retained refresh.
pub async fn slice_with_load_settings_and_refresh(
    input: impl AsRef<[u8]>,
    request: LoadSettingsSliceRequest<'_>,
    refresh: Option<&crate::NativeRefreshPresets>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    let input = input.as_ref();
    with_load_settings_source(
        (ProjectInput::Bytes(input), input),
        request,
        refresh,
        |source| slice_source(source, metadata, false, context),
    )
    .map(|processed| processed.gcode)
}

/// Slices one plate of a fixed-placement assembly with the ordinary CLI preset overlay.
pub async fn slice_assembly(
    assembly: &crate::Assembly<'_>,
    request: LoadSettingsSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    let input = ProjectInput::Assembly(assembly);
    if request.settings.is_empty() && request.filaments.is_empty() {
        return slice_source(
            ProjectSource {
                input,
                plate: request.plate,
                profile: None,
                refresh: None,
                system_catalog: None,
                overrides: request.overrides,
                validate_print: true,
                validate_config: true,
                orient: false,
                upward: None,
                sliced_plate: Some(request.sliced_plate),
                edits: &crate::plater::NO_EDITS,
            },
            metadata,
            false,
            context,
        )
        .map(|processed| processed.gcode);
    }
    let bytes = assembly.archive().map_or(&[][..], |file| file.bytes);
    with_load_settings_source((input, bytes), request, None, |source| {
        slice_source(source, metadata, false, context)
    })
    .map(|processed| processed.gcode)
}

/// Slice separately loaded STL objects at their supplied positions.
pub async fn slice_stl_files_with_load_settings(
    files: &[crate::StlFile<'_>],
    request: LoadSettingsSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    let model = crate::Project::from_cli_stl_files(files)?;
    with_load_settings_source(
        (ProjectInput::Model(&model), files[0].bytes),
        request,
        None,
        |source| slice_source(source, metadata, false, context),
    )
    .map(|processed| processed.gcode)
}

pub(crate) fn with_load_settings_source<T>(
    (input, bytes): (ProjectInput<'_>, &[u8]),
    request: LoadSettingsSliceRequest<'_>,
    refresh: Option<&crate::NativeRefreshPresets>,
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
    let (composed, load_names) = compose_project_load_settings(bytes, &request)?;
    let catalog = catalog_json
        .map(crate::profiles::project_selection::loaded_catalog)
        .transpose()?;
    // `OrcaSlicer.cpp:1682-1690`: the embedded `post_process` check lives
    // in the 3MF load block, so the composite entry validates 3MF inputs
    // only — `from_3mf_with_profiles` checks the pre-overlay embedded raw.
    let validate_print = matches!(input, ProjectInput::Assembly(_))
        || crate::InputFormat::detect(bytes) == crate::InputFormat::ThreeMf;
    body(ProjectSource {
        input,
        plate,
        profile: Some(ProfileInput {
            filename,
            composed: &composed.profile,
            raw: &composed.raw,
            filament_raws: &composed.filament_raws,
            load_names: Some(&load_names),
            retained_refresh: refresh
                .map(crate::profiles::ProjectPresetRefresh::native)
                .or_else(|| {
                    catalog
                        .as_ref()
                        .map(|catalog| crate::profiles::ProjectPresetRefresh {
                            source: crate::profiles::project_refresh::RefreshSource::Catalog(
                                catalog,
                            ),
                            policy: ProjectPresetPolicy::Cli { uptodate },
                            filename,
                            placement,
                        })
                }),
            placement,
            validate_print,
        }),
        refresh: None,
        system_catalog: None,
        overrides,
        validate_print,
        validate_config: true,
        orient,
        upward: None,
        sliced_plate: Some(sliced_plate),
        edits: &crate::plater::NO_EDITS,
    })
}
