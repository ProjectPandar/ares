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
    /// Display palettes for the processed export (`sliceProjectPlateProcessed`
    /// keeps them as arguments; the arity limit folds them into the options
    /// document here).
    #[serde(default)]
    tool_colors: Option<Vec<String>>,
    #[serde(default)]
    color_print_colors: Option<Vec<String>>,
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
    fn parse(options_json: &str) -> Result<Self, String> {
        let options: ProfileSliceOptions = serde_json::from_str(options_json)
            .map_err(|error| format!("invalid profile slice options JSON: {error}"))?;
        if let Some(placement) = &options.placement {
            ares_core::Placement::parse(placement).map_err(|error| error.to_string())?;
        }
        Ok(options)
    }

    fn overrides(&self) -> ares_core::ConfigOverrides {
        self.overrides.clone().unwrap_or_default()
    }

    fn placement(&self) -> ares_core::Placement {
        self.placement
            .as_deref()
            .and_then(|value| ares_core::Placement::parse(value).ok())
            .unwrap_or(ares_core::Placement::None)
    }
}

#[cfg(test)]
pub(crate) fn parse_profile_slice_overrides_for_test(
    options_json: &str,
) -> Result<ares_core::ConfigOverrides, String> {
    Ok(ProfileSliceOptions::parse(options_json)?.overrides())
}

#[cfg(test)]
pub(crate) fn parse_profile_slice_options_for_test(
    options_json: &str,
) -> Result<ares_core::Placement, String> {
    Ok(ProfileSliceOptions::parse(options_json)?.placement())
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

/// Stateful preset catalog (`PresetBundle` holding semantics): vendor data is
/// parsed once per `addVendor` and filtering reuses the loaded records.
#[wasm_bindgen]
pub struct PresetCatalog {
    handle: ares_core::PresetCatalogHandle,
}

impl Default for PresetCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl PresetCatalog {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            handle: ares_core::PresetCatalogHandle::new(),
        }
    }

    /// One vendor envelope `{index, files}` (Uint8Array JSON bytes).
    #[wasm_bindgen(js_name = addVendor)]
    pub fn add_vendor(&mut self, vendor_json: Vec<u8>) -> Result<(), JsValue> {
        self.add_vendor_json(&vendor_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// An Orca preset JSON or `.orca_*`/`.zip` import.
    #[wasm_bindgen(js_name = addImport)]
    pub fn add_import(&mut self, input: Vec<u8>, filename: String) -> Result<(), JsValue> {
        self.add_import_json(&input, &filename)
            .map_err(|error| JsValue::from_str(&error))
    }

    #[wasm_bindgen(js_name = filterCompatible)]
    pub fn filter_compatible(
        &self,
        printer: String,
        process: Option<String>,
    ) -> Result<String, JsValue> {
        self.filter_compatible_json(&printer, process.as_deref())
            .map_err(|error| JsValue::from_str(&error))
    }

    /// Selectable printers grouped by vendor, as JSON
    /// `[{vendor, name, printer_model, printer_variant}]`.
    #[wasm_bindgen(js_name = printers)]
    pub fn printers(&self) -> Result<String, JsValue> {
        self.printers_json()
            .map_err(|error| JsValue::from_str(&error))
    }

    /// System printer models deduplicated by first occurrence, as JSON
    /// `[{model}]` (`PresetComboBoxes.cpp:1243-1251`).
    #[wasm_bindgen(js_name = printerModels)]
    pub fn printer_models(&self) -> Result<String, JsValue> {
        self.printer_models_json()
            .map_err(|error| JsValue::from_str(&error))
    }

    /// `PresetBundle::get_similar_printer_preset`: resolves a model/variant
    /// pair to a concrete preset name (JSON string or null).
    #[wasm_bindgen(js_name = similarPrinterPreset)]
    pub fn similar_printer_preset(
        &self,
        model: String,
        variant: String,
        current: String,
    ) -> String {
        self.similar_printer_preset_json(&model, &variant, &current)
    }
}

impl PresetCatalog {
    pub fn printers_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.handle.printers())
            .map_err(|error| format!("printer listing cannot serialize: {error}"))
    }

    pub fn add_vendor_json(&mut self, vendor_json: &[u8]) -> Result<(), String> {
        self.handle
            .add_vendor(vendor_json)
            .map_err(|e| e.to_string())
    }

    pub fn add_import_json(&mut self, input: &[u8], filename: &str) -> Result<(), String> {
        self.handle
            .add_import(input, filename)
            .map_err(|e| e.to_string())
    }

    pub fn filter_compatible_json(
        &self,
        printer: &str,
        process: Option<&str>,
    ) -> Result<String, String> {
        let filtered = self
            .handle
            .filter_compatible(printer, process)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&filtered)
            .map_err(|error| format!("compatible preset result cannot serialize: {error}"))
    }
}

#[wasm_bindgen(js_name = sliceWithProfilesProcessed)]
pub async fn slice_with_profiles_processed(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<crate::SliceOutput, JsValue> {
    let (output, tool_colors, color_print_colors) = slice_with_profiles_processed_bytes(
        input,
        &filename,
        &catalog_json,
        &selection_json,
        &options_json,
    )
    .await
    .map_err(|error| JsValue::from_str(&error))?;
    Ok(crate::SliceOutput::from_processed(
        output,
        &tool_colors,
        &color_print_colors,
    ))
}

/// Returns the processed output plus the palette lists the options document
/// carried (empty defaults match `sliceProjectPlateProcessed`).
pub async fn slice_with_profiles_processed_bytes(
    input: Vec<u8>,
    filename: &str,
    catalog_json: &str,
    selection_json: &str,
    options_json: &str,
) -> Result<
    (
        ares_core::gcode_processor::ProcessedGCode,
        Vec<String>,
        Vec<String>,
    ),
    String,
> {
    let options = ProfileSliceOptions::parse(options_json)?;
    let overrides = options.overrides();
    let tool_colors = options.tool_colors.clone().unwrap_or_default();
    let color_print_colors = options.color_print_colors.clone().unwrap_or_default();
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
    .map(|output| (output, tool_colors, color_print_colors))
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

#[cfg(test)]
pub(crate) async fn profile_settings_json_for_test(
    input: Vec<u8>,
    filename: &str,
    catalog_json: &str,
    selection_json: &str,
    options_json: &str,
) -> Result<String, String> {
    profile_settings_json(input, filename, catalog_json, selection_json, options_json).await
}

impl PresetCatalog {
    pub fn printer_models_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.handle.printer_models())
            .map_err(|error| format!("printer model listing cannot serialize: {error}"))
    }

    pub fn similar_printer_preset_json(&self, model: &str, variant: &str, current: &str) -> String {
        serde_json::to_string(&self.handle.similar_printer_preset(model, variant, current))
            .unwrap_or_else(|_| "null".to_owned())
    }
}
