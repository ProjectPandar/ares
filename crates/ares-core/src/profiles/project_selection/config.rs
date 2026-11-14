use std::borrow::Cow;

use serde::Serialize;

use super::super::project_refresh::{
    keys::full_config_value,
    preset::{connection, strings},
};
use super::typed::Typed;
use crate::{
    ProfileKind, SliceError,
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct PresetConfig(pub(super) ProjectSettingsRaw);

impl PresetConfig {
    pub(super) fn defaults(kind: ProfileKind) -> Result<Self, SliceError> {
        let mut raw = ProjectSettingsRaw::default();
        raw.extend(super::super::project_refresh::default_preset_fields(kind)?);
        let mut config = Self(raw);
        config.set_string(super::collection::identity(kind), String::new());
        Ok(config)
    }

    // Preset::load_external_config (Preset.cpp:1091-1105): every key but the connection ones.
    pub(super) fn apply_project(&mut self, project: &ProjectSettingsRaw) -> Result<(), SliceError> {
        let keys = self
            .0
            .entries()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        for key in keys.into_iter().filter(|key| !connection(key)) {
            if let Some(value) = project_value(project, &key)? {
                self.0.insert(key, value.into_owned());
            }
        }
        Ok(())
    }

    pub(super) fn string(&self, key: &str) -> String {
        strings(&self.0, key).into_iter().next().unwrap_or_default()
    }

    pub(super) fn set_string(&mut self, key: &str, value: String) {
        let value = RawConfigValue::Text(value);
        let value = if crate::option_definition(key)
            .is_some_and(|definition| definition.kind == crate::OptionValueKind::Strings)
        {
            RawConfigValue::List(vec![value])
        } else {
            value
        };
        self.0.insert(key.to_owned(), value);
    }

    // Config.cpp:531-555: compare only common options, using their typed values.
    pub(super) fn diff(&self, other: &Self) -> Result<Vec<String>, SliceError> {
        let mut result = Vec::new();
        for (key, left) in self.0.entries() {
            let Some(right) = other.0.get(key) else {
                continue;
            };
            if !typed(key, left)?.same(&typed(key, right)?) {
                result.push(key.clone());
            }
        }
        Ok(result)
    }

    pub(super) fn same(&self, other: &Self) -> Result<bool, SliceError> {
        for (key, left) in self.0.entries() {
            if ignored(key) {
                continue;
            }
            if let Some(right) = other.0.get(key)
                && !typed(key, left)?.same(&typed(key, right)?)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn dirty(&self, edited: &Self) -> Result<(bool, Vec<String>), SliceError> {
        let mut differences = self.diff(edited)?;
        for key in ["compatible_prints", "compatible_printers"] {
            if self.0.contains_key(key) != edited.0.contains_key(key)
                && !differences.iter().any(|entry| entry == key)
            {
                differences.push(key.to_owned());
            }
        }
        let dirty = differences.iter().any(|key| {
            !matches!(
                key.as_str(),
                "print_settings_id" | "filament_settings_id" | "printer_settings_id"
            )
        });
        Ok((dirty, differences))
    }
}

pub(super) fn typed(key: &str, raw: &RawConfigValue) -> Result<Typed, SliceError> {
    use crate::OptionValueKind as K;
    let definition =
        crate::option_definition(key).expect("preset collection keys have option definitions");
    // ConfigOption::serialize retains empty vectors and every nil slot; the
    // G-code export echo deliberately omits all-nil values and cannot be used here.
    let wire = match raw {
        RawConfigValue::Text(text) if definition.kind == K::String => {
            crate::options::escape_scalar_string(text)
        }
        RawConfigValue::List(values) if definition.kind == K::Strings => values
            .iter()
            .map(|value| {
                scalar(value)
                    .map(|value| crate::options::escape_string_vector_element(&value, values.len()))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(";"),
        RawConfigValue::List(values) if definition.kind == K::PointsGroups => values
            .iter()
            .map(|group| match group {
                RawConfigValue::List(points) => points
                    .iter()
                    .map(scalar)
                    .collect::<Result<Vec<_>, _>>()
                    .map(|points| points.join(",")),
                value => scalar(value),
            })
            .collect::<Result<Vec<_>, _>>()?
            // ConfigOptionPointsGroups::serialize (Config.hpp:1725-1735) joins groups with
            // '#', which deserialize splits on (:1748-1773).
            .join("#"),
        RawConfigValue::List(values) => values
            .iter()
            .map(scalar)
            .collect::<Result<Vec<_>, _>>()?
            .join(","),
        value => scalar(value)?,
    };
    Typed::parse(definition.kind, &wire)
        .map_err(|error| SliceError::InvalidInput(format!("{key}: {error}")))
}

fn scalar(value: &RawConfigValue) -> Result<String, SliceError> {
    match value {
        RawConfigValue::Text(value) => Ok(value.clone()),
        RawConfigValue::Number(value) => Ok(value.to_string()),
        RawConfigValue::Flag(value) => Ok(u8::from(*value).to_string()),
        RawConfigValue::Null => Ok("nil".to_owned()),
        RawConfigValue::List(_) | RawConfigValue::Object(_) => Err(SliceError::InvalidInput(
            "invalid collection option shape".to_owned(),
        )),
    }
}

// Preset.cpp:3009-3024: complete profile_print_params_same exclusion set.
fn ignored(key: &str) -> bool {
    matches!(
        key,
        "compatible_prints"
            | "compatible_prints_condition"
            | "compatible_printers"
            | "compatible_printers_condition"
            | "inherits"
            | "print_settings_id"
            | "filament_settings_id"
            | "printer_settings_id"
            | "sla_print_settings_id"
            | "sla_material_settings_id"
            | "printer_model"
            | "printer_variant"
            | "default_print_profile"
            | "default_filament_profile"
            | "default_sla_print_profile"
            | "default_sla_material_profile"
    )
}

/// `key` of the project config the presets load from: the plater loads the 3MF over
/// `FullPrintConfig::defaults()` (Plater.cpp:9463), so an option the 3MF lacks holds its default.
pub(super) fn project_value<'a>(
    project: &'a ProjectSettingsRaw,
    key: &str,
) -> Result<Option<Cow<'a, RawConfigValue>>, SliceError> {
    Ok(match project.get(key) {
        Some(value) => Some(Cow::Borrowed(value)),
        None => full_config_value(key)?.map(Cow::Owned),
    })
}
