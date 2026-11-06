use ares_core::{
    ConfigOverrides, GenerationMetadata, ProfileSliceRequest, SliceError, SlicerContext,
};
use sha2::{Digest, Sha256};
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
    let supply = Supply {
        digest: Sha256::digest(catalog.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        native_dir: parent.join("native"),
        catalog,
        catalog_path: path,
    };
    if supply.digest != receipt.catalog_sha256 {
        return Err("supplied catalog differs from its runtime producer receipt".to_owned());
    }
    if !supply.native_dir.join("system").is_dir() {
        return Err("supplied catalog has no staged Native system tree".to_owned());
    }
    Ok(Some(supply))
}

pub(crate) fn request<'a>(
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
