//! OrcaSlicer vendor profile flattening: resolves a preset's `inherits`
//! chain into a single preset JSON the OrcaSlicer CLI accepts via
//! `--load-settings`/`--load-filaments` (the CLI itself does not resolve
//! inheritance between separately loaded files).

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Map, Value};

#[cfg(test)]
#[path = "presets/tests.rs"]
mod tests;

pub(super) struct VendorProfiles {
    /// preset name → merged (flattened) preset fields
    machine: BTreeMap<String, Map<String, Value>>,
    process: BTreeMap<String, Map<String, Value>>,
    filament: BTreeMap<String, Map<String, Value>>,
}

impl VendorProfiles {
    pub(super) fn load(root: &Path, vendor: &str) -> Result<Self, String> {
        let mut filament = index_kind(root, "OrcaFilamentLibrary", "filament")?;
        let vendor_filament = root.join(vendor).join("filament");
        if vendor_filament.is_dir() {
            filament.extend(index_directory(&vendor_filament)?);
        }
        Ok(Self {
            machine: index_kind(root, vendor, "machine")?,
            process: index_kind(root, vendor, "process")?,
            filament,
        })
    }

    pub(super) fn instantiated_machine_names(&self) -> Vec<String> {
        self.machine
            .iter()
            .filter(|(_, fields)| {
                fields.get("instantiation").and_then(Value::as_str) == Some("true")
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub(super) fn machine(&self, name: &str) -> Result<Map<String, Value>, String> {
        self.flatten(&self.machine, name, "machine")
    }

    pub(super) fn process(&self, name: &str) -> Result<Map<String, Value>, String> {
        self.flatten(&self.process, name, "process")
    }

    pub(super) fn filament(&self, name: &str) -> Result<Map<String, Value>, String> {
        self.flatten(&self.filament, name, "filament")
    }

    pub(super) fn process_exists(&self, name: &str) -> bool {
        self.process.contains_key(name)
    }

    pub(super) fn process_is_compatible(&self, name: &str, printer: &str) -> bool {
        self.preset_is_compatible(&self.process, name, printer)
    }

    pub(super) fn filament_exists(&self, name: &str) -> bool {
        self.filament.contains_key(name)
    }

    pub(super) fn filament_is_compatible(&self, name: &str, printer: &str) -> bool {
        self.preset_is_compatible(&self.filament, name, printer)
    }

    pub(super) fn compatible_process(&self, printer: &str) -> Option<String> {
        self.compatible_preset(&self.process, printer, |name, _| name.contains("0.20mm"))
    }

    pub(super) fn compatible_filament(&self, printer: &str) -> Option<String> {
        self.compatible_preset(&self.filament, printer, |name, fields| {
            name.to_ascii_uppercase().contains("PLA")
                || fields
                    .get("filament_type")
                    .is_some_and(|value| value.to_string().contains("PLA"))
        })
    }

    fn preset_is_compatible(
        &self,
        index: &BTreeMap<String, Map<String, Value>>,
        name: &str,
        printer: &str,
    ) -> bool {
        let Ok(fields) = self.flatten(index, name, "compatible preset") else {
            return false;
        };
        fields
            .get("compatible_printers")
            .and_then(Value::as_array)
            .filter(|printers| !printers.is_empty())
            .is_none_or(|printers| {
                printers
                    .iter()
                    .any(|candidate| candidate.as_str() == Some(printer))
            })
    }

    fn compatible_preset(
        &self,
        index: &BTreeMap<String, Map<String, Value>>,
        printer: &str,
        preferred: impl Fn(&str, &Map<String, Value>) -> bool,
    ) -> Option<String> {
        index
            .keys()
            .filter_map(|name| {
                let fields = self.flatten(index, name, "compatible preset").ok()?;
                if fields.get("instantiation").and_then(Value::as_str) != Some("true") {
                    return None;
                }
                let compatibility = fields
                    .get("compatible_printers")
                    .and_then(Value::as_array)
                    .filter(|printers| !printers.is_empty())
                    .map(|printers| {
                        printers
                            .iter()
                            .any(|candidate| candidate.as_str() == Some(printer))
                    });
                match compatibility {
                    Some(false) => None,
                    exact => Some((
                        if exact == Some(true) { 0 } else { 1 },
                        if preferred(name, &fields) { 0 } else { 1 },
                        name.clone(),
                    )),
                }
            })
            .min()
            .map(|(_, _, name)| name)
    }

    fn flatten(
        &self,
        index: &BTreeMap<String, Map<String, Value>>,
        name: &str,
        kind: &str,
    ) -> Result<Map<String, Value>, String> {
        let entry = index
            .get(name)
            .ok_or_else(|| format!("unknown {kind} preset {name:?}"))?;
        let mut merged = match entry.get("inherits").and_then(Value::as_str) {
            Some(parent) if index.contains_key(parent) => self.flatten(index, parent, kind)?,
            Some(_) | None => Map::new(),
        };
        for (key, value) in entry {
            merged.insert(key.clone(), value.clone());
        }
        merged.remove("inherits");
        Ok(merged)
    }
}

fn index_kind(
    root: &Path,
    vendor: &str,
    kind: &str,
) -> Result<BTreeMap<String, Map<String, Value>>, String> {
    index_directory(&root.join(vendor).join(kind))
}

fn index_directory(dir: &Path) -> Result<BTreeMap<String, Map<String, Value>>, String> {
    let mut pending = vec![dir.to_owned()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries =
            std::fs::read_dir(&directory).map_err(|error| format!("{directory:?}: {error}"))?;
        for entry in entries {
            let path = entry.map_err(|error| error.to_string())?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
                files.push(path);
            }
        }
    }
    files.sort();

    let mut index = BTreeMap::new();
    for path in files {
        let text = std::fs::read_to_string(&path).map_err(|error| format!("{path:?}: {error}"))?;
        let Value::Object(fields) =
            serde_json::from_str(&text).map_err(|error| format!("{path:?}: {error}"))?
        else {
            continue;
        };
        let Some(Value::String(name)) = fields.get("name") else {
            continue;
        };
        index.insert(name.clone(), fields);
    }
    Ok(index)
}
