//! Adapter orchestration of `Print.hpp:911,935`: apply once, validate, process.
use super::{ProjectSource, slice_source};
use crate::{
    ConfigOverrides, GenerationMetadata, SliceError, SlicerContext, gcode_processor::ProcessedGCode,
};

pub async fn slice_project_plate_validated(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext,
) -> Result<Vec<u8>, SliceError> {
    slice_source(
        ProjectSource {
            bytes: input.as_ref(),
            plate,
            profile: None,
            overrides,
            validate_print: true,
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
    context: SlicerContext,
) -> Result<ProcessedGCode, SliceError> {
    slice_source(
        ProjectSource {
            bytes: input.as_ref(),
            plate,
            profile: None,
            overrides,
            validate_print: true,
        },
        metadata,
        true,
        context,
    )
}
