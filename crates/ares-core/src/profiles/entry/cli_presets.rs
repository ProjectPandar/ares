use super::{
    CliProfileSliceRequest, LoadSettingsSliceRequest, Placement, ProfileSliceRequest,
    load_settings::slice_load_settings_source, slice_cli_profile_source,
};
use crate::{
    CliUpwardMachine, ConfigOverrides, GenerationMetadata, InputFormat, LoadSettingsFile,
    LoadedPresetSources, ProcessedGCode, SliceError, SlicedPlate, SlicerContext, SourceIdentity,
    UpwardMachineInput, UpwardMachineSource, project::LoadedProfileProject,
};

pub enum CliPresetFiles<'a> {
    LoadSettings {
        filename: &'a str,
        catalog: Option<String>,
        settings: Vec<(String, String, Option<SourceIdentity>)>,
        filaments: Vec<(String, String, Option<SourceIdentity>)>,
        loaded_sources: Box<LoadedPresetSources>,
    },
    ProjectCatalog {
        filename: &'a str,
        catalog: String,
        upward_source: Option<UpwardMachineSource>,
        upward_report: &'a mut Vec<String>,
    },
}

pub struct CliPresetSliceRequest<'a> {
    pub plate: Option<u32>,
    pub overrides: &'a ConfigOverrides,
    pub orient: bool,
    pub uptodate: bool,
    pub record_moves: bool,
    pub sliced_plate: &'a mut SlicedPlate,
}

/// OrcaSlicer.cpp:1755–1810 imports and admits the 3MF before reading preset
/// files. The adapter reports import and file-read errors separately from
/// later slice errors, which can carry plate state.
pub fn slice_cli_preset_project<'a, E>(
    input: &[u8],
    request: CliPresetSliceRequest<'a>,
    context: SlicerContext<'_>,
    import_error: impl FnOnce(SliceError) -> E,
    read_presets: impl FnOnce() -> Result<CliPresetFiles<'a>, E>,
) -> Result<Result<ProcessedGCode, SliceError>, E> {
    let loaded = if InputFormat::detect(input) == InputFormat::ThreeMf {
        Some(Box::new(
            LoadedProfileProject::load(input).map_err(import_error)?,
        ))
    } else {
        None
    };
    let CliPresetSliceRequest {
        plate,
        overrides,
        orient,
        uptodate,
        record_moves,
        sliced_plate,
    } = request;
    let metadata = GenerationMetadata::default();
    Ok(match read_presets()? {
        CliPresetFiles::LoadSettings {
            filename,
            catalog,
            settings,
            filaments,
            loaded_sources,
        } => {
            let settings = borrow_files(&settings);
            let filaments = borrow_files(&filaments);
            slice_load_settings_source(
                input,
                LoadSettingsSliceRequest {
                    filename,
                    catalog_json: catalog.as_deref(),
                    loaded_sources: Some(&loaded_sources),
                    uptodate,
                    settings: &settings,
                    filaments: &filaments,
                    plate,
                    overrides,
                    placement: Placement::None,
                    orient,
                    sliced_plate,
                },
                metadata,
                context,
                loaded,
            )
        }
        CliPresetFiles::ProjectCatalog {
            filename,
            catalog,
            upward_source,
            upward_report,
        } => {
            let selection_json = if uptodate {
                r#"{"from_project":true,"uptodate":true}"#
            } else {
                r#"{"from_project":true,"uptodate":false}"#
            };
            slice_cli_profile_source(
                input,
                CliProfileSliceRequest {
                    profile: ProfileSliceRequest {
                        filename,
                        catalog_json: &catalog,
                        selection_json,
                        plate,
                        overrides,
                        placement: Placement::None,
                    },
                    orient,
                    record_moves,
                    sliced_plate,
                    upward: CliUpwardMachine {
                        source: upward_source,
                        input: UpwardMachineInput::Embedded,
                        report: upward_report,
                    },
                },
                metadata,
                context,
                loaded,
            )
        }
    })
}

fn borrow_files(files: &[(String, String, Option<SourceIdentity>)]) -> Vec<LoadSettingsFile<'_>> {
    files
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect()
}
