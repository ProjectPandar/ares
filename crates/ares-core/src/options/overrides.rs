//! User option overrides for project re-slice (M854).
//!
//! Mirrors the upstream GUI edit flow (`slic3r/GUI/GUI.cpp:113
//! change_opt_value` → `libslic3r/Config.cpp:573-620
//! ConfigBase::set_deserialize`): callers pass `{key: string|string[]}` in
//! the same serialized shape as `Metadata/project_settings.config`; each
//! entry is validated at this public boundary (unknown key, malformed
//! token, out-of-range numeric value) and then applied as last-write-wins
//! on the project-settings layer.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::{
    model_config_deserialize::{WireValue, decode_wire},
    registry::{OptionValueKind, option_definition, option_metadata},
    typed_legacy::transform_obsolete,
};
use crate::{
    SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

/// Boundary-validated `{key: serialized value}` map. `Default`/empty is a
/// no-op: slicing with no overrides is byte-identical to the unedited
/// project.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConfigOverrides {
    /// Per-key element tokens in the document's array-of-strings shape.
    /// Scalar options carry exactly one token; vector options carry one
    /// token per element.
    values: BTreeMap<String, OverrideValue>,
}

#[derive(Clone, Debug, PartialEq)]
struct OverrideValue {
    tokens: Vec<String>,
    cli_numbers: Vec<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OverrideEntry {
    Text(String),
    List(Vec<String>),
}

impl ConfigOverrides {
    /// An empty override set; slicing through it is byte-identical to the
    /// unedited project.
    pub(crate) const fn empty() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Parses the overrides JSON object and validates every entry.
    /// Values are strings or arrays of strings, matching
    /// `project_settings.config` (`ConfigBase::set_deserialize` serialized
    /// tokens).
    pub fn from_json(json: &[u8]) -> Result<Self, SliceError> {
        let entries: BTreeMap<String, OverrideEntry> =
            serde_json::from_slice(json).map_err(|error| {
                SliceError::InvalidInput(format!("invalid overrides JSON: {error}"))
            })?;
        Self::from_entries(entries)
    }

    fn from_entries(entries: BTreeMap<String, OverrideEntry>) -> Result<Self, SliceError> {
        let mut values = BTreeMap::new();
        for (key, entry) in entries {
            let tokens = match entry {
                OverrideEntry::Text(text) => vec![text],
                OverrideEntry::List(items) => items,
            };
            values.insert(key.clone(), validate_entry(&key, tokens)?);
        }
        Ok(Self { values })
    }

