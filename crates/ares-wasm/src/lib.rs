mod profiles;
mod scene;

use ares_core::{GenerationMetadata, SliceError, SliceOptions};
use wasm_bindgen::prelude::*;

pub use profiles::{slice_with_profiles, slice_with_profiles_bytes};
pub use scene::{SceneData, load_scene, scene_from_bytes};

#[wasm_bindgen]
pub async fn slice_stl(input: Vec<u8>, options_json: String) -> Result<Vec<u8>, JsValue> {
    slice_stl_bytes(input, &options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_stl_bytes(input: Vec<u8>, options_json: &str) -> Result<Vec<u8>, String> {
    let options = parse_options(options_json)?;
    ares_core::slice(input, options)
        .await
        .map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = sliceProject)]
pub async fn slice_project(input: Vec<u8>) -> Result<Vec<u8>, JsValue> {
    slice_project_plate(input, None).await
}

#[wasm_bindgen(js_name = sliceProjectPlate)]
pub async fn slice_project_plate(input: Vec<u8>, plate: Option<u32>) -> Result<Vec<u8>, JsValue> {
    ares_core::slice_project_plate(
        input,
        GenerationMetadata::default(),
        plate,
        ares_core::SlicerContext::new(0),
    )
    .await
    .map_err(slice_error_js)
}

fn slice_error_js(error: SliceError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

fn parse_options(options_json: &str) -> Result<SliceOptions, String> {
    serde_json::from_str(options_json).map_err(|error| format!("invalid options JSON: {error}"))
}

/// Serialized `PrintConfigDef` metadata (UTF-8 JSON bytes) for the settings UI.
/// Generated from upstream `PrintConfig.cpp`; see scripts/gen-configdef.mjs.
#[wasm_bindgen(js_name = printConfigDef)]
pub fn print_config_def() -> Vec<u8> {
    ares_core::print_config_def_json().to_vec()
}

#[cfg(test)]
mod tests;
