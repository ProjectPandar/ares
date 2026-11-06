use std::{cell::RefCell, collections::HashMap, rc::Rc};

mod random;

// PlaceholderParser.hpp:16-26: config copies share the execution context.
pub(crate) type TemplateRng = Rc<RefCell<crate::slicer_context::Mt19937>>;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Value {
    Nil,
    Integer(i64),
    Number(f64),
    String(String),
    List(Vec<Value>),
    OptionBool(bool),
    Bool(bool),
}

impl Value {
    pub(crate) fn ensure_defined(&self) -> Result<(), String> {
        if matches!(self, Self::Nil) {
            Err(
                "Trying to reference an undefined (nil) element of vector of optional values"
                    .to_owned(),
            )
        } else {
            Ok(())
        }
    }

    pub(crate) fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub(crate) fn option_bool(value: bool) -> Self {
        Self::OptionBool(value)
    }

    pub(crate) fn scalar(&self) -> &Self {
        match self {
            Self::List(values) if !values.is_empty() => &values[0],
            _ => self,
        }
    }

    /// Upstream `ConfigOptionVector::get_at` never fails for a non-empty
    /// option: an out-of-range index falls back to the first value, and a
    /// scalar behaves as a one-element vector (`Config.hpp:624-628`).
    pub(crate) fn index(&self, index: usize) -> Option<&Self> {
        match self {
            Self::List(values) => values.get(index).or_else(|| values.first()),
            scalar => Some(scalar),
        }
    }

