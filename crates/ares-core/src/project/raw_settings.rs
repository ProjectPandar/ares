//! Parse-time typed representation of raw `Metadata/project_settings.config`
//! values.
//!
//! The settings document keeps arbitrary preset keys that the typed
//! `ProjectSettings` does not model; the config block writer only needs each
//! key's export token (`GCode::append_full_config`), so values deserialize
//! into a typed tree and render tokens on demand without production code
//! holding dynamic JSON.

mod deserialize;

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
        let strings = crate::OrcaStrings(values.to_vec());
        let value = crate::options::serialize_config_value(&strings)
            .expect("profile string vectors serialize");
        self.0
            .insert(key.to_owned(), RawConfigValue::Text(value.token));
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
        crate::options::json_deserialize::normalize_empty_vector_strings(&mut self.0);
        crate::options::json_deserialize::normalize_scalar_vector_strings(&mut self.0)
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (&String, &RawConfigValue)> {
        self.0.iter()
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, Option<String>)> {
        self.0
            .iter()
            .map(|(key, value)| (key.as_str(), value.token()))
    }

    /// Renders the export token of only the first `count` list items
    /// (scalar values render like `token()`). Orca trims preset vectors to
    /// the active extruder count, so trailing preset slots must not count
    /// toward nil-ness checks.
    pub(crate) fn token_prefix(&self, key: &str, count: usize) -> Option<String> {
        self.0.get(key).and_then(|value| value.token_prefix(count))
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
    fn token(&self) -> Option<String> {
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
#[cfg(test)]
mod tests;
