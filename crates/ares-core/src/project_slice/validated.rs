//! Adapter orchestration of `Print.hpp:911,935`: apply once, validate, process.
use super::source::CliUpwardMachine;
use super::{
    ProjectSource, slice_source, slice_source_with_archive, slice_source_with_custom_gcodes,
};
use crate::{
    ConfigOverrides, GenerationMetadata, Project, SliceError, SlicerContext,
    gcode_processor::ProcessedGCode,
};

/// `OrcaSlicer.cpp:1682-1690`: a 3MF whose embedded `post_process` array
/// is non-empty (including `[""]`) is rejected before slicing with the
/// exact `:131` message. Called from the shared loading path when
/// `validate_print` is true — after the single `load_project` and before
/// `Print::validate` (profile paths check the pre-overlay embedded raw).
pub(crate) fn validate_post_process(project: &Project) -> Result<(), SliceError> {
    let Some(value) = project.documents().project_settings_raw.get("post_process") else {
        return Ok(());
    };
    let len = match value {
        crate::project::raw_settings::RawConfigValue::List(items) => items.len(),
        _ => 1,
    };
    if len > 0 {
        return Err(SliceError::PostProcessNotSupported);
    }
    Ok(())
}

/// CLI report-aware entry: early errors leave the caller's empty report
/// untouched; later errors preserve the snapshot committed by project state.
pub struct CliProjectSliceRequest<'a> {
    pub plate: Option<u32>,
    pub overrides: &'a ConfigOverrides,
    pub record_moves: bool,
    pub orient: bool,
    pub upward: CliUpwardMachine<'a>,
    /// Filled with the plate's success report values (`crate::SlicedPlate`).
    pub sliced_plate: &'a mut crate::SlicedPlate,
}

pub async fn slice_cli_project_plate_validated(
    input: impl AsRef<[u8]>,
    request: CliProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_cli_project_plate_validated_with_catalog(input, request, metadata, None, context).await
}

pub async fn slice_cli_project_plate_validated_with_catalog(
    input: impl AsRef<[u8]>,
    request: CliProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    system_catalog: Option<&crate::PresetCatalogHandle>,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_cli_project(
        (input.as_ref(), None),
        request,
        metadata,
        (system_catalog, None),
        context,
    )
}

pub async fn slice_cli_project_plate_with_refresh(
    input: impl AsRef<[u8]>,
    request: CliProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    refresh: &crate::NativeRefreshPresets,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_cli_project(
        (input.as_ref(), None),
        request,
        metadata,
        (None, Some(refresh)),
        context,
    )
}

impl crate::CliPresetProject<'_> {
    pub fn slice_project(
        self,
        request: CliProjectSliceRequest<'_>,
        metadata: GenerationMetadata,
        presets: (
            Option<&crate::PresetCatalogHandle>,
            Option<&crate::NativeRefreshPresets>,
        ),
        context: SlicerContext<'_>,
    ) -> Result<ProcessedGCode, SliceError> {
        slice_cli_project(self.into_parts(), request, metadata, presets, context)
    }
}

fn slice_cli_project(
    input: (&[u8], Option<Box<crate::project::LoadedProfileProject>>),
    request: CliProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    presets: (
        Option<&crate::PresetCatalogHandle>,
        Option<&crate::NativeRefreshPresets>,
    ),
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    let mut gcode = Vec::new();
    let streamed = super::streaming::slice_cli_project_to_sink(
        input,
        request,
        (metadata, presets),
        context,
        &mut gcode,
    )?;
    Ok(streamed.into_processed(gcode))
}

pub struct DirectProjectSliceRequest<'a> {
    pub plate: Option<u32>,
    pub overrides: &'a ConfigOverrides,
    pub system_catalog: Option<&'a crate::PresetCatalogHandle>,
}

pub async fn slice_project_plate_validated(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    slice_project_plate_validated_with_catalog(
        input,
        DirectProjectSliceRequest {
            plate,
            overrides,
            system_catalog: None,
        },
        metadata,
        context,
    )
    .await
}

