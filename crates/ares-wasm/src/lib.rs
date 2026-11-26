mod actual_speed_profile;
mod color_print;
mod colors;
mod custom_gcode_overview;
mod direct_project;
mod estimation;
mod filament_legend;
mod heights_widths;
mod moves;
mod panic_hook;
#[cfg(feature = "panic-probe")]
mod panic_probe;
mod plater;
mod profiles;
mod raw_moves;
mod role_times;
mod scene;
mod status;

pub use colors::{option_colors, role_colors, view_types};
pub use direct_project::{
    slice_project_plate_processed_with_catalog, slice_project_plate_processed_with_catalog_bytes,
    slice_project_plate_processed_with_catalog_handle, slice_project_plate_with_catalog,
};
pub use moves::{SliceOutput, VertexBuffers, option_types};
pub use plater::{
    PlaterSession, PlaterSnapshot, create_plater_session_from_stl_files,
    create_plater_session_from_stl_files_with_profiles, create_plater_session_with_profiles,
};
pub use raw_moves::MoveBuffers;

use ares_core::{SliceError, SliceOptions};
use wasm_bindgen::prelude::*;

pub use profiles::{
    PresetCatalog, filter_compatible_presets_json, guide_models, guide_models_json,
    parse_preset_bundle, parse_preset_bundle_json, placement_scene, profile_settings,
    resolve_default_bed_type, slice_with_profiles_bytes, slice_with_profiles_processed,
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

/// The processed slice of `plate`, with the legend Custom G-code overview of
/// that plate's custom G-codes.
pub async fn slice_project_plate_processed_bytes(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<&str>,
) -> Result<SliceOutput, String> {
    slice_project_plate_processed_with_catalog_bytes(input, plate, overrides_json, None).await
}

/// The print-job `.gcode.3mf` of plate 1 of a saved project: `Plater::send_gcode`
/// (`Plater.cpp:21240`) / CLI `--min-save --slice 1 --export-3mf`.
#[wasm_bindgen(js_name = exportPlateSlicedFile)]
pub async fn export_plate_sliced_file(
    input: Vec<u8>,
    filename: String,
) -> Result<Vec<u8>, JsValue> {
    ares_core::export_saved_sliced_3mf(&input, &filename, true, ares_core::SlicerContext::new(0))
        .await
        .map_err(slice_error_js)
}

/// Polls an immediately-ready `ares_core` future without allocating a JS Promise,
/// allowing synchronous `#[wasm_bindgen]` entry points to borrow `&PresetCatalog`.
pub(crate) fn poll_ready<T>(future: impl std::future::Future<Output = T>) -> Result<T, String> {
    let future = std::pin::pin!(future);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    match future.poll(&mut cx) {
        std::task::Poll::Ready(value) => Ok(value),
        std::task::Poll::Pending => Err("slicer future yielded unexpectedly".to_owned()),
    }
}

fn slice_error_js(error: SliceError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// `{key: string|string[]}` → validated `ConfigOverrides` (M854; absent or
/// `undefined` means no overrides).
fn parse_overrides(overrides_json: Option<&str>) -> Result<ares_core::ConfigOverrides, String> {
    match overrides_json {
        None => Ok(ares_core::ConfigOverrides::default()),
        Some(json) => ares_core::ConfigOverrides::from_json(json.as_bytes())
            .map_err(|error| error.to_string()),
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

/// The OrcaSlicer release whose behaviour the core ports (`ORCA_SLICER_COMPATIBILITY_VERSION`).
#[wasm_bindgen(js_name = orcaSlicerVersion)]
pub fn orca_slicer_version() -> String {
    ares_core::ORCA_SLICER_COMPATIBILITY_VERSION.to_owned()
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
