use ares_core::{ConfigOverrides, GenerationMetadata, ProcessedGCode, SliceError, SlicerContext};

use super::project_preset_catalog::request;
pub(crate) use super::project_preset_catalog::{project_supply, slice_project_plate_validated};

pub(crate) async fn slice_project(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    match project_supply()? {
        None => ares_core::slice_project(input, metadata, context).await,
        Some(supply) => {
            ares_core::slice_with_profiles(
                input,
                request(supply, None, &ConfigOverrides::default()),
                metadata,
                context,
            )
            .await
        }
    }
}

pub(crate) async fn slice_project_plate(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<Vec<u8>, SliceError> {
    match project_supply()? {
        None => ares_core::slice_project_plate(input, metadata, plate, overrides, context).await,
        Some(supply) => {
            ares_core::slice_with_profiles(
                input,
                request(supply, plate, overrides),
                metadata,
                context,
            )
            .await
        }
    }
}

pub(crate) async fn slice_project_plate_processed(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    match project_supply()? {
        None => {
            ares_core::slice_project_plate_processed(input, metadata, plate, overrides, context)
                .await
        }
        Some(supply) => {
            ares_core::slice_with_profiles_processed(
                input,
                request(supply, plate, overrides),
                metadata,
                context,
            )
            .await
        }
    }
}

pub(crate) async fn slice_project_plate_processed_validated(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    match project_supply()? {
        None => {
            ares_core::slice_project_plate_processed_validated(
                input, metadata, plate, overrides, context,
            )
            .await
        }
        Some(supply) => {
            ares_core::slice_with_profiles_processed_validated(
                input,
                request(supply, plate, overrides),
                metadata,
                context,
            )
            .await
        }
    }
}

pub(crate) async fn slice_plater_plate(
    session: &ares_core::PlaterSession,
    slice: ares_core::PlaterSlice<'_>,
    metadata: GenerationMetadata,
    context: SlicerContext<'_>,
) -> Result<ProcessedGCode, SliceError> {
    match project_supply()? {
        None => ares_core::slice_plater_plate(session, slice, metadata, context).await,
        Some(supply) => {
            ares_core::slice_plater_with_profiles(
                session,
                request(supply, slice.plate, slice.overrides),
                slice.validate_print,
                metadata,
                context,
            )
            .await
        }
    }
}

pub(crate) fn native_arguments(mut args: Vec<String>) -> Result<Vec<String>, SliceError> {
    let project_action = args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "--slice" | "--export-settings" | "--export-3mf"
        )
    });
    if !project_action || args.iter().any(|arg| arg == "--datadir") {
        return Ok(args);
    }
    if let Some(supply) = project_supply()? {
        args.splice(
            0..0,
            [
                "--datadir".to_owned(),
                supply.native_dir.to_string_lossy().into_owned(),
            ],
        );
    }
    Ok(args)
}
