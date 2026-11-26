mod methods;

use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

/// Stateful preset catalog (`PresetBundle` holding semantics): vendor data is
/// parsed once per `addVendor` / `fromJson` and reused across filtering,
/// profile slicing, and plater session operations.
#[wasm_bindgen]
pub struct PresetCatalog {
    pub(crate) handle: ares_core::PresetCatalogHandle,
    vendors_json: Vec<String>,
    imports_json: Vec<String>,
    cached_json: OnceLock<String>,
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
            vendors_json: Vec::new(),
            imports_json: Vec::new(),
            cached_json: OnceLock::new(),
        }
    }

    /// Parses one `{vendors, imports}` JSON string into a reusable handle.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(catalog_json: String) -> Result<PresetCatalog, JsValue> {
        Self::from_json_str(catalog_json).map_err(|error| JsValue::from_str(&error))
    }

    /// Loads and pads this catalog's presets once (`PresetBundle::load_presets`),
    /// so later profile and project loads reuse them without copying JSON across WASM.
    #[wasm_bindgen(js_name = loadPresetBundle)]
    pub fn load_preset_bundle(&self) -> Result<(), JsValue> {
        self.load_preset_bundle_json()
            .map_err(|error| JsValue::from_str(&error))
    }

    /// One vendor envelope `{id, index, files}` (Uint8Array JSON bytes).
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

    /// System printer models in "System presets" group order, as JSON
    /// `[{model}]` (`PresetComboBoxes.cpp:1243-1251, 1427`).
    #[wasm_bindgen(js_name = printerModels)]
    pub fn printer_models(&self) -> Result<String, JsValue> {
        self.printer_models_json()
            .map_err(|error| JsValue::from_str(&error))
    }

    /// `PresetBundle::get_similar_printer_preset`: resolves a model/variant
    /// pair to a concrete preset name (JSON string or null), given the
    /// selected printer's name and `printer_variant`.
    #[wasm_bindgen(js_name = similarPrinterPreset)]
    pub fn similar_printer_preset(
        &self,
        model: String,
        variant: String,
        current_name: String,
        current_variant: String,
    ) -> String {
        self.similar_printer_preset_json(&model, &variant, &current_name, &current_variant)
    }

    /// Resolved values (after `inherits`) of the named presets of `kind`
    /// ("machine" | "process" | "filament"), as JSON `[{key: value}]` in
    /// `names` order; `names_json` and `keys_json` are JSON string arrays.
    #[wasm_bindgen(js_name = presetValues)]
    pub fn preset_values(
        &self,
        kind: String,
        names_json: String,
        keys_json: String,
    ) -> Result<String, JsValue> {
        self.preset_values_json(&kind, &names_json, &keys_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// Vendor, bed type, default material and nozzle data of a
    /// `printer_model`, as JSON `{vendor, not_support_bed_types,
    /// default_materials, nozzle_diameters}`.
    #[wasm_bindgen(js_name = printerModelInfo)]
    pub fn printer_model_info(&self, model: String) -> Result<String, JsValue> {
        self.printer_model_info_json(&model)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// PresetBundle minimum and flush-layout normalization for a printer selection.
    #[wasm_bindgen(js_name = normalizeFilamentSlots)]
    pub fn normalize_filament_slots(
        &self,
        printer: String,
        state_json: String,
    ) -> Result<String, JsValue> {
        self.normalize_filament_slots_json(&printer, &state_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// The installed filament names (JSON string array) seeded like
    /// `PresetBundle::load_installed_filaments`, as a sorted JSON array.
    #[wasm_bindgen(js_name = installedFilaments)]
    pub fn installed_filaments(&self, installed_json: String) -> Result<String, JsValue> {
        self.installed_filaments_json(&installed_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// `PresetBundle::update_compatible(Always)` over the filament slots, as a JSON array.
    #[wasm_bindgen(js_name = selectCompatibleFilaments)]
    pub fn select_compatible_filaments(
        &self,
        printer: String,
        process: Option<String>,
        filaments_json: String,
        installed_json: String,
    ) -> Result<String, JsValue> {
        self.select_compatible_filaments_json(
            (&printer, process.as_deref()),
            &filaments_json,
            &installed_json,
        )
        .map_err(|error| JsValue::from_str(&error))
    }

    /// Sets the `AppConfig` vendors section, JSON `{vendor: {model: [variant]}}`. The printer
    /// listings and `installedFilaments` then follow `set_visible_from_appconfig`.
    #[wasm_bindgen(js_name = setInstalledPrinters)]
    pub fn set_installed_printers(&mut self, installed_json: String) -> Result<(), JsValue> {
        self.set_installed_printers_json(&installed_json)
            .map_err(|error| JsValue::from_str(&error))
    }

    /// `find_system_preset_by_model_and_variant`: the preset name (JSON string or null).
    #[wasm_bindgen(js_name = systemPrinter)]
    pub fn system_printer(&self, model: String, variant: String) -> String {
        self.system_printer_json(&model, &variant)
    }

    /// The print a selection of `printer` holds (`PresetBundle::update_compatible` with
    /// `Always`): `current` while compatible, else the preferred compatible print, as a JSON
    /// string or null.
    #[wasm_bindgen(js_name = compatiblePrint)]
    pub fn compatible_print(
        &self,
        printer: String,
        current: Option<String>,
    ) -> Result<String, JsValue> {
        self.compatible_print_json(&printer, current.as_deref())
            .map_err(|error| JsValue::from_str(&error))
    }

    /// The setup guide's system filaments, JSON
    /// `[{name, vendor, filament_type, models: [[printer_model, printer_variant]]}]`.
    #[wasm_bindgen(js_name = guideFilaments)]
    pub fn guide_filaments(&self) -> Result<String, JsValue> {
        self.guide_filaments_json()
            .map_err(|error| JsValue::from_str(&error))
    }
}
