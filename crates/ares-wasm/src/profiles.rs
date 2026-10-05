use ares_core::{GenerationMetadata, ProfileSliceRequest, SlicerContext};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

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
struct ProfileSliceOptions {
    plate: Option<u32>,
    seed: u32,
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
    let options: ProfileSliceOptions = serde_json::from_str(options_json)
        .map_err(|error| format!("invalid profile slice options JSON: {error}"))?;
    ares_core::slice_with_profiles(
        input,
        ProfileSliceRequest {
            filename,
            catalog_json,
            selection_json,
            plate: options.plate,
        },
        GenerationMetadata::default(),
        SlicerContext::new(options.seed),
    )
    .await
    .map_err(|error| error.to_string())
}
