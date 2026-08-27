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
        Ok(Self {
            machine: index_kind(root, vendor, "machine")?,
            process: index_kind(root, vendor, "process")?,
            filament: index_kind(root, vendor, "filament")?,
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

    pub(super) fn filament_exists(&self, name: &str) -> bool {
        self.filament.contains_key(name)
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
            Some(parent) => self.flatten(index, parent, kind)?,
            None => Map::new(),
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
    let dir = root.join(vendor).join(kind);
    let mut index = BTreeMap::new();
    let entries = std::fs::read_dir(&dir).map_err(|error| format!("{dir:?}: {error}"))?;
    for entry in entries {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
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
