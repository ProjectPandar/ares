//! Slicing a plater session: the loaded input with the session's edits,
//! one plate at a time (`PartPlateList` per-plate `Print`,
//! `PartPlate.cpp:2398, 2520`). As the GUI's `BackgroundSlicingProcess::validate`
//! (`BackgroundSlicingProcess.cpp:726-734`), a session slice runs `Print::validate` only: the
//! CLI's `DynamicPrintConfig::validate` (`OrcaSlicer.cpp:4277-4282`) is not run.

use super::PlaterSession;
use crate::project_slice::{ProjectSource, slice_source};
use crate::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, ProfileSliceRequest, SliceError,
    SlicerContext,
};

/// How a session plate is sliced.
#[derive(Clone, Copy, Debug)]
pub struct PlaterSlice<'a> {
    /// The plate id (`plater_id`, 1-based); `None` slices the first plate.
    pub plate: Option<u32>,
    pub overrides: &'a ConfigOverrides,
    /// Run `Print::validate`.
    pub validate_print: bool,
}

/// Slices a session plate with the project's own presets.
pub async fn slice_plater_plate(
    session: &PlaterSession,
    slice: PlaterSlice<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    let edits = session.edits();
    slice_source(
        ProjectSource {
            input: session.source(),
            plate: slice.plate,
            profile: None,
            refresh: None,
            system_catalog: None,
            overrides: slice.overrides,
            validate_print: slice.validate_print,
            normative_check: true,
            validate_config: false,
            orient: false,
            upward: None,
            sliced_plate: None,
            edits: &edits,
        },
        metadata,
        true,
        context,
    )
}

/// Slices a session plate with selected presets (`request.plate` selects it). The session's
/// transforms already hold any load placement, so `request.placement` is not applied again.
pub async fn slice_plater_with_profiles(
    session: &PlaterSession,
    mut request: ProfileSliceRequest<'_>,
    validate_print: bool,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    request.placement = crate::Placement::None;
    let edits = session.edits();
    crate::profiles::with_edited_profile_source(session.source(), request, &edits, |mut source| {
        source.validate_print = validate_print;
        source.validate_config = false;
        if let Some(profile) = &mut source.profile {
            profile.validate_print = validate_print;
        }
        slice_source(source, metadata, true, context)
    })
}
