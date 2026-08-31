//! Parse-time typed representation of raw `Metadata/project_settings.config`
//! values.
//!
//! The settings document keeps arbitrary preset keys that the typed
//! `ProjectSettings` does not model; the config block writer only needs each
//! key's export token (`GCode::append_full_config`), so values deserialize
//! into a typed tree and render tokens on demand without production code
//! holding dynamic JSON.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Raw project settings keyed for the config block; `None` tokens keep the
/// key known (`nil`, `null`, and object placeholders) without an export value.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(transparent)]
pub(crate) struct ProjectSettingsRaw(BTreeMap<String, RawConfigValue>);

impl ProjectSettingsRaw {
    pub(crate) fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, Option<String>)> {
        self.0
            .iter()
            .map(|(key, value)| (key.as_str(), value.token()))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
enum RawConfigValue {
    Text(String),
    Number(serde_json::Number),
    Flag(bool),
    List(Vec<RawConfigValue>),
    Object(BTreeMap<String, RawConfigValue>),
    Null,
}

impl RawConfigValue {
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
                (!rendered.is_empty()).then(|| rendered.join(";"))
            }
            Self::Object(_) | Self::Null => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSettingsRaw;

    fn tokens(json: &str) -> Vec<(String, Option<String>)> {
        let raw: ProjectSettingsRaw = serde_json::from_str(json).unwrap();
        raw.iter()
            .map(|(key, token)| (key.to_owned(), token))
            .collect()
    }

    #[test]
    fn scalars_render_their_export_tokens() {
        assert_eq!(
            tokens(
                r#"{"text":"0.42","flag":true,"off":false,"int":-5,"float":3.0,"precise":0.45}"#
            ),
            vec![
                ("flag".to_owned(), Some("1".to_owned())),
                ("float".to_owned(), Some("3.0".to_owned())),
                ("int".to_owned(), Some("-5".to_owned())),
                ("off".to_owned(), Some("0".to_owned())),
                ("precise".to_owned(), Some("0.45".to_owned())),
                ("text".to_owned(), Some("0.42".to_owned())),
            ]
        );
    }

    #[test]
    fn nil_null_and_objects_stay_known_without_tokens() {
        let raw: ProjectSettingsRaw =
            serde_json::from_str(r#"{"nil":"nil","none":null,"object":{"nested":1}}"#).unwrap();
        assert!(raw.contains_key("nil"));
        assert!(raw.contains_key("none"));
        assert!(raw.contains_key("object"));
        assert!(raw.iter().all(|(_, token)| token.is_none()));
    }

    #[test]
    fn lists_join_renderable_items_and_drop_everything_else() {
        assert_eq!(
            tokens(
                r#"{"nozzle":["0.4","0.4"],"mixed":[1,2.5],"skipped":[true,null,[1],{},"nil"],"empty":[]}"#
            ),
            vec![
                ("empty".to_owned(), None),
                ("mixed".to_owned(), Some("1;2.5".to_owned())),
                ("nozzle".to_owned(), Some("0.4;0.4".to_owned())),
                ("skipped".to_owned(), None),
            ]
        );
    }

    #[test]
    fn duplicate_keys_keep_the_last_value() {
        assert_eq!(
            tokens(r#"{"key":"first","key":"second"}"#),
            vec![("key".to_owned(), Some("second".to_owned()))]
        );
    }
}
