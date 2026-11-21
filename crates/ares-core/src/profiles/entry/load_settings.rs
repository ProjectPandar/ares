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

/// Export the effective Orca CLI settings of a direct STL with loaded presets, without slicing.
pub fn export_stl_settings(
    input: impl AsRef<[u8]>,
    filename: &str,
    presets: LoadSettingsRequest<'_>,
) -> Result<Vec<u8>, SliceError> {
    let model = crate::ImportedStl::read(input.as_ref(), filename)?;
    let request = LoadSettingsSliceRequest {
        filename,
        catalog_json: None,
        loaded_sources: presets.loaded_sources,
        uptodate: false,
        settings: presets.settings,
        filaments: presets.filaments,
        plate: None,
        overrides: &Default::default(),
        placement: Placement::None,
        orient: false,
        sliced_plate: &mut Default::default(),
    };
    with_load_settings_source(
        (ProjectInput::ImportedStl(&model), &[]),
        request,
        None,
        |source| {
            let project = crate::project_slice::load_profile_project(&source)?;
            let mut raw = project.documents().project_settings_raw.clone();
            let mut settings = project.settings().clone();
            // OrcaSlicer.cpp:3857–4023 computes the flush values before the action loop.
            crate::options::flush_matrix::prepare_stl_settings_export(
                &mut settings,
                &mut raw,
                source.overrides,
            )?;
            crate::project::export_loaded_settings(raw, project.cli_filament_count())
        },
    )
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
    slice_load_settings_source(
        (input.as_ref(), None),
        request,
        refresh,
        (metadata, false),
        context,
    )
    .map(|processed| processed.gcode)
}

pub(super) fn slice_load_settings_source(
    (input, loaded): (&[u8], Option<Box<crate::project::LoadedProfileProject>>),
    request: LoadSettingsSliceRequest<'_>,
    refresh: Option<&crate::NativeRefreshPresets>,
    (metadata, record_moves): (GenerationMetadata, bool),
    context: SlicerContext<'_>,
) -> Result<crate::ProcessedGCode, SliceError> {
    with_load_settings_source(
        (ProjectInput::Bytes(input), input),
        request,
        refresh,
        |source| {
            slice_source(
                ProjectSource { loaded, ..source },
                metadata,
                record_moves,
                context,
            )
        },
    )
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
                loaded: None,
                input,
                plate: request.plate,
                profile: None,
                refresh: None,
                system_catalog: None,
                overrides: request.overrides,
                validate_print: true,
                normative_check: true,
                validate_config: true,
                skip_modified_gcodes: false,
                allow_mix_temp: false,
                orient: false,
                upward: None,
                sliced_plate: Some(request.sliced_plate),
                max_triangle_count: 0,
                custom_gcodes: None,
                downward: None,
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

pub async fn slice_with_load_settings_processed(
    input: impl AsRef<[u8]>,
    request: LoadSettingsSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    record_moves: bool,
) -> Result<crate::ProcessedGCode, SliceError> {
    slice_load_settings_source(
        (input.as_ref(), None),
        request,
        None,
        (metadata, record_moves),
        context,
    )
}

/// Apply loaded presets to a previously imported STL and slice it.
pub async fn slice_imported_stl_with_load_settings_processed(
    model: &crate::ImportedStl,
    request: LoadSettingsSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    record_moves: bool,
) -> Result<crate::ProcessedGCode, SliceError> {
    with_load_settings_source(
        (ProjectInput::ImportedStl(model), &[]),
        request,
        None,
        |source| slice_source(source, metadata, record_moves, context),
    )
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
        loaded: None,
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
            enable_timelapse: false,
            validate_print,
        }),
        refresh: None,
        system_catalog: None,
        overrides,
        validate_print,
        normative_check: true,
        validate_config: true,
        skip_modified_gcodes: false,
        allow_mix_temp: false,
        orient,
        upward: None,
        sliced_plate: Some(sliced_plate),
        max_triangle_count: 0,
        custom_gcodes: None,
        downward: None,
        edits: &crate::plater::NO_EDITS,
    })
}
