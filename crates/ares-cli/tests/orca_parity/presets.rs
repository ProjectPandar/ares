//! OrcaSlicer vendor profile flattening: resolves a preset's `inherits`
//! chain into a single preset JSON the OrcaSlicer CLI accepts via
//! `--load-settings`/`--load-filaments` (the CLI itself does not resolve
//! inheritance between separately loaded files).

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Map, Value};

#[cfg(unix)]
#[path = "presets/filesystem_tests.rs"]
mod filesystem_tests;
#[cfg(test)]
#[path = "presets/tests.rs"]
mod tests;

/// `Preset::get_extruder_names_and_keysets` empty set for kinds without
/// stride-2 keys (`Preset.cpp:936-943`).
const NO_VARIANT_KEYS: &[&str] = &[];

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
            .ok_or_else(|| format!("unknown {kind} preset {name:?}"))?
            .clone();
        let mut merged = match entry.get("inherits").and_then(Value::as_str) {
            Some(parent) if index.contains_key(parent) => self.flatten(index, parent, kind)?,
            Some(_) | None => Map::new(),
        };
        if merged.is_empty() {
            // Custom root preset: no parent layout to map onto
            // (`Preset.cpp:1688-1696` default-preset branch).
            for (key, value) in &entry {
                merged.insert(key.clone(), value.clone());
            }
        } else {
            Self::overlay_variant_mapped(&mut merged, &entry, kind);
        }
        merged.remove("inherits");
        Ok(merged)
    }

    /// `DynamicPrintConfig::update_diff_values_to_child_config`
    /// (`PrintConfig.cpp:10209-10298`): the child overlays the parent
    /// through the parent's variant layout. The variant key itself stays
    /// the parent's (`:10263`), scalars and non-variant vectors take the
    /// child's value (`:10285-10289`), and per-variant vectors copy the
    /// child slot matched by variant name (`set_only_diff`,
    /// `Config.hpp:561-584`) when the base length agrees with the layout
    /// (`:10287-10292` falls back to the child value otherwise).
    fn overlay_variant_mapped(
        merged: &mut Map<String, Value>,
        entry: &Map<String, Value>,
        kind: &str,
    ) {
        let (id_name, variant_name, key_set1, key_set2) = match kind {
            "machine" => (
                Some("printer_extruder_id"),
                "printer_extruder_variant",
                ares_core::printer_options_with_variant_1(),
                ares_core::printer_options_with_variant_2(),
            ),
            "process" => (
                Some("print_extruder_id"),
                "print_extruder_variant",
                ares_core::print_options_with_variant(),
                NO_VARIANT_KEYS,
            ),
            "filament" => (
                None,
                "filament_extruder_variant",
                ares_core::filament_options_with_variant(),
                NO_VARIANT_KEYS,
            ),
            // No variant vocabulary (`Preset::get_extruder_names_and_keysets`,
            // `Preset.cpp:941-946`): plain child values.
            _ => (None, "", NO_VARIANT_KEYS, NO_VARIANT_KEYS),
        };
        let variant_list = |fields: &Map<String, Value>| -> Vec<String> {
            fields
                .get(variant_name)
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        let cur_variants = variant_list(merged);
        let target_variants = variant_list(entry);
        let extruder_ids = |fields: &Map<String, Value>| -> Vec<i64> {
            id_name
                .and_then(|name| fields.get(name))
                .and_then(Value::as_array)
                .map(|values| values.iter().filter_map(Value::as_i64).collect::<Vec<_>>())
                .unwrap_or_default()
        };
        let cur_ids = extruder_ids(merged);
        let target_ids = extruder_ids(entry);
        let variant_index =
            Self::matched_variant_index(&cur_variants, &target_variants, &cur_ids, &target_ids);
        for (key, value) in entry {
            if key == "inherits" || key.as_str() == variant_name || Some(key.as_str()) == id_name {
                // The variant keys keep the parent's layout (`:10262-10263`).
                continue;
            }
            let Some(parent_value) = merged.get(key) else {
                merged.insert(key.clone(), value.clone());
                continue;
            };
            if parent_value == value {
                continue;
            }
            let stride = if key_set2.contains(&key.as_str()) {
                2
            } else if key_set1.contains(&key.as_str()) {
                1
            } else {
                0
            };
            let (Value::Array(parent_items), Value::Array(child_items)) = (parent_value, value)
            else {
                merged.insert(key.clone(), value.clone());
                continue;
            };
            if stride == 0 || parent_items.len() != variant_index.len() * stride {
                merged.insert(key.clone(), value.clone());
                continue;
            }
            let mut mapped = parent_items.clone();
            Self::set_only_diff_slots(&mut mapped, child_items, &variant_index, stride);
            merged.insert(key.clone(), Value::Array(mapped));
        }
    }

    /// Parent-variant → child-variant mapping: match by variant name (and
    /// extruder id when the kind carries ids), leaving unmatched parent
    /// slots at -1; an empty child list maps only the first slot
    /// (`PrintConfig.cpp:10242-10273`).
    fn matched_variant_index(
        cur_variants: &[String],
        target_variants: &[String],
        cur_ids: &[i64],
        target_ids: &[i64],
    ) -> Vec<i64> {
        let mut variant_index: Vec<i64> = if cur_variants.is_empty() {
            vec![0]
        } else {
            vec![-1; cur_variants.len()]
        };
        if target_variants.is_empty() {
            variant_index[0] = 0;
            return variant_index;
        }
        for (i, cur) in cur_variants.iter().enumerate() {
            let matched = target_variants.iter().enumerate().find(|(j, target)| {
                cur == *target && (cur_ids.is_empty() || cur_ids.get(i) == target_ids.get(*j))
            });
            if let Some((j, _)) = matched {
                variant_index[i] = j as i64;
            }
        }
        variant_index
    }

    /// `ConfigOptionVector::set_only_diff` (`Config.hpp:561-584`): each
    /// parent slot with a matched child variant copies the child's stride
    /// block when its leading element is not nil.
    fn set_only_diff_slots(
        mapped: &mut [Value],
        child_items: &[Value],
        variant_index: &[i64],
        stride: usize,
    ) {
        for (slot, target) in variant_index.iter().enumerate() {
            let Ok(target) = usize::try_from(*target) else {
                continue;
            };
            if matches!(child_items.get(target * stride), Some(Value::Null) | None) {
                continue;
            }
            let source = target * stride;
            let count = (source + stride).min(child_items.len()) - source;
            let dest = slot * stride;
            mapped[dest..dest + count].clone_from_slice(&child_items[source..source + count]);
        }
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
            if std::fs::metadata(&path)
                .map_err(|error| format!("{path:?}: {error}"))?
                .is_dir()
            {
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
