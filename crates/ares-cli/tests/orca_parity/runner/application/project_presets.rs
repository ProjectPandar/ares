use ares_core::{
    ConfigOverrides, GenerationMetadata, ProcessedGCode, ProfileSliceRequest, SliceError,
    SlicerContext,
};
use std::{fs, path::PathBuf, sync::OnceLock};

pub(crate) struct Supply {
    pub(crate) catalog: String,
    pub(crate) native_dir: PathBuf,
    pub(crate) digest: String,
    pub(crate) catalog_path: PathBuf,
}

pub(crate) fn project_supply() -> Result<Option<&'static Supply>, SliceError> {
    static SUPPLY: OnceLock<Result<Option<Supply>, String>> = OnceLock::new();
    SUPPLY
        .get_or_init(read_supply)
        .as_ref()
        .map(Option::as_ref)
        .map_err(|error| SliceError::InvalidInput(error.clone()))
}

fn read_supply() -> Result<Option<Supply>, String> {
    let Some(path) = std::env::var_os("ARES_PARITY_PROJECT_PRESET_CATALOG") else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    let catalog = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read supplied catalog {}: {error}", path.display()))?;
    let parent = path
        .parent()
        .ok_or_else(|| "catalog has no supply directory".to_owned())?;
    #[derive(serde::Deserialize)]
    struct Receipt {
        catalog_sha256: String,
    }
    let receipt: Receipt = serde_json::from_slice(
        &fs::read(parent.join("source-hashes.json")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let digest = crate::runner::cache_identity::raw_digest(catalog.as_bytes());
    if digest != receipt.catalog_sha256 {
        return Err("supplied catalog differs from its runtime producer receipt".to_owned());
    }
    let native_dir = parent.join("native");
    if !native_dir.join("system").is_dir() {
        return Err("supplied catalog has no staged Native system tree".to_owned());
    }
    Ok(Some(Supply {
        catalog,
        native_dir,
        digest,
        catalog_path: path,
    }))
}

fn request<'a>(
    supply: &'a Supply,
    plate: Option<u32>,
    overrides: &'a ConfigOverrides,
) -> ProfileSliceRequest<'a> {
    ProfileSliceRequest {
        filename: "project.3mf",
        catalog_json: &supply.catalog,
        selection_json: r#"{"from_project":true,"uptodate":false}"#,
        plate,
        overrides,
        placement: ares_core::Placement::None,
    }
}

pub(crate) async fn slice_project(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    context: SlicerContext,
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
    context: SlicerContext,
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

pub(crate) async fn slice_project_plate_validated(
    input: impl AsRef<[u8]>,
    metadata: GenerationMetadata,
    plate: Option<u32>,
    overrides: &ConfigOverrides,
    context: SlicerContext,
) -> Result<Vec<u8>, SliceError> {
    match project_supply()? {
        None => {
            ares_core::slice_project_plate_validated(input, metadata, plate, overrides, context)
                .await
        }
        Some(supply) => {
            ares_core::slice_with_profiles_validated(
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
    context: SlicerContext,
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
    context: SlicerContext,
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
    context: SlicerContext,
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
