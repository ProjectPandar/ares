use crate::{
    GenerationMetadata, SliceError, SlicerContext,
    project_slice::{ProfileInput, ProjectSource, slice_source},
};

use super::catalog::{Catalog, selection};
mod load_settings;
pub use load_settings::{LoadSettingsSliceRequest, slice_with_load_settings};

/// Model placement before slicing. `Center` mirrors the CLI `--center`
/// transform (`OrcaSlicer.cpp:4376-4392`): bed-center placement plus the
/// Z ground drop. The default keeps the model's own coordinates; upstream
/// CLI STL inputs default to full libnest2d arrange (`:1438,:1797,:4890`),
/// a known deviation deferred to the arrange slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Placement {
    None,
    Center,
}

impl Placement {
    pub fn parse(value: &str) -> Result<Self, SliceError> {
        match value {
            "none" => Ok(Self::None),
            "center" => Ok(Self::Center),
            other => Err(SliceError::InvalidInput(format!(
                "invalid placement: {other}"
            ))),
        }
    }
}

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
    /// Model placement before slicing (`--center` transform).
    pub placement: Placement,
}

/// Compose selected Orca profiles and slice through the project Print lifecycle.
pub async fn slice_with_profiles(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
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
    context: SlicerContext<'_>,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    slice_profile_source(input, request, metadata, true, context)
}

/// Adapter entry: compose and load once, then validate before processing.
pub async fn slice_with_profiles_validated(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    slice_validated_profile_source(input, request, metadata, (false, None, None), context)
        .map(|output| output.gcode)
}

pub async fn slice_with_profiles_processed_validated(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    slice_validated_profile_source(input, request, metadata, (true, None, None), context)
}

/// Selected profiles and the CLI's processing/report outputs.
pub struct CliProfileSliceRequest<'a> {
    pub profile: ProfileSliceRequest<'a>,
    pub record_moves: bool,
    pub sliced_plate: &'a mut crate::SlicedPlate,
    pub upward: crate::CliUpwardMachine<'a>,
}

/// The validated profile entry with the existing CLI plate report channel
/// (`OrcaSlicer.cpp:6926-6931,7150-7160,8126-8136`).
pub async fn slice_cli_with_profiles_validated(
    input: impl AsRef<[u8]>,
    request: CliProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    let CliProfileSliceRequest {
        profile,
        record_moves,
        sliced_plate,
        upward,
    } = request;
    upward.report.clear();
    slice_validated_profile_source(
        input,
        profile,
        metadata,
        (record_moves, Some(sliced_plate), Some(upward)),
        context,
    )
}

fn slice_validated_profile_source(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    (record_moves, sliced_plate, upward): (
        bool,
        Option<&mut crate::SlicedPlate>,
        Option<crate::CliUpwardMachine<'_>>,
    ),
    context: SlicerContext<'_>,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    request.overrides.validate_cli_config()?;
    with_profile_source(input.as_ref(), request, |source| {
        let mut source = ProjectSource {
            sliced_plate,
            upward,
            ..source
        };
        source.validate_print = true;
        if let Some(profile) = &mut source.profile {
            profile.validate_print = true;
        }
        slice_source(source, metadata, record_moves, context)
    })
}

/// Loads the model with the selected profiles and applies the requested
/// placement, returning the placed project for scene extraction without
/// slicing. Reuses the exact `load_with_profile` and placement logic from
/// `prepare_profile_slice` — no separate centering implementation.
pub async fn profile_placed_project(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
) -> Result<crate::Project, SliceError> {
    with_profile_source(input.as_ref(), request, |source| {
        if let Some(refresh) = source.refresh {
            let mut project = crate::project_slice::load_profile_project(&source)?;
            project.apply_plater_edits(source.edits);
            refresh.place(&mut project);
            return Ok(project);
        }
        let Some(profile) = source.profile else {
            return Err(SliceError::InvalidInput(
                "a selected profile is required".to_owned(),
            ));
        };
        crate::project_slice::load_and_place(&source, profile)
    })
}
pub fn validate_with_profiles(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
) -> Result<(), SliceError> {
    with_profile_source(input.as_ref(), request, |source| {
        if let Some(profile) = source.profile {
            crate::project_slice::validate_profile_source(&source, profile)?;
        } else if source.refresh.is_some() {
            let project = crate::project_slice::load_profile_project(&source)?;
            project
                .settings()
                .validate_loaded_config(&project.documents().project_settings_raw, true)?;
            crate::print::validate::validate_loaded_project(project, source.plate, None)?;
        }
        Ok(())
    })
}

