//! Parse-time typed representation of raw `Metadata/project_settings.config`
//! values.
//!
//! The settings document keeps arbitrary preset keys that the typed
//! `ProjectSettings` does not model; the config block writer only needs each
//! key's export token (`GCode::append_full_config`), so values deserialize
//! into a typed tree and render tokens on demand without production code
//! holding dynamic JSON.

mod deserialize;
mod numeric;

use std::collections::BTreeMap;

use serde::Deserialize;

/// Raw project settings keyed for the config block; `None` tokens keep the
/// key known (`nil`, `null`, and object placeholders) without an export value.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct ProjectSettingsRaw(BTreeMap<String, RawConfigValue>);

impl ProjectSettingsRaw {
    pub(crate) fn extend(&mut self, values: impl IntoIterator<Item = (String, RawConfigValue)>) {
        self.0.extend(values);
    }

    pub(crate) fn insert_strings(&mut self, key: &str, values: &[String]) {
        self.0.insert(
            key.to_owned(),
            RawConfigValue::List(values.iter().cloned().map(RawConfigValue::Text).collect()),
        );
    }

    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&RawConfigValue> {
        self.0.get(key)
    }

    pub(crate) fn insert(&mut self, key: String, value: RawConfigValue) {
        self.0.insert(key, value);
    }

    /// Applies the shared upstream deserializer normalizations for the
    /// 3MF embedded raw: empty strings on vector-kind options become
    /// empty vectors (`ConfigOptionStrings::deserialize`,
    /// `Config.hpp:1153-1159`) and scalar strings become single-element
    /// vectors (`ConfigOptionVector::deserialize`, `Config.hpp:868-887`,
    /// reached for every JSON string via `set_deserialize`,
    /// `Config.cpp:927-929`) — the same semantics preset documents get
    /// through `json_deserialize::deserialize_preset_config`.
    pub(crate) fn normalize_option_scalars(&mut self) -> Result<(), String> {
        crate::options::json_deserialize::normalize_unknown_wire_substitutions(&mut self.0);
        crate::options::json_deserialize::normalize_empty_vector_strings(&mut self.0);
        crate::options::json_deserialize::normalize_scalar_vector_strings(&mut self.0)
    }

    /// The 3MF load's wire substitutions alone (`Config.cpp:654-676`) for
    /// the EMBEDDED raw, before CLI overrides merge in: override values
    /// reach the config through their own deserialization rule and are not
    /// substituted.
    pub(crate) fn normalize_embedded_wire_substitutions(
        &mut self,
    ) -> Vec<crate::ConfigSubstitution> {
        crate::options::json_deserialize::normalize_unknown_wire_substitutions(&mut self.0)
    }

    /// `DynamicPrintConfig::load_from_json` (`Config.cpp:905-990`) hands
    /// only JSON strings (`:927-929`) and arrays to `set_deserialize`;
    /// every other scalar JSON type hits the `invalid json type`
    /// else-branch and the key is never set, so the option keeps its
    /// default. Applied at the project settings load boundary before any
    /// per-key deserialization, so dropped keys fall back to defaults
    /// exactly like upstream (bools then resolve through
    /// `ConfigOptionBool::deserialize`, `Config.hpp:1828-1848`, with the
    /// `enum_looks_like_true_value` retry, `Config.hpp:96-99`).
    pub(crate) fn drop_invalid_wire_scalars(&mut self) {
        self.0.retain(|_, value| {
            !matches!(
                value,
                RawConfigValue::Number(_) | RawConfigValue::Flag(_) | RawConfigValue::Null
            )
        });
    }

    /// The vector-shape normalizations without the wire substitutions,
    /// for a raw that already carries override values on top of the
    /// normalized embedded config.
    pub(crate) fn normalize_wire_vector_shapes(&mut self) -> Result<(), String> {
        crate::options::json_deserialize::normalize_empty_vector_strings(&mut self.0);
        crate::options::json_deserialize::normalize_scalar_vector_strings(&mut self.0)
    }

    /// `Config.cpp:945-1001` array branch for the embedded raw: a
    /// homogeneous string array on a scalar-kind option joins
    /// (`parse_str_arr`) and decodes through the option's own deserializer
    /// — the second `handle_legacy` pass over the joined value inside
    /// `set_deserialize` (`Config.cpp:577-579`) — so the flat map
    /// `Slic3r::validate` checks and the export view carry the bound scalar,
    /// not the array. Runs after `handle_legacy` (the branch's first pass
    /// renames the key with an empty value).
    pub(crate) fn normalize_scalar_arrays(&mut self) -> Result<(), String> {
        crate::options::json_deserialize::normalize_numeric_scalar_arrays(&mut self.0)
    }

