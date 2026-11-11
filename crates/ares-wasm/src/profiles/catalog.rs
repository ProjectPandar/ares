use wasm_bindgen::prelude::*;

/// Stateful preset catalog (`PresetBundle` holding semantics): vendor data is
/// parsed once per `addVendor` and filtering reuses the loaded records.
#[wasm_bindgen]
pub struct PresetCatalog {
    pub(crate) handle: ares_core::PresetCatalogHandle,
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
}

impl PresetCatalog {
    pub fn preset_values_json(
        &self,
        kind: &str,
        names_json: &str,
        keys_json: &str,
    ) -> Result<String, String> {
        let kind = match kind {
            "machine" => ares_core::ProfileKind::Machine,
            "process" => ares_core::ProfileKind::Process,
            "filament" => ares_core::ProfileKind::Filament,
            _ => return Err(format!("unknown preset kind '{kind}'")),
        };
        let names: Vec<String> = serde_json::from_str(names_json)
            .map_err(|error| format!("invalid preset names JSON: {error}"))?;
        let keys: Vec<String> = serde_json::from_str(keys_json)
            .map_err(|error| format!("invalid option keys JSON: {error}"))?;
        let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
        let values = self
            .handle
            .preset_values(kind, &names, &keys)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&values)
            .map_err(|error| format!("preset values cannot serialize: {error}"))
    }

    pub fn printer_model_info_json(&self, model: &str) -> Result<String, String> {
        let info = self
            .handle
            .printer_model_info(model)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&info)
            .map_err(|error| format!("printer model info cannot serialize: {error}"))
    }

    pub fn installed_filaments_json(&self, installed_json: &str) -> Result<String, String> {
        let installed: Vec<String> = serde_json::from_str(installed_json)
            .map_err(|error| format!("invalid installed filaments JSON: {error}"))?;
        let installed = self
            .handle
            .installed_filaments(&installed)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&installed)
            .map_err(|error| format!("installed filaments cannot serialize: {error}"))
    }

    pub fn select_compatible_filaments_json(
        &self,
        selection: (&str, Option<&str>),
        filaments_json: &str,
        installed_json: &str,
    ) -> Result<String, String> {
        let filaments: Vec<String> = serde_json::from_str(filaments_json)
            .map_err(|error| format!("invalid filaments JSON: {error}"))?;
        let installed: Vec<String> = serde_json::from_str(installed_json)
            .map_err(|error| format!("invalid installed filaments JSON: {error}"))?;
        let selected = self
            .handle
            .select_compatible_filaments(selection, &filaments, &installed)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&selected)
            .map_err(|error| format!("selected filaments cannot serialize: {error}"))
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

impl PresetCatalog {
    pub fn printer_models_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.handle.printer_models())
            .map_err(|error| format!("printer model listing cannot serialize: {error}"))
    }

    pub fn similar_printer_preset_json(
        &self,
        model: &str,
        variant: &str,
        current_name: &str,
        current_variant: &str,
    ) -> String {
        let current = ares_core::CurrentPrinter {
            name: current_name,
            printer_variant: current_variant,
        };
        serde_json::to_string(&self.handle.similar_printer_preset(model, variant, current))
            .unwrap_or_else(|_| "null".to_owned())
    }
}
