use ares_core::{GenerationMetadata, PresetCatalogHandle, SlicerContext};
use wasm_bindgen::prelude::*;

use crate::{SliceOutput, custom_gcode_overview, parse_overrides};

#[wasm_bindgen(js_name = sliceProjectWithCatalog)]
pub async fn slice_project_with_catalog(
    input: Vec<u8>,
    catalog_json: Option<String>,
    overrides_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    slice_project_plate_with_catalog(input, None, overrides_json, catalog_json).await
}

#[wasm_bindgen(js_name = sliceProjectPlateWithCatalog)]
pub async fn slice_project_plate_with_catalog(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<String>,
    catalog_json: Option<String>,
) -> Result<Vec<u8>, JsValue> {
    let overrides =
        parse_overrides(overrides_json.as_deref()).map_err(|e| JsValue::from_str(&e))?;
    let catalog = catalog_json
        .as_deref()
        .map(PresetCatalogHandle::from_json)
        .transpose()
        .map_err(crate::slice_error_js)?;
    ares_core::slice_project_plate_validated_with_catalog(
        input,
        ares_core::DirectProjectSliceRequest {
            plate,
            overrides: &overrides,
            system_catalog: catalog.as_ref(),
        },
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .map_err(crate::slice_error_js)
}

#[wasm_bindgen(js_name = sliceProjectPlateProcessedWithCatalog)]
pub async fn slice_project_plate_processed_with_catalog(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<String>,
    catalog_json: Option<String>,
) -> Result<SliceOutput, JsValue> {
    slice_project_plate_processed_with_catalog_bytes(
        input,
        plate,
        overrides_json.as_deref(),
        catalog_json.as_deref(),
    )
    .await
    .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_project_plate_processed_with_catalog_bytes(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<&str>,
    catalog_json: Option<&str>,
) -> Result<SliceOutput, String> {
    let overrides = parse_overrides(overrides_json)?;
    let catalog = catalog_json
        .map(PresetCatalogHandle::from_json)
        .transpose()
        .map_err(|error| error.to_string())?;
    let output = ares_core::slice_project_plate_processed_validated_with_catalog(
        &input,
        ares_core::DirectProjectSliceRequest {
            plate,
            overrides: &overrides,
            system_catalog: catalog.as_ref(),
        },
        GenerationMetadata::default(),
        SlicerContext::new(0),
    )
    .await
    .map_err(|error| error.to_string())?;
    let custom_gcodes = custom_gcode_overview::plate_custom_gcodes(&input, plate)
        .map_err(|error| error.to_string())?;
    Ok(SliceOutput::from_processed(output).with_custom_gcodes(&custom_gcodes))
}