    /// `OrcaSlicer.cpp:7198,7209-7214`: validate caller config before loading input.
    /// C1 owns only these ten `PrintConfig.cpp:10340-10457` threshold keys.
    pub fn validate_cli_config(&self) -> Result<(), SliceError> {
        let errors = self
            .values
            .iter()
            .filter_map(|(key, value)| {
                cli_threshold_error(key, &value.cli_numbers)
                    .map(|message| format!("{key}: {message}"))
            })
            .collect::<Vec<_>>();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(SliceError::InvalidInput(errors.join("\n")))
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Applies the overrides onto the raw project-settings view
    /// (`ConfigBase::apply_only`, `Config.cpp:461`: later values win).
    pub(crate) fn apply(&self, raw: &mut ProjectSettingsRaw) {
        for (key, override_value) in &self.values {
            let tokens = &override_value.tokens;
            let definition = option_definition(key).expect("validated override key");
            let value = if is_scalar(definition.kind) {
                RawConfigValue::Text(tokens[0].clone())
            } else {
                RawConfigValue::List(
                    tokens
                        .iter()
                        .map(|token| RawConfigValue::Text(token.clone()))
                        .collect(),
                )
            };
            raw.insert(key.clone(), value);
        }
    }
}

fn cli_threshold_error(key: &str, numbers: &[f64]) -> Option<String> {
    let threshold = match key {
        "filament_diameter" => Some(1.0),
        "filament_flow_ratio" => Some(0.0),
        _ => None,
    };
    if let Some(threshold) = threshold {
        if !numbers
            .iter()
            .any(|&value| value < threshold || (threshold == 0.0 && value == 0.0))
        {
            return None;
        }
        return Some(format!(
            "invalid value {}",
            numbers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    if !matches!(
        key,
        "layer_height"
            | "initial_layer_print_height"
            | "bridge_flow"
            | "internal_bridge_flow"
            | "extruder_clearance_radius"
            | "extruder_clearance_height_to_rod"
            | "extruder_clearance_height_to_lid"
            | "nozzle_height"
    ) {
        return None;
    }
    let value = numbers[0];
    (value <= 0.0 || (key == "layer_height" && (value % 0.000001).abs() > 1e-4))
        .then(|| format!("invalid value {value:.6}"))
}

/// Validates one `{key: tokens}` entry and returns the per-element tokens
/// to store (`Config.cpp set_deserialize` error ordering: unknown key →
/// malformed token → out-of-range value → typed deserialize failure).
fn validate_entry(key: &str, items: Vec<String>) -> Result<OverrideValue, SliceError> {
    // `transform_obsolete` mirrors the obsolete-key handling the document
    // deserializer applies; overrides must name live option keys.
    let Some(definition) = option_definition(key) else {
        if transform_obsolete(key).is_some() {
            return Err(invalid(key, "obsolete Orca option key"));
        }
        return Err(invalid(key, "unknown Orca option key"));
    };

    // A plain string for a vector kind is the upstream `serialize()` comma
    // token and is decoded with `wire::decode` (which splits on `,` / `#`).
    // Array items are already per-element tokens.
    let decoded = if items.len() == 1 {
        decode_wire(definition.kind, &items[0]).map_err(|reason| invalid(key, &reason))?
    } else {
        if is_scalar(definition.kind) {
            return Err(invalid(key, "scalar option expects a single value"));
        }
        WireValue::Sequence(items)
    };
    let cli_numbers = validate_numeric_range(key, definition.kind, &decoded)?;
    let token = match &decoded {
        WireValue::Scalar(value) => value.clone(),
        WireValue::Sequence(values) => values.join(","),
    };

    // Reject values the typed project-settings field cannot deserialize;
    // registry-only keys (no typed field) pass lexical validation upstream
    // (`validate_model_config_value` → `Config.cpp set_deserialize_raw`).
    // Its errors already name the key (`InvalidInput`), matching the
    // upstream `BadOptionValueException`/`UnknownOptionException` surface.
    crate::options::model_config_deserialize::validate_model_config_value(key, &token)?;
    let tokens = match decoded {
        WireValue::Scalar(value) => vec![value],
        WireValue::Sequence(values) => values,
    };
    Ok(OverrideValue {
        tokens,
        cli_numbers,
    })
}

fn validate_numeric_range(
    key: &str,
    kind: OptionValueKind,
    decoded: &WireValue,
) -> Result<Vec<f64>, SliceError> {
    let numeric = matches!(
        kind,
        OptionValueKind::Float
            | OptionValueKind::FloatOrPercent
            | OptionValueKind::Percent
            | OptionValueKind::Int
            | OptionValueKind::Floats
            | OptionValueKind::FloatsNullable
            | OptionValueKind::Percents
            | OptionValueKind::PercentsNullable
            | OptionValueKind::Ints
            | OptionValueKind::IntsNullable
    );
    if !numeric {
        return Ok(Vec::new());
    }
    let Some(metadata) = option_metadata(key) else {
        return Ok(Vec::new());
    };
    let retain_cli_numbers = matches!(
        key,
        "filament_diameter"
            | "filament_flow_ratio"
            | "layer_height"
            | "initial_layer_print_height"
            | "bridge_flow"
            | "internal_bridge_flow"
            | "extruder_clearance_radius"
            | "extruder_clearance_height_to_rod"
            | "extruder_clearance_height_to_lid"
            | "nozzle_height"
    );
    let mut cli_numbers = Vec::new();
    let raw_tokens: Vec<&str> = match decoded {
        WireValue::Scalar(value) => vec![value.as_str()],
        WireValue::Sequence(values) => values.iter().map(String::as_str).collect(),
    };
    for token in raw_tokens {
        // FloatOrPercent tokens may carry a trailing `%`; strip it for the
        // numeric bound check (upstream validates the stored double).
        let number = token.trim().trim_end_matches('%');
        if number == "nil" {
            if !metadata.nullable {
                return Err(invalid(key, "nil value for non-nullable option"));
            }
            continue;
        }
        let parsed: f64 = number
            .parse()
            .map_err(|_| invalid(key, "numeric option expects a number"))?;
        // No min/max check here: `ConfigBase::set_deserialize`
        // (`Config.cpp:591-620`) carries no range validation at all —
        // upstream's only numeric gate is `Slic3r::validate` over
        // `FullPrintConfig` (`PrintConfig.cpp:10515-10565`), which the
        // loaded-config path runs after `overrides.apply` (load.rs →
        // profile_entry.rs). Range rejections therefore surface once,
        // with the upstream `not in range [min,max]` map entry, and the
        // dynamic `filament_extruder_override_keys` exemption applies
        // there exactly as upstream `fpc.apply` dictates.
        if retain_cli_numbers {
            cli_numbers.push(parsed);
        }
    }
    Ok(cli_numbers)
}

const fn is_scalar(kind: OptionValueKind) -> bool {
    matches!(
        kind,
        OptionValueKind::Float
            | OptionValueKind::FloatOrPercent
            | OptionValueKind::Percent
            | OptionValueKind::Int
            | OptionValueKind::Bool
            | OptionValueKind::Enum
            | OptionValueKind::String
            | OptionValueKind::Point
    )
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid override for Orca option {key}: {reason}"))
}

impl<'de> serde::Deserialize<'de> for ConfigOverrides {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let entries: BTreeMap<String, OverrideEntry> =
            serde::Deserialize::deserialize(deserializer)?;
        Self::from_entries(entries).map_err(serde::de::Error::custom)
    }
}
