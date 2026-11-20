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
#[derive(Debug)]
pub struct LoadSettingsSliceRequest<'a> {
    pub filename: &'a str,
    /// Explicit catalog bytes for retained kinds (OrcaSlicer.cpp:3164–3200).
    pub catalog_json: Option<&'a str>,
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

/// The output and report of one plate from the direct STL slice action.
pub struct SlicedStlPlate {
    pub id: u32,
    pub gcode: Vec<u8>,
    pub report: crate::SlicedPlate,
}

/// Place the decoded STL models once, then apply the one-based slice action
/// (`OrcaSlicer.cpp:2042–2045,6441–6445,6522–6531`). `0` slices every plate.
pub async fn slice_stl_models_with_load_settings(
    models: &[(&str, crate::Model)],
    request: LoadSettingsSliceRequest<'_>,
    selection: u32,
) -> Result<Vec<SlicedStlPlate>, SliceError> {
    if models.is_empty() {
        return Err(SliceError::EmptyInput);
    }
    let model = crate::Project::from_cli_stl_models(models);
    with_load_settings_source(&[], request, |source| {
        let source = ProjectSource {
            input: ProjectInput::Model(&model),
            plate: None,
            ..source
        };
        let (project, preparation) = crate::project_slice::load_profile_slice(
            &source,
            source.profile.expect("loaded settings supply a profile"),
            None,
        )?;
        let mut plates = Vec::new();
        for plate in project.plates() {
            let id = plate.id();
            if selection != 0 && selection != id {
                continue;
            }
            let mut report = crate::SlicedPlate::default();
            let output = slice_source(
                ProjectSource {
                    input: ProjectInput::ArrangedStl(&project, &preparation),
                    plate: Some(id),
                    profile: None,
                    upward: None,
                    sliced_plate: Some(&mut report),
                    ..source
                },
                GenerationMetadata::default(),
                false,
                SlicerContext::new(0),
            )?;
            plates.push(SlicedStlPlate {
                id,
                gcode: output.gcode,
                report,
            });
        }
        Ok(plates)
    })
}

fn with_load_settings_source<T>(
    input: &[u8],
    request: LoadSettingsSliceRequest<'_>,
    body: impl FnOnce(ProjectSource<'_>) -> Result<T, SliceError>,
) -> Result<T, SliceError> {
    let LoadSettingsSliceRequest {
        filename,
        catalog_json,
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
        overrides,
        validate_print,
        validate_config: true,
        orient,
        upward: None,
        sliced_plate: Some(sliced_plate),
        edits: &crate::plater::NO_EDITS,
    })
}
