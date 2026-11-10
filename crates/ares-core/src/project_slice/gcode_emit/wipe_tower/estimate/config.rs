//! The full print config the tower estimate and the plate list read by key, as upstream reads a
//! `DynamicPrintConfig`: `option_of` (`GCode/WipeTowerEstimate.cpp:31-39` at v2.5.0-beta
//! `345efec64`) takes the stored value, else the key's declared default from the
//! `PrintConfigDef` registry, and parses it by the registry's declared type.

use std::collections::BTreeMap;

use crate::options::collect_config_entries;
use crate::{OptionValueKind, ProjectSettings, SliceError, decode_config_value, option_definition};

/// One option's value (`ConfigOption`), by its declared kind.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ConfigValue {
    /// `coFloat`, `coPercent` and `coFloatOrPercent` (`getFloat`).
    Float(f64),
    /// `coFloats` and `coPercents`; `nil` reads as NaN.
    Floats(Vec<f64>),
    Int(i64),
    Ints(Vec<i64>),
    Bool(bool),
    Bools(Vec<bool>),
    /// `coEnum` by its serialized key.
    Enum(String),
    Text(String),
    Strings(Vec<String>),
}

impl ConfigValue {
    pub(crate) fn float(&self) -> f64 {
        match self {
            Self::Float(value) => *value,
            Self::Int(value) => *value as f64,
            other => panic!("not a scalar number: {other:?}"),
        }
    }

    pub(crate) fn int(&self) -> i64 {
        match self {
            Self::Int(value) => *value,
            other => panic!("not an int: {other:?}"),
        }
    }

    pub(crate) fn bool(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            other => panic!("not a bool: {other:?}"),
        }
    }

    pub(crate) fn key(&self) -> &str {
        match self {
            Self::Enum(value) => value,
            other => panic!("not an enum: {other:?}"),
        }
    }

    pub(crate) fn text(&self) -> &str {
        match self {
            Self::Text(value) => value,
            other => panic!("not a string: {other:?}"),
        }
    }

    /// `coPoints`, decoded as `[[x, y], ...]` mm.
    pub(crate) fn points(&self) -> Vec<[f64; 2]> {
        serde_json::from_str(self.text()).expect("decoded points are JSON pairs")
    }

    pub(crate) fn floats(&self) -> &[f64] {
        match self {
            Self::Floats(values) => values,
            other => panic!("not floats: {other:?}"),
        }
    }

    pub(crate) fn ints(&self) -> &[i64] {
        match self {
            Self::Ints(values) => values,
            other => panic!("not ints: {other:?}"),
        }
    }

    pub(crate) fn bools(&self) -> &[bool] {
        match self {
            Self::Bools(values) => values,
            other => panic!("not bools: {other:?}"),
        }
    }

    pub(crate) fn strings(&self) -> &[String] {
        match self {
            Self::Strings(values) => values,
            other => panic!("not strings: {other:?}"),
        }
    }
}

/// `ConfigOptionVector::get_at`: the element, else the first one.
pub(crate) fn get_at<T: Copy>(values: &[T], index: usize) -> T {
    values.get(index).copied().unwrap_or(values[0])
}

/// `DynamicPrintConfig`: the composed settings by key (`full_config()`), each value the token
/// `serialize` writes; a key without one reads the registry default.
pub struct DynamicPrintConfig {
    tokens: BTreeMap<String, String>,
}

impl DynamicPrintConfig {
    pub(crate) fn new(settings: &ProjectSettings) -> Result<Self, SliceError> {
        let tokens = collect_config_entries(settings)
            .map_err(|error| SliceError::InvalidInput(error.to_string()))?
            .into_iter()
            .map(|entry| (entry.key, entry.token))
            .collect();
        Ok(Self { tokens })
    }

    /// `DynamicPrintConfig::full_print_config()`: every key at its registry default.
    pub fn full_print_config() -> Self {
        Self {
            tokens: BTreeMap::new(),
        }
    }

    /// `ConfigBase::set_deserialize_strict` (`Config.hpp:2836-2840`): stores the serialized `value` of
    /// the registered `key`, rejecting an unknown key or a value its type cannot deserialize.
    pub fn set_deserialize_strict(&mut self, key: &str, value: &str) -> Result<(), SliceError> {
        let definition = option_definition(key)
            .ok_or_else(|| SliceError::InvalidInput(format!("unknown option {key}")))?;
        decode_config_value(definition.kind, value)
            .map_err(|error| SliceError::InvalidInput(format!("{key}: {error}")))?;
        self.tokens.insert(key.to_owned(), value.to_owned());
        Ok(())
    }

    /// `option_of(config, key)`.
    pub(crate) fn option(&self, key: &str) -> ConfigValue {
        let definition = option_definition(key).unwrap_or_else(|| panic!("no option {key}"));
        let token = self
            .tokens
            .get(key)
            .map_or(definition.default_wire_value, String::as_str);
        parse(definition.kind, token)
    }
}

/// `ConfigOption::deserialize` of `token` for an option of `kind` (`decode_config_value`).
fn parse(kind: OptionValueKind, token: &str) -> ConfigValue {
    use OptionValueKind as K;
    use serde_json::Value;
    let json = decode_config_value(kind, token).unwrap_or_else(|error| panic!("{error}"));
    let value: Value = serde_json::from_str(&json).expect("decoded config value is JSON");
    // A `nil` element of a nullable vector reads as NaN.
    let float = |value: &Value| value.as_f64().unwrap_or(f64::NAN);
    let elements = |value: &Value| value.as_array().cloned().unwrap_or_default();
    match kind {
        K::Float | K::Percent => ConfigValue::Float(float(&value)),
        K::FloatOrPercent => ConfigValue::Float(float(&value["value"])),
        K::FloatsOrPercents => ConfigValue::Floats(
            elements(&value)
                .iter()
                .map(|item| float(&item["value"]))
                .collect(),
        ),
        K::Floats | K::FloatsNullable | K::Percents | K::PercentsNullable => {
            ConfigValue::Floats(elements(&value).iter().map(float).collect())
        }
        K::Int => ConfigValue::Int(value.as_i64().expect("int")),
        K::Ints | K::IntsNullable => ConfigValue::Ints(
            elements(&value)
                .iter()
                .map(|item| item.as_i64().unwrap_or(0))
                .collect(),
        ),
        K::Bool => ConfigValue::Bool(value.as_bool().expect("bool")),
        K::Bools | K::BoolsNullable => ConfigValue::Bools(
            elements(&value)
                .iter()
                .map(|item| item.as_bool().unwrap_or(false))
                .collect(),
        ),
        K::Enum => ConfigValue::Enum(value.as_str().expect("enum").to_owned()),
        K::String => ConfigValue::Text(value.as_str().expect("string").to_owned()),
        K::Strings => ConfigValue::Strings(
            elements(&value)
                .iter()
                .map(|item| item.as_str().unwrap_or_default().to_owned())
                .collect(),
        ),
        K::Enums | K::EnumsNullable | K::Point | K::Points | K::PointsGroups => {
            ConfigValue::Text(json)
        }
    }
}
