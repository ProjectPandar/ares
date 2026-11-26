//! Streaming `GCodeSink` entry points mirroring `project_slice` and `validated`
//! (`GCode.cpp:2850-3060`, `GCodeProcessor.cpp:859-898`).

use super::{
    CliProjectSliceRequest, DirectProjectSliceRequest, ProjectInput, ProjectSource,
    slice_source_to_sink,
};
use crate::{GCodeSink, GenerationMetadata, SliceError, SlicerContext, StreamedGCode};

/// Slices one plate of a 3MF project and streams finalized G-code chunks into
/// `sink` (`GCode.cpp:do_export`).
pub async fn slice_project_plate_to_sink(
    project: impl AsRef<[u8]>,
    request: DirectProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    slice_project_plate_to_sink_inner(
        project.as_ref(),
        request,
        (metadata, false, false),
        context,
        sink,
    )
}

/// Slices one plate with move recording enabled and streams finalized G-code
/// chunks into `sink` while returning the exact [`crate::GCodeProcessorResult`].
pub async fn slice_project_plate_processed_to_sink(
    project: impl AsRef<[u8]>,
    request: DirectProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    slice_project_plate_to_sink_inner(
        project.as_ref(),
        request,
        (metadata, true, false),
        context,
        sink,
    )
}

/// Validated streaming counterpart to [`crate::slice_project_plate_validated`].
pub async fn slice_project_plate_validated_to_sink(
    input: impl AsRef<[u8]>,
    request: DirectProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    request.overrides.validate_cli_config()?;
    slice_project_plate_to_sink_inner(
        input.as_ref(),
        request,
        (metadata, false, true),
        context,
        sink,
    )
}

/// CLI report-aware streaming counterpart to
/// [`crate::slice_cli_project_plate_validated`].
pub async fn slice_cli_project_plate_validated_to_sink(
    input: impl AsRef<[u8]>,
    request: CliProjectSliceRequest<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    slice_cli_project_plate_validated_with_catalog_to_sink(
        input,
        request,
        (metadata, None),
        context,
        sink,
    )
    .await
}

/// CLI report-aware streaming counterpart to
/// [`crate::slice_cli_project_plate_validated_with_catalog`].
pub async fn slice_cli_project_plate_validated_with_catalog_to_sink(
    input: impl AsRef<[u8]>,
    request: CliProjectSliceRequest<'_>,
    (metadata, system_catalog): (GenerationMetadata, Option<&crate::PresetCatalogHandle>),
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    slice_cli_project_to_sink(
        (input.as_ref(), None),
        request,
        (metadata, (system_catalog, None)),
        context,
        sink,
    )
}

impl crate::CliPresetProject<'_> {
    /// Streams finalized G-code chunks directly into `sink` using the
    /// already-opened 3MF archive.
    pub fn slice_project_to_sink(
        self,
        request: CliProjectSliceRequest<'_>,
        metadata_and_presets: (
            GenerationMetadata,
            (
                Option<&crate::PresetCatalogHandle>,
                Option<&crate::NativeRefreshPresets>,
            ),
        ),
        context: SlicerContext<'_>,
        sink: &mut dyn GCodeSink,
    ) -> Result<StreamedGCode, SliceError> {
        slice_cli_project_to_sink(
            self.into_parts(),
            request,
            metadata_and_presets,
            context,
            sink,
        )
    }
}

pub(super) fn slice_cli_project_to_sink(
    (input, loaded): (&[u8], Option<Box<crate::project::LoadedProfileProject>>),
    request: CliProjectSliceRequest<'_>,
    (metadata, (system_catalog, refresh)): (
        GenerationMetadata,
        (
            Option<&crate::PresetCatalogHandle>,
            Option<&crate::NativeRefreshPresets>,
        ),
    ),
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    let CliProjectSliceRequest {
        plate,
        overrides,
        record_moves,
        orient,
        upward,
        sliced_plate,
    } = request;
    upward.report.clear();
    overrides.validate_cli_config()?;
    slice_source_to_sink(
        ProjectSource {
            loaded,
            input: match refresh {
                None => ProjectInput::SavedCliBytes(input),
                Some(_) => ProjectInput::Bytes(input),
            },
            plate,
            profile: None,
            refresh: refresh.map(|presets| {
                crate::profiles::ProjectPresetRefresh::native(
                    presets,
                    context.skip_modified_gcodes(),
                )
            }),
            overrides,
            validate_print: true,
            normative_check: true,
            allow_mix_temp: false,
            validate_config: true,
            skip_modified_gcodes: false,
            orient,
            upward: Some(upward),
            sliced_plate: Some(sliced_plate),
            system_catalog,
            max_triangle_count: 0,
            custom_gcodes: None,
            downward: None,
            edits: &crate::plater::NO_EDITS,
        },
        (metadata, record_moves),
        context,
        sink,
    )
}

pub(super) fn slice_project_plate_to_sink_inner(
    input: &[u8],
    request: DirectProjectSliceRequest<'_>,
    (metadata, record_moves, validate_print): (GenerationMetadata, bool, bool),
    context: SlicerContext<'_>,
    sink: &mut dyn GCodeSink,
) -> Result<StreamedGCode, SliceError> {
    let DirectProjectSliceRequest {
        plate,
        overrides,
        system_catalog,
    } = request;
    slice_source_to_sink(
        ProjectSource {
            loaded: None,
            input: ProjectInput::Bytes(input),
            plate,
            profile: None,
            refresh: None,
            system_catalog,
            overrides,
            validate_print,
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
        (metadata, record_moves),
        context,
        sink,
    )
}