pub async fn slice_project_plate_validated_with_catalog(
    input: impl AsRef<[u8]>,
    request: DirectProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    let DirectProjectSliceRequest {
        plate,
        overrides,
        system_catalog,
    } = request;
    overrides.validate_cli_config()?;
    slice_source(
        ProjectSource {
            loaded: None,
            input: crate::project_slice::ProjectInput::Bytes(input.as_ref()),
            plate,
            profile: None,
            refresh: None,
            system_catalog,
            overrides,
            validate_print: true,
            normative_check: true,
            allow_mix_temp: false,
            validate_config: true,
            skip_modified_gcodes: false,
            orient: false,
            upward: None,
            sliced_plate: None,
            max_triangle_count: 0,
            custom_gcodes: None,
            downward: None,
            edits: &crate::plater::NO_EDITS,
        },
        metadata,
        false,
        context,
    )
    .map(|output| output.gcode)
}

pub async fn slice_project_plate_processed_validated(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_project_plate_processed_validated_with_catalog(
        input,
        DirectProjectSliceRequest {
            plate,
            overrides,
            system_catalog: None,
        },
        metadata,
        context,
    )
    .await
}

pub async fn slice_project_plate_processed_validated_with_catalog(
    input: impl AsRef<[u8]>,
    request: DirectProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_project_plate_processed_validated_with_custom_gcodes(input, request, metadata, context)
        .await
        .map(|(output, _)| output)
}

pub async fn slice_project_plate_processed_validated_with_custom_gcodes(
    input: impl AsRef<[u8]>,
    request: DirectProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<(ProcessedGCode, Vec<crate::gcode_processor::CustomGCodeItem>), SliceError> {
    let DirectProjectSliceRequest {
        plate,
        overrides,
        system_catalog,
    } = request;
    overrides.validate_cli_config()?;
    slice_source_with_custom_gcodes(
        ProjectSource {
            loaded: None,
            input: crate::project_slice::ProjectInput::Bytes(input.as_ref()),
            plate,
            profile: None,
            refresh: None,
            system_catalog,
            overrides,
            validate_print: true,
            normative_check: true,
            allow_mix_temp: false,
            validate_config: true,
            skip_modified_gcodes: false,
            orient: false,
            upward: None,
            sliced_plate: None,
            max_triangle_count: 0,
            custom_gcodes: None,
            downward: None,
            edits: &crate::plater::NO_EDITS,
        },
        metadata,
        true,
        context,
    )
}

pub async fn slice_project_plate_with_archive(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_project_plate_with_archive_loaded(
        (input.as_ref(), None),
        metadata,
        plate,
        overrides,
        context,
    )
}

pub(crate) fn slice_loaded_project_plate_with_archive(
    (input, loaded): (&[u8], Box<crate::project::LoadedProfileProject>),
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    slice_project_plate_with_archive_loaded(
        (input, Some(loaded)),
        metadata,
        plate,
        overrides,
        context,
    )
}

fn slice_project_plate_with_archive_loaded(
    (input, loaded): (&[u8], Option<Box<crate::project::LoadedProfileProject>>),
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    overrides.validate_cli_config()?;
    slice_source_with_archive(
        ProjectSource {
            loaded,
            input: crate::project_slice::ProjectInput::Bytes(input),
            plate,
            profile: None,
            refresh: None,
            system_catalog: None,
            overrides,
            validate_print: true,
            normative_check: true,
            validate_config: true,
            skip_modified_gcodes: false,
            allow_mix_temp: false,
            orient: false,
            upward: None,
            sliced_plate: None,
            max_triangle_count: 0,
            custom_gcodes: None,
            downward: None,
            edits: &crate::plater::NO_EDITS,
        },
        metadata,
        false,
        context,
        Some(super::ArchiveSource::Saved),
    )
}