/// `OrcaSlicer.cpp:1682-1690`: the 3MF load block admits the project
/// before the CLI adapter reads the `--load-settings` files
/// (`:1953-2024`); this admission probe runs the same embedded
/// `post_process` gate the composite slice entry runs, so the `-19`
/// rejection wins over the adapter's missing-file `-5`.
pub fn validate_load_settings_input(input: impl AsRef<[u8]>) -> Result<(), SliceError> {
    // The gate lives in the 3MF load block, so other input formats take
    // the adapter path unchanged (the same keying the composite entry
    // applies to its `validate_print` flag).
    if crate::InputFormat::detect(input.as_ref()) != crate::InputFormat::ThreeMf {
        return Ok(());
    }
    let project = crate::project::load_project(input)?;
    crate::project_slice::validate_post_process(&project)
}

fn slice_profile_source(
    input: impl AsRef<[u8]>,
    request: ProfileSliceRequest<'_>,
    metadata: GenerationMetadata,
    record_moves: bool,
    context: SlicerContext<'_>,
) -> Result<crate::gcode_processor::ProcessedGCode, SliceError> {
    with_profile_source(input.as_ref(), request, |source| {
        slice_source(source, metadata, record_moves, context)
    })
}

/// The effective config view for a selected-profile request.
#[derive(serde::Serialize)]
pub struct ProfileSettings(pub(crate) crate::project::raw_settings::ProjectSettingsRaw);

/// Composes the request and runs `body` on the resulting source; both the
/// slice entries and the settings view share this single compose path.
fn with_profile_source<T>(
    input: &[u8],
    request: ProfileSliceRequest<'_>,
    body: impl FnOnce(ProjectSource<'_>) -> Result<T, SliceError>,
) -> Result<T, SliceError> {
    with_edited_profile_source(
        crate::project_slice::ProjectInput::Bytes(input),
        request,
        &crate::plater::NO_EDITS,
        body,
    )
}

/// `with_profile_source` with plater edits applied to the loaded model.
pub(crate) fn with_edited_profile_source<T>(
    input: crate::project_slice::ProjectInput<'_>,
    request: ProfileSliceRequest<'_>,
    edits: &crate::plater::PlaterEdits,
    body: impl FnOnce(ProjectSource<'_>) -> Result<T, SliceError>,
) -> Result<T, SliceError> {
    let ProfileSliceRequest {
        filename,
        catalog_json,
        selection_json,
        plate,
        overrides,
        placement,
    } = request;
    if filename.is_empty() {
        return Err(SliceError::InvalidInput(
            "model filename is required".to_owned(),
        ));
    }
    let catalog = Catalog::from_json(catalog_json)?;
    if let Some(uptodate) = super::project_refresh::selection(selection_json)? {
        return body(ProjectSource {
            input,
            plate,
            profile: None,
            refresh: Some(super::ProjectPresetRefresh {
                catalog: &catalog,
                uptodate,
                placement,
            }),
            overrides,
            validate_print: false,
            validate_config: true,
            upward: None,
            sliced_plate: None,
            edits,
        });
    }
    let selection = selection(selection_json)?;
    let composed = catalog.compose(&selection)?;
    body(ProjectSource {
        input,
        plate,
        profile: Some(ProfileInput {
            filename,
            composed: &composed.profile,
            raw: &composed.raw,
            filament_raws: &composed.filament_raws,
            load_names: None,
            retained_refresh: None,
            placement,
            validate_print: false,
        }),
        refresh: None,
        overrides,
        validate_print: false,
        validate_config: true,
        upward: None,
        sliced_plate: None,
        edits,
    })
}
