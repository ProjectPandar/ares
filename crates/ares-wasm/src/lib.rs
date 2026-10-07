mod color_print;
mod colors;
mod filament_legend;
mod moves;
mod profiles;
mod raw_moves;
mod scene;

pub use colors::{option_colors, role_colors, view_types};
pub use moves::{SliceOutput, VertexBuffers, option_types};
pub use raw_moves::MoveBuffers;

use ares_core::{GenerationMetadata, SliceError, SliceOptions};
use wasm_bindgen::prelude::*;

pub use profiles::{
    PresetCatalog, filter_compatible_presets, filter_compatible_presets_json, parse_preset_bundle,
    parse_preset_bundle_json, slice_with_profiles, slice_with_profiles_bytes,
};
pub use scene::{SceneData, load_scene, scene_from_bytes};

#[wasm_bindgen]
pub async fn slice_stl(input: Vec<u8>, options_json: String) -> Result<Vec<u8>, JsValue> {
    slice_stl_bytes(input, &options_json)
        .await
        .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_stl_bytes(input: Vec<u8>, options_json: &str) -> Result<Vec<u8>, String> {
    let options = parse_options(options_json)?;
    ares_core::slice_stl_validated(input, options)
        .await
        .map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = sliceProject)]
pub async fn slice_project(
    input: Vec<u8>,
    overrides_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    slice_project_plate(input, None, overrides_json).await
}

#[wasm_bindgen(js_name = sliceProjectPlate)]
pub async fn slice_project_plate(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let overrides = parse_overrides(overrides_json)?;
    ares_core::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        plate,
        &overrides,
        ares_core::SlicerContext::new(0),
    )
    .await
    .map_err(slice_error_js)
}

/// Palettes derive from processed filament colors and custom G-code items.
#[wasm_bindgen(js_name = sliceProjectPlateProcessed)]
pub async fn slice_project_plate_processed(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<String>,
) -> Result<SliceOutput, JsValue> {
    let overrides = parse_overrides(overrides_json)?;
    let output = ares_core::slice_project_plate_processed_validated(
        input,
        GenerationMetadata::default(),
        plate,
        &overrides,
        ares_core::SlicerContext::new(0),
    )
    .await
    .map_err(slice_error_js)?;
    Ok(SliceOutput::from_processed(output))
}

fn slice_error_js(error: SliceError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// `{key: string|string[]}` → validated `ConfigOverrides` (M854; absent or
/// `undefined` means no overrides).
fn parse_overrides(overrides_json: Option<String>) -> Result<ares_core::ConfigOverrides, JsValue> {
    match overrides_json {
        None => Ok(ares_core::ConfigOverrides::default()),
        Some(json) => ares_core::ConfigOverrides::from_json(json.as_bytes())
            .map_err(|error| JsValue::from_str(&error.to_string())),
    }
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

/// Decodes a `ConfigOption::serialize()`-form string (as stored in 3MF /
/// presets) for option `key` into typed JSON text. Nullable `nil` elements
/// decode to `null`. Throws on unknown key or a malformed value.
#[wasm_bindgen(js_name = decodeConfigValue)]
pub fn decode_config_value(key: &str, serialized: &str) -> Result<String, JsValue> {
    ares_core::decode_config_value_by_key(key, serialized)
        .map_err(|error| JsValue::from_str(&error))
}

/// Encodes typed JSON text (the shape `decodeConfigValue` returns) for
/// option `key` into its `ConfigOption::serialize()` string. Throws on unknown
/// key or a value of the wrong shape.
#[wasm_bindgen(js_name = encodeConfigValue)]
pub fn encode_config_value(key: &str, typed_json: &str) -> Result<String, JsValue> {
    ares_core::encode_config_value_by_key(key, typed_json)
        .map_err(|error| JsValue::from_str(&error))
}

#[cfg(test)]
mod tests;