    pub(crate) fn iter_list(&self) -> std::slice::Iter<'_, Self> {
        match self {
            Self::List(values) => values.iter(),
            scalar @ (Self::Nil
            | Self::Integer(_)
            | Self::Number(_)
            | Self::String(_)
            | Self::OptionBool(_)
            | Self::Bool(_)) => std::slice::from_ref(scalar).iter(),
        }
    }

    pub(crate) fn as_number(&self) -> Option<f64> {
        match self.scalar() {
            Self::Integer(value) => Some(*value as f64),
            Self::Number(value) => Some(*value),
            Self::OptionBool(value) | Self::Bool(value) => Some(f64::from(*value)),
            Self::String(value) => value.parse().ok(),
            Self::Nil | Self::List(_) => None,
        }
    }

    pub(crate) fn as_integer(&self) -> Option<i64> {
        match self.scalar() {
            Self::Integer(value) => Some(*value),
            Self::OptionBool(value) | Self::Bool(value) => Some(i64::from(*value)),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> bool {
        match self.scalar() {
            Self::Nil => false,
            Self::OptionBool(value) | Self::Bool(value) => *value,
            Self::Integer(value) => *value != 0,
            Self::Number(value) => *value != 0.0,
            Self::String(value) => !value.is_empty(),
            Self::List(values) => !values.is_empty(),
        }
    }

    pub(crate) fn as_string(&self) -> String {
        match self.scalar() {
            Self::Nil => "nil".to_owned(),
            Self::Integer(value) => value.to_string(),
            Self::Number(value) => format_number(*value),
            Self::String(value) => value.clone(),
            Self::OptionBool(value) | Self::Bool(value) => value.to_string(),
            Self::List(values) => values
                .iter()
                .map(Self::as_string)
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    pub(crate) fn as_legacy_string(&self) -> String {
        match self.scalar() {
            Self::OptionBool(value) => i64::from(*value).to_string(),
            value => value.as_string(),
        }
    }
}

fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return match (value.is_nan(), value.is_sign_negative()) {
            (true, false) => "nan",
            (true, true) => "-nan",
            (false, false) => "inf",
            (false, true) => "-inf",
        }
        .to_owned();
    }
    // Select defaultfloat's style from the six-digit rounded binary value.
    let scientific = format!("{value:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    if !(-4..6).contains(&exponent) {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        format!("{mantissa}e{exponent:+03}")
    } else {
        let decimals = (5 - exponent) as usize;
        let fixed = format!("{value:.decimals$}");
        if decimals == 0 {
            fixed
        } else {
            fixed.trim_end_matches('0').trim_end_matches('.').to_owned()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Config {
    values: HashMap<String, Value>,
    pub(crate) random: Option<TemplateRng>,
}

impl Config {
    pub(crate) fn from_block(block: &[u8]) -> Self {
        let mut config = Self::default();
        for line in String::from_utf8_lossy(block).lines() {
            let Some(line) = line.strip_prefix("; ") else {
                continue;
            };
            let Some((key, raw)) = line.split_once(" = ") else {
                continue;
            };
            let kind =
                crate::options::registry::option_definition(key).map(|definition| definition.kind);
            config.insert(key, parse_value(raw, kind));
        }
        config
    }

    pub(crate) fn insert(&mut self, key: impl Into<String>, value: Value) {
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
    pub(crate) fn assign(&mut self, name: &str, index: Option<usize>, value: Value) {
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

    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Drops an output variable created during a template render
    /// (upstream `config_local` lifetime, `PlaceholderParser.cpp:889-906`),
    /// including the spread `key_index` entries of an assigned list.
    pub(crate) fn remove(&mut self, key: &str) {
        if let Some(Value::List(values)) = self.values.get(key).cloned() {
            for index in 0..values.len() {
                self.values.remove(&format!("{key}_{index}"));
            }
        }
        self.values.remove(key);
    }
}

fn parse_value(raw: &str, kind: Option<crate::options::registry::OptionValueKind>) -> Value {
    use crate::options::registry::OptionValueKind as Kind;
    // Registry-typed vector options keep their vector shape regardless of
    // element count — upstream vectors stay `is_vector()` even with a
    // single element, and a bare brace reference then enters
    // `vector_element_to_expr` (`PlaceholderParser.cpp:1086-1211`) instead
    // of scalar resolution. Element kinds follow the option type:
    // `Ints → Integer`, float/percent vectors → `Number`, bool vectors →
    // OptionBool, string vectors → String.
    if let Some(kind) = kind {
        let elements = match kind {
            Kind::Ints | Kind::IntsNullable => raw
                .split([';', ','])
                .map(|scalar| parse_vector_scalar(scalar, false, true, kind == Kind::IntsNullable))
                .collect::<Vec<_>>(),
            Kind::Floats | Kind::FloatsNullable | Kind::Percents | Kind::PercentsNullable => raw
                .split([';', ','])
                .map(|scalar| {
                    parse_vector_scalar(
                        scalar,
                        false,
                        false,
                        matches!(kind, Kind::FloatsNullable | Kind::PercentsNullable),
                    )
                })
                .collect::<Vec<_>>(),
            Kind::Bools | Kind::BoolsNullable => raw
                .split([';', ','])
                .map(|scalar| parse_vector_scalar(scalar, true, false, kind == Kind::BoolsNullable))
                .collect::<Vec<_>>(),
            Kind::Strings | Kind::Enums | Kind::EnumsNullable => raw
                .split([';', ','])
                .map(|scalar| parse_scalar(scalar, false, false))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        if !elements.is_empty() {
            return Value::List(elements);
        }
    }
    let is_bool = kind.is_some_and(|kind| matches!(kind, Kind::Bool | Kind::Bools));
    let is_ints = kind == Some(Kind::Ints);
    let mut values = raw
        .split(';')
        .map(|scalar| parse_scalar(scalar, is_bool, is_ints))
        .collect::<Vec<_>>();
    if values.len() == 1 {
        let value = values.remove(0);
        let comma_values = value
            .as_string()
            .split(',')
            .map(|scalar| parse_scalar(scalar, is_bool, is_ints))
            .collect::<Vec<_>>();
        return if comma_values.len() == 1 {
            value
        } else {
            Value::List(comma_values)
        };
    }
    Value::List(values)
}

fn parse_vector_scalar(raw: &str, is_bool: bool, is_ints: bool, nullable: bool) -> Value {
    if nullable && raw.trim() == "nil" {
        Value::Nil
    } else {
        parse_scalar(raw, is_bool, is_ints)
    }
}

fn parse_scalar(raw: &str, is_bool: bool, is_ints: bool) -> Value {
    let raw = raw.trim().trim_matches('"');
    if is_bool && matches!(raw, "0" | "1" | "false" | "true") {
        Value::option_bool(matches!(raw, "1" | "true"))
    } else if is_ints {
        Value::Integer(raw.parse().unwrap())
    } else if let Ok(value) = raw.parse::<f64>() {
        Value::Number(value)
    } else if raw == "true" || raw == "false" {
        Value::Bool(raw == "true")
    } else {
        Value::String(raw.to_owned())
    }
}
