use ares_core::{GenerationMetadata, PresetCatalogHandle, SlicerContext};
use wasm_bindgen::prelude::*;

use crate::{PresetCatalog, SliceOutput, parse_overrides};

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

/// Reuses a loaded `PresetCatalog` handle directly without re-parsing `catalog_json`.
#[wasm_bindgen(js_name = sliceProjectPlateProcessedWithCatalogHandle)]
pub fn slice_project_plate_processed_with_catalog_handle(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<String>,
    catalog: &PresetCatalog,
) -> Result<SliceOutput, JsValue> {
    crate::poll_ready(slice_project_plate_processed_with_handle(
        &input,
        plate,
        overrides_json.as_deref(),
        Some(&catalog.handle),
    ))
    .and_then(|res| res)
    .map_err(|error| JsValue::from_str(&error))
}

pub async fn slice_project_plate_processed_with_catalog_bytes(
    input: Vec<u8>,
    plate: Option<u32>,
    overrides_json: Option<&str>,
    catalog_json: Option<&str>,
) -> Result<SliceOutput, String> {
    let catalog = catalog_json
        .map(PresetCatalogHandle::from_json)
        .transpose()
        .map_err(|error| error.to_string())?;
    slice_project_plate_processed_with_handle(&input, plate, overrides_json, catalog.as_ref()).await
}

async fn slice_project_plate_processed_with_handle(
    input: &[u8],
    plate: Option<u32>,
    overrides_json: Option<&str>,
    catalog: Option<&PresetCatalogHandle>,
) -> Result<SliceOutput, String> {
    let overrides = parse_overrides(overrides_json)?;
    let (output, custom_gcodes) =
        ares_core::slice_project_plate_processed_validated_with_custom_gcodes(
            input,
            ares_core::DirectProjectSliceRequest {
                plate,
                overrides: &overrides,
                system_catalog: catalog,
            },
            GenerationMetadata::default(),
            SlicerContext::new(0),
        )
        .await
        .map_err(|error| error.to_string())?;
    Ok(SliceOutput::from_processed(output).with_custom_gcodes(&custom_gcodes))
}
