use std::{cell::Cell, collections::HashMap};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Value {
    Integer(i64),
    Number(f64),
    String(String),
    List(Vec<Value>),
    Bool(bool),
}

impl Value {
    pub(super) fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub(super) fn scalar(&self) -> &Self {
        match self {
            Self::List(values) if !values.is_empty() => &values[0],
            _ => self,
        }
    }

    /// Upstream `ConfigOptionVector::get_at` never fails for a non-empty
    /// option: an out-of-range index falls back to the first value, and a
    /// scalar behaves as a one-element vector (`Config.hpp:624-628`).
    pub(super) fn index(&self, index: usize) -> Option<&Self> {
        match self {
            Self::List(values) => values.get(index).or_else(|| values.first()),
            scalar => Some(scalar),
        }
    }

    pub(super) fn iter_list(&self) -> std::slice::Iter<'_, Self> {
        match self {
            Self::List(values) => values.iter(),
            scalar @ (Self::Integer(_) | Self::Number(_) | Self::String(_) | Self::Bool(_)) => {
                std::slice::from_ref(scalar).iter()
            }
        }
    }

    pub(super) fn as_number(&self) -> Option<f64> {
        match self.scalar() {
            Self::Integer(value) => Some(*value as f64),
            Self::Number(value) => Some(*value),
            Self::Bool(value) => Some(f64::from(*value)),
            Self::String(value) => value.parse().ok(),
            Self::List(_) => None,
        }
    }

    pub(super) fn as_integer(&self) -> Option<i64> {
        match self.scalar() {
            Self::Integer(value) => Some(*value),
            Self::Bool(value) => Some(i64::from(*value)),
            _ => None,
        }
    }

    pub(super) fn as_bool(&self) -> bool {
        match self.scalar() {
            Self::Bool(value) => *value,
            Self::Integer(value) => *value != 0,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::List(values) => !values.is_empty(),
        }
    }

    pub(super) fn as_string(&self) -> String {
        match self.scalar() {
            Self::Integer(value) => value.to_string(),
            Self::Number(value) if value.fract() == 0.0 => format!("{value:.0}"),
            Self::Number(value) => format_number(*value),
            Self::String(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::List(values) => values
                .iter()
                .map(Self::as_string)
                .collect::<Vec<_>>()
                .join(","),
        }
    }
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let exponent = value.abs().log10().floor() as i32;
    let decimals = (5 - exponent).max(0) as usize;
    let formatted = format!("{value:.decimals$}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

#[derive(Clone, Debug)]
pub(super) struct Config {
    values: HashMap<String, Value>,
    random_state: Cell<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
            random_state: Cell::new(5_489),
        }
    }
}

impl Config {
    pub(super) fn from_block(block: &[u8]) -> Self {
        let mut config = Self::default();
        for line in String::from_utf8_lossy(block).lines() {
            let Some(line) = line.strip_prefix("; ") else {
                continue;
            };
            let Some((key, raw)) = line.split_once(" = ") else {
                continue;
            };
            let is_bool =
                crate::options::registry::option_definition(key).is_some_and(|definition| {
                    matches!(
                        definition.kind,
                        crate::options::registry::OptionValueKind::Bool
                            | crate::options::registry::OptionValueKind::Bools
                    )
                });
            config.insert(key, parse_value(raw, is_bool));
        }
        config
    }

    pub(super) fn insert(&mut self, key: impl Into<String>, value: Value) {
        let key = key.into();
        if let Value::List(values) = &value {
            for (index, item) in values.iter().enumerate() {
                self.values.insert(format!("{key}_{index}"), item.clone());
            }
        }
        self.values.insert(key, value);
    }

    /// Stores an assignment target (`{var[idx] = value}`): lists grow with
    /// zero padding like the upstream per-extruder vectors.
    pub(super) fn assign(&mut self, name: &str, index: Option<usize>, value: Value) {
        match index {
            None => self.insert(name, value),
            Some(index) => {
                let mut values = match self.get(name) {
                    Some(Value::List(existing)) => existing.clone(),
                    Some(other) => vec![other.clone()],
                    None => Vec::new(),
                };
                if values.len() <= index {
                    values.resize(index + 1, Value::Integer(0));
                }
                values[index] = value;
                self.insert(name, Value::List(values));
            }
        }
    }

    pub(super) fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub(super) fn random_unit(&self) -> f64 {
        let current = self.random_state.get();
        let next = current
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.random_state.set(next);
        (next >> 11) as f64 / (1_u64 << 53) as f64
    }
}

fn parse_value(raw: &str, is_bool: bool) -> Value {
    let mut values = raw
        .split(';')
        .map(|scalar| parse_scalar(scalar, is_bool))
        .collect::<Vec<_>>();
    if values.len() == 1 {
        let value = values.remove(0);
        let comma_values = value
            .as_string()
            .split(',')
            .map(|scalar| parse_scalar(scalar, is_bool))
            .collect::<Vec<_>>();
        return if comma_values.len() == 1 {
            value
        } else {
            Value::List(comma_values)
        };
    }
    Value::List(values)
}

fn parse_scalar(raw: &str, is_bool: bool) -> Value {
    let raw = raw.trim().trim_matches('"');
    if is_bool && (raw == "0" || raw == "1") {
        Value::Bool(raw == "1")
    } else if let Ok(value) = raw.parse::<f64>() {
        Value::Number(value)
    } else if raw == "true" || raw == "false" {
        Value::Bool(raw == "true")
    } else {
        Value::String(raw.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_parses_scalars_and_vectors() {
        let config = Config::from_block(b"; one = 2\n; values = 1,2\n; names = PLA;PETG\n");
        assert_eq!(config.get("one").unwrap().as_number(), Some(2.0));
        assert_eq!(
            config.get("values").unwrap().index(1).unwrap().as_number(),
            Some(2.0)
        );
        assert_eq!(
            config.get("names").unwrap().index(1).unwrap().as_string(),
            "PETG"
        );
        assert_eq!(config.get("values_1").unwrap().as_number(), Some(2.0));
        assert_eq!(
            config.get("values").unwrap().index(7).unwrap().as_number(),
            Some(1.0)
        );
        assert_eq!(config.get("names_0").unwrap().as_string(), "PLA");
    }

    #[test]
    fn scalar_numbers_use_six_significant_digits() {
        assert_eq!(Value::number(523.843179).as_string(), "523.843");
        assert_eq!(Value::number(4.3653598).as_string(), "4.36536");
        assert_eq!(Value::number(0.022).as_string(), "0.022");
    }
}
