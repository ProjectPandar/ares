mod catalog;

use ares_core::{GenerationMetadata, ProfileSliceRequest, SlicerContext};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub use catalog::PresetCatalog;

#[wasm_bindgen(js_name = parsePresetBundle)]
pub fn parse_preset_bundle(input: Vec<u8>, filename: String) -> Result<String, JsValue> {
    parse_preset_bundle_json(&input, &filename).map_err(|error| JsValue::from_str(&error))
}

pub fn parse_preset_bundle_json(input: &[u8], filename: &str) -> Result<String, String> {
    let bundle = ares_core::parse_preset_bundle(input, filename).map_err(|e| e.to_string())?;
    serde_json::to_string(&bundle)
        .map_err(|error| format!("preset bundle result cannot serialize: {error}"))
}

#[wasm_bindgen(js_name = filterCompatiblePresets)]
pub fn filter_compatible_presets(
    catalog_json: String,
    printer: String,
    process: Option<String>,
) -> Result<String, JsValue> {
    filter_compatible_presets_json(&catalog_json, &printer, process.as_deref())
        .map_err(|error| JsValue::from_str(&error))
}

pub fn filter_compatible_presets_json(
    catalog_json: &str,
    printer: &str,
    process: Option<&str>,
) -> Result<String, String> {
    let filtered = ares_core::filter_compatible_presets(catalog_json, printer, process)
        .map_err(|error| error.to_string())?;
    serde_json::to_string(&filtered)
        .map_err(|error| format!("compatible preset result cannot serialize: {error}"))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileSliceOptions {
    pub(crate) plate: Option<u32>,
    pub(crate) seed: u32,
    /// M854 user edits in the same `{key: token}` shape as
    /// `sliceProjectPlate`'s `overridesJson`; deserialization validates
    /// every entry (shared with `ConfigOverrides::from_json`).
    #[serde(default)]
    overrides: Option<ares_core::ConfigOverrides>,
    /// Model placement (`--center` transform); "none" by default.
    #[serde(default)]
    placement: Option<String>,
}

impl ProfileSliceOptions {
    pub(crate) fn parse(options_json: &str) -> Result<Self, String> {
        let options: ProfileSliceOptions = serde_json::from_str(options_json)
            .map_err(|error| format!("invalid profile slice options JSON: {error}"))?;
        if let Some(placement) = &options.placement {
            ares_core::Placement::parse(placement).map_err(|error| error.to_string())?;
        }
        Ok(options)
    }

    pub(crate) fn overrides(&self) -> ares_core::ConfigOverrides {
        self.overrides.clone().unwrap_or_default()
    }

    pub(crate) fn placement(&self) -> ares_core::Placement {
        self.placement
            .as_deref()
            .and_then(|value| ares_core::Placement::parse(value).ok())
            .unwrap_or(ares_core::Placement::None)
    }
}

#[wasm_bindgen(js_name = sliceWithProfiles)]
pub async fn slice_with_profiles(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<Vec<u8>, JsValue> {
    slice_with_profiles_bytes(
        input,
        &filename,
        &catalog_json,
        &selection_json,
        &options_json,
    )
    .await
    .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_with_profiles_bytes(
    input: Vec<u8>,
    filename: &str,
    catalog_json: &str,
    selection_json: &str,
    options_json: &str,
) -> Result<Vec<u8>, String> {
    let options = ProfileSliceOptions::parse(options_json)?;
    let overrides = options.overrides();
    let request = ProfileSliceRequest {
        filename,
        catalog_json,
        selection_json,
        plate: options.plate,
        overrides: &overrides,
        placement: options.placement(),
    };
    ares_core::slice_with_profiles_validated(
        input,
        request,
        GenerationMetadata::default(),
        SlicerContext::new(options.seed),
    )
    .await
    .map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = sliceWithProfilesProcessed)]
pub async fn slice_with_profiles_processed(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<crate::SliceOutput, JsValue> {
    let output = slice_with_profiles_processed_bytes(
        input,
        &filename,
        &catalog_json,
        &selection_json,
        &options_json,
    )
    .await
    .map_err(|error| JsValue::from_str(&error))?;
    Ok(crate::SliceOutput::from_processed(output))
}

/// Returns the core processed output; palettes derive from its metadata.
pub async fn slice_with_profiles_processed_bytes(
    input: Vec<u8>,
    filename: &str,
    catalog_json: &str,
    selection_json: &str,
    options_json: &str,
) -> Result<ares_core::gcode_processor::ProcessedGCode, String> {
    let options = ProfileSliceOptions::parse(options_json)?;
    let overrides = options.overrides();
    let request = ares_core::ProfileSliceRequest {
        filename,
        catalog_json,
        selection_json,
        plate: options.plate,
        overrides: &overrides,
        placement: options.placement(),
    };
    ares_core::slice_with_profiles_processed_validated(
        input,
        request,
        ares_core::GenerationMetadata::default(),
        ares_core::SlicerContext::new(options.seed),
    )
    .await
    .map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = profileSettings)]
pub async fn profile_settings(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<String, JsValue> {
    profile_settings_json(
        input,
        &filename,
        &catalog_json,
        &selection_json,
        &options_json,
    )
    .await
    .map_err(|error| JsValue::from_str(&error))
}

pub async fn profile_settings_json(
    input: Vec<u8>,
    filename: &str,
    catalog_json: &str,
    selection_json: &str,
    options_json: &str,
) -> Result<String, String> {
    let options = ProfileSliceOptions::parse(options_json)?;
    let overrides = options.overrides();
    let settings = ares_core::profile_settings(
        input,
        ares_core::ProfileSliceRequest {
            filename,
            catalog_json,
            selection_json,
            plate: options.plate,
            overrides: &overrides,
            placement: options.placement(),
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    serde_json::to_string(&settings)
        .map_err(|error| format!("profile settings cannot serialize: {error}"))
}

/// Returns the placed scene (after `placement` is applied) as `SceneData`,
/// reusing the existing scene extraction. Bed shape (`printable_area`) is
/// available in the metadata's `settings.printer` section.
#[wasm_bindgen(js_name = placementScene)]
pub async fn placement_scene(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<crate::SceneData, JsValue> {
    let options =
        ProfileSliceOptions::parse(&options_json).map_err(|error| JsValue::from_str(&error))?;
    let overrides = options.overrides();
    let project = ares_core::profile_placed_project(
        input,
        ares_core::ProfileSliceRequest {
            filename: &filename,
            catalog_json: &catalog_json,
            selection_json: &selection_json,
            plate: options.plate,
            overrides: &overrides,
            placement: options.placement(),
        },
    )
    .await
    .map_err(|error| JsValue::from_str(&error.to_string()))?;
    Ok(crate::scene::project_scene_data(&project))
}
