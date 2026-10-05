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
    values: BTreeMap<String, Vec<String>>,
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

    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Applies the overrides onto the raw project-settings view
    /// (`ConfigBase::apply_only`, `Config.cpp:461`: later values win).
    pub(crate) fn apply(&self, raw: &mut ProjectSettingsRaw) {
        for (key, tokens) in &self.values {
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

/// Validates one `{key: tokens}` entry and returns the per-element tokens
/// to store (`Config.cpp set_deserialize` error ordering: unknown key →
/// malformed token → out-of-range value → typed deserialize failure).
fn validate_entry(key: &str, items: Vec<String>) -> Result<Vec<String>, SliceError> {
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
    validate_numeric_range(key, definition.kind, &decoded)?;
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
    match decoded {
        WireValue::Scalar(value) => Ok(vec![value]),
        WireValue::Sequence(values) => Ok(values),
    }
}

/// Port of `ConfigOptionDef::is_value_valid` (`Config.cpp:321-337`): NaN is
/// only valid for nullable options; a `min == 0` bound rejects negative
/// input; values within `10^-4` of either bound count as equal; otherwise
/// the value must satisfy `min <= value <= max`. Upstream unset bounds are
/// `-FLT_MAX`/`FLT_MAX`.
fn is_value_valid(metadata: &super::registry::OptionMetadata, value: f64) -> bool {
    if value.is_nan() {
        return metadata.nullable;
    }
    let min = metadata.min.unwrap_or(-f32::MAX as f64);
    let max = metadata.max.unwrap_or(f32::MAX as f64);
    if min == 0.0 && value < 0.0 {
        return false;
    }
    let epsilon = 0.1f64.powi(4);
    if (value - min).abs() <= epsilon || (value - max).abs() <= epsilon {
        return true;
    }
    min <= value && value <= max
}

fn validate_numeric_range(
    key: &str,
    kind: OptionValueKind,
    decoded: &WireValue,
) -> Result<(), SliceError> {
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
        return Ok(());
    }
    let Some(metadata) = option_metadata(key) else {
        return Ok(());
    };
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
        if !is_value_valid(metadata, parsed) {
            // Upstream reports `serialize() + " not in range [min,max]"`
            // (`PrintConfig.cpp:10561`).
            let min = metadata.min.unwrap_or(-f32::MAX as f64);
            let max = metadata.max.unwrap_or(f32::MAX as f64);
            return Err(invalid(key, &format!("{token} not in range [{min},{max}]")));
        }
    }
    Ok(())
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
