use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Value {
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
            Self::List(values) if values.len() == 1 => &values[0],
            _ => self,
        }
    }

    pub(super) fn index(&self, index: usize) -> Option<&Self> {
        match self {
            Self::List(values) => values.get(index),
            _ if index == 0 => Some(self),
            _ => None,
        }
    }

    pub(super) fn as_number(&self) -> Option<f64> {
        match self.scalar() {
            Self::Number(value) => Some(*value),
            Self::Bool(value) => Some(f64::from(*value)),
            Self::String(value) => value.parse().ok(),
            Self::List(_) => None,
        }
    }

    pub(super) fn as_bool(&self) -> bool {
        match self.scalar() {
            Self::Bool(value) => *value,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::List(values) => !values.is_empty(),
        }
    }

    pub(super) fn as_string(&self) -> String {
        match self.scalar() {
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

#[derive(Clone, Debug, Default)]
pub(super) struct Config {
    values: HashMap<String, Value>,
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
            config.values.insert(key.to_owned(), parse_value(raw));
        }
        config
    }

    pub(super) fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.values.insert(key.into(), value);
    }

    pub(super) fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

fn parse_value(raw: &str) -> Value {
    let mut values = raw.split(';').map(parse_scalar).collect::<Vec<_>>();
    if values.len() == 1 {
        let value = values.remove(0);
        let comma_values = value
            .as_string()
            .split(',')
            .map(parse_scalar)
            .collect::<Vec<_>>();
        return if comma_values.len() == 1 {
            value
        } else {
            Value::List(comma_values)
        };
    }
    Value::List(values)
}

fn parse_scalar(raw: &str) -> Value {
    let raw = raw.trim().trim_matches('"');
    if let Ok(value) = raw.parse::<f64>() {
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
    }

    #[test]
    fn scalar_numbers_use_six_significant_digits() {
        assert_eq!(Value::number(523.843179).as_string(), "523.843");
        assert_eq!(Value::number(4.3653598).as_string(), "4.36536");
        assert_eq!(Value::number(0.022).as_string(), "0.022");
    }
}
