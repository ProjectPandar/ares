use std::sync::OnceLock;

use super::PresetCatalog;

fn push_comma_separated(out: &mut String, items: &[String]) {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(item);
    }
}

impl PresetCatalog {
    pub fn from_json_str(catalog_json: String) -> Result<Self, String> {
        let handle = ares_core::PresetCatalogHandle::from_json(&catalog_json)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            handle,
            vendors_json: Vec::new(),
            imports_json: Vec::new(),
            cached_json: OnceLock::from(catalog_json),
        })
    }

    pub(crate) fn catalog_json(&self) -> &str {
        self.cached_json.get_or_init(|| {
            let vendors_len: usize = self.vendors_json.iter().map(String::len).sum();
            let imports_len: usize = self.imports_json.iter().map(String::len).sum();
            let mut json = String::with_capacity(
                24 + vendors_len + self.vendors_json.len() + imports_len + self.imports_json.len(),
            );
            json.push_str(r#"{"vendors":["#);
            push_comma_separated(&mut json, &self.vendors_json);
            json.push_str(r#"],"imports":["#);
            push_comma_separated(&mut json, &self.imports_json);
            json.push_str("]}");
            json
        })
    }

    pub fn load_preset_bundle_json(&self) -> Result<(), String> {
        ares_core::load_preset_bundle(self.catalog_json()).map_err(|error| error.to_string())
    }

    pub fn set_installed_printers_json(&mut self, installed_json: &str) -> Result<(), String> {
        let installed = serde_json::from_str(installed_json)
            .map_err(|error| format!("invalid installed printers JSON: {error}"))?;
        self.handle.set_installed_printers(installed);
        Ok(())
    }

    pub fn system_printer_json(&self, model: &str, variant: &str) -> String {
        serde_json::to_string(&self.handle.system_printer(model, variant))
            .unwrap_or_else(|_| "null".to_owned())
    }

    pub fn compatible_print_json(
        &self,
        printer: &str,
        current: Option<&str>,
    ) -> Result<String, String> {
        let print = self
            .handle
            .compatible_print(printer, current)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&print)
            .map_err(|error| format!("compatible print cannot serialize: {error}"))
    }

    pub fn guide_filaments_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.handle.guide_filaments())
            .map_err(|error| format!("guide filaments cannot serialize: {error}"))
    }

    pub fn normalize_filament_slots_json(
        &self,
        printer: &str,
        state_json: &str,
    ) -> Result<String, String> {
        let state: ares_core::FilamentSlotState = serde_json::from_str(state_json)
            .map_err(|error| format!("invalid filament slot state: {error}"))?;
        let state = self
            .handle
            .normalize_filament_slots(printer, state)
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&state)
            .map_err(|error| format!("filament slot state cannot serialize: {error}"))
    }

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

    pub fn printers_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.handle.printers())
            .map_err(|error| format!("printer listing cannot serialize: {error}"))
    }

    pub fn add_vendor_json(&mut self, vendor_json: &[u8]) -> Result<(), String> {
        self.handle
            .add_vendor(vendor_json)
            .map_err(|e| e.to_string())?;
        let trimmed = vendor_json
            .strip_prefix(b"\xef\xbb\xbf")
            .unwrap_or(vendor_json);
        let text = std::str::from_utf8(trimmed)
            .map_err(|error| format!("invalid vendor profile UTF-8: {error}"))?;
        self.vendors_json.push(text.to_owned());
        self.cached_json.take();
        Ok(())
    }

    pub fn add_import_json(&mut self, input: &[u8], filename: &str) -> Result<(), String> {
        let bundle = ares_core::parse_preset_bundle(input, filename).map_err(|e| e.to_string())?;
        let serialized = serde_json::to_string(&bundle)
            .map_err(|error| format!("preset bundle result cannot serialize: {error}"))?;
        self.handle
            .add_import(input, filename)
            .map_err(|e| e.to_string())?;
        self.imports_json.push(serialized);
        self.cached_json.take();
        Ok(())
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