    /// `PrintConfigDef::handle_legacy` (`PrintConfig.cpp:8033`) runs inside
    /// `set_deserialize` (`Config.cpp:577-579`) on each `key/value` token
    /// before the option is stored, so the loaded config — and the flat map
    /// `Slic3r::validate` checks — is post-legacy. Applies the shared
    /// string-token `handle_legacy` core to every entry: the value's
    /// `token()` is the `value` string upstream sees, the normalized token is
    /// re-decoded by the (possibly renamed) key's option kind, and a cleared
    /// key drops the entry.
    /// `PrintConfigDef::handle_legacy` (`PrintConfig.cpp:8033`) runs inside
    /// `set_deserialize` (`Config.cpp:577-579`) on each `key/value` token
    /// before the option is stored, so the loaded config — and the flat map
    /// `Slic3r::validate` checks — is post-legacy.
    ///
    /// Upstream `load_from_json` (`Config.cpp:945-962`) hands `handle_legacy`
    /// the raw string for scalar values but an *empty* `value_str` for array
    /// elements (the array is element-processed later), so non-scalar
    /// `RawConfigValue`s only ever get a key rename — their elements are
    /// preserved untouched. Scalars carry their `token()` through the rewrite
    /// and are re-decoded by the (possibly renamed) key's option kind. A
    /// cleared key drops the entry; keys with no option definition are kept
    /// (matching `SliceOptions`' normalization, which does not drop them).
    pub(crate) fn handle_legacy(&mut self) {
        let mut normalized = BTreeMap::new();
        for (mut key, value) in std::mem::take(&mut self.0) {
            let scalar = matches!(
                value,
                RawConfigValue::Text(_) | RawConfigValue::Number(_) | RawConfigValue::Flag(_)
            );
            let mut token = if let RawConfigValue::Text(text) = &value {
                // handle_legacy receives the deserialize input, not the export
                // view: Config.hpp:946-950 distinguishes "nil" from "" (zero).
                text.clone()
            } else if scalar {
                value.token().unwrap_or_default()
            } else {
                String::new()
            };
            // `load_from_json`-level derived keys (`Config.cpp:937-945`): a
            // string `key`/token can produce an extra entry (e.g.
            // `is_infill_first` for `infill/…` wall orders) before
            // `handle_legacy` rewrites the pair.
            if scalar
                && let Some((derived_key, derived_token)) =
                    crate::options::legacy::handle_legacy_derived(&key, &token)
            {
                normalized
                    .entry(derived_key.to_owned())
                    .or_insert_with(|| RawConfigValue::Text(derived_token.to_owned()));
            }
            crate::options::handle_legacy(&mut key, &mut token);
            if key.is_empty() {
                continue;
            }
            // Upstream `handle_legacy` only rewrites the `value` string; the
            // option's own `deserialize` (run later by the typed builder) is
            // what parses/validates it, so the normalized token is stored back
            // as a plain text scalar — not re-decoded here (which would
            // pre-empt the typed builder's diagnostics).
            let stored = if scalar {
                RawConfigValue::Text(token)
            } else {
                value
            };
            normalized.insert(key, stored);
        }
        self.0 = normalized;
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (&String, &RawConfigValue)> {
        self.0.iter()
    }

    /// The post-`handle_legacy` flat map, for the `is_value_valid` range
    /// loop (`PrintConfig.cpp:10519-10559`) which reads `cfg.keys()`.
    pub(crate) fn as_map(&self) -> &BTreeMap<String, RawConfigValue> {
        &self.0
    }

    /// Consumes the raw view into its typed tree (`RawConfigValue::Object`),
    /// a serde `Deserializer` for the typed builders.
    pub(crate) fn into_object(self) -> RawConfigValue {
        RawConfigValue::Object(self.0)
    }

    /// Extends with another raw view; later keys win (`extend` semantics).
    pub(crate) fn extend_raw(&mut self, other: &Self) {
        self.0.extend(
            other
                .0
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }

    /// Export-echo view: every entry renders through `option_token`,
    /// which serializes Floats-kind vectors the way upstream
    /// `ConfigOptionFloats::serialize` does (per-element `%g`, comma
    /// join) — a non-float element is invalid input, not a verbatim
    /// echo.
    pub(crate) fn iter(&self) -> impl Iterator<Item = Result<(&str, Option<String>), String>> {
        self.0.iter().map(|(key, value)| {
            value
                .option_token(key, None)
                .map(|token| (key.as_str(), token))
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub(crate) enum RawConfigValue {
    Text(String),
    Number(serde_json::Number),
    Flag(bool),
    List(Vec<RawConfigValue>),
    Object(BTreeMap<String, RawConfigValue>),
    Null,
}

impl RawConfigValue {
    /// Raw coStrings retain logical slots for typed deserialization; only
    /// export renders `escape_strings_cstyle` (Config.cpp:72-120).
    fn option_token(&self, key: &str, count: Option<usize>) -> Result<Option<String>, String> {
        use crate::options::registry::{OptionValueKind, option_definition};
        let definition = option_definition(key);
        if definition.is_some_and(|definition| definition.kind == OptionValueKind::String)
            && let Self::Text(text) = self
        {
            return Ok(Some(crate::options::escape_scalar_string(text)));
        }
        if definition.is_some_and(|definition| definition.kind == OptionValueKind::Strings)
            && let Self::List(values) = self
        {
            let count = count.unwrap_or(values.len()).min(values.len());
            let rendered = values[..count]
                .iter()
                .map(|value| match value {
                    Self::Text(text) => crate::options::escape_string_vector_element(text, count),
                    _ => unreachable!("normalized coStrings contain logical string slots"),
                })
                .collect::<Vec<_>>();
            return Ok(Some(rendered.join(";")));
        }
        if let Some(definition) = definition
            && numeric::is_vector(definition.kind)
            && let Self::List(values) = self
        {
            let count = count.unwrap_or(values.len()).min(values.len());
            return numeric::token(&values[..count], definition.kind, key);
        }
        Ok(count.map_or_else(|| self.token(), |count| self.token_prefix(count)))
    }

    /// `ConfigOptionVector::get_at(index)` equivalent: lists return their
    /// element, scalars return themselves at index 0 only.
    pub(crate) fn element(&self, index: usize) -> Option<&RawConfigValue> {
        match self {
            Self::List(values) => values.get(index),
            value if index == 0 => Some(value),
            _ => None,
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::List(values) => values.len(),
            Self::Null => 0,
            _ => 1,
        }
    }

    /// Renders the export token; `nil` strings, `null`, and object
    /// placeholders stay without a value and lists join their renderable
    /// items with `;`.
    pub(crate) fn token(&self) -> Option<String> {
        match self {
            Self::Text(text) => (text != "nil").then(|| text.clone()),
            Self::Number(number) => Some(number.to_string()),
            Self::Flag(flag) => Some(if *flag {
                "1".to_owned()
            } else {
                "0".to_owned()
            }),
            Self::List(values) => {
                let rendered: Vec<String> = values
                    .iter()
                    .filter_map(|item| match item {
                        Self::Text(text) if text != "nil" => Some(text.clone()),
                        Self::Number(number) => Some(number.to_string()),
                        _ => None,
                    })
                    .collect();
                (!rendered.is_empty()).then(|| render_list_token(&rendered))
            }
            Self::Object(_) | Self::Null => None,
        }
    }

    /// `token()` over only the first `count` list items.
    fn token_prefix(&self, count: usize) -> Option<String> {
        match self {
            Self::List(values) => {
                let rendered: Vec<String> = values
                    .iter()
                    .take(count)
                    .filter_map(|item| match item {
                        Self::Text(text) if text != "nil" => Some(text.clone()),
                        Self::Number(number) => Some(number.to_string()),
                        Self::Flag(flag) => Some(if *flag {
                            "1".to_owned()
                        } else {
                            "0".to_owned()
                        }),
                        _ => None,
                    })
                    .collect();
                (!rendered.is_empty()).then(|| render_list_token(&rendered))
            }
            other => other.token(),
        }
    }
}

fn render_list_token(rendered: &[String]) -> String {
    // A single empty string element serializes quoted
    // (`ConfigOptionString::serialize` wraps `escape_string_cstyle`);
    // multi-element joins stay raw.
    if rendered.len() == 1 && rendered[0].is_empty() {
        return "\"\"".to_owned();
    }
    rendered.join(";")
}
