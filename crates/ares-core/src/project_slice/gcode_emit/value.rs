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

/// The `expr` type an option value takes: `scalar_variable_to_expr`
/// (`PlaceholderParser.cpp:1170-1232`) and `vector_element_to_expr` (`:1355-1361`).
#[derive(Clone, Copy)]
enum Scalar {
    Bool,
    Int,
    /// `coFloat`, `coPercent` and their vectors: the number without its `%`.
    Float,
    /// `coString`, `coEnum` and `coStrings`: the serialized text.
    Text,
    Untyped,
}

fn parse_value(raw: &str, kind: Option<crate::options::registry::OptionValueKind>) -> Value {
    use crate::options::registry::OptionValueKind as Kind;
    // Registry-typed vector options keep their vector shape regardless of
    // element count — upstream vectors stay `is_vector()` even with a
    // single element, and a bare brace reference then enters
    // `vector_element_to_expr` (`PlaceholderParser.cpp:1086-1211`) instead
    // of scalar resolution.
    let (scalar, nullable) = match kind {
        Some(Kind::Int) => return parse_scalar(raw, Scalar::Int),
        Some(Kind::Float | Kind::Percent) => return parse_scalar(raw, Scalar::Float),
        Some(Kind::String | Kind::Enum) => return parse_scalar(raw, Scalar::Text),
        Some(Kind::Bool) => return untyped_list(raw, Scalar::Bool),
        Some(Kind::Ints) => (Scalar::Int, false),
        Some(Kind::IntsNullable) => (Scalar::Int, true),
        Some(Kind::Floats | Kind::Percents) => (Scalar::Float, false),
        Some(Kind::FloatsNullable | Kind::PercentsNullable) => (Scalar::Float, true),
        Some(Kind::Bools) => (Scalar::Bool, false),
        Some(Kind::BoolsNullable) => (Scalar::Bool, true),
        Some(Kind::Strings) => (Scalar::Text, false),
        Some(Kind::Enums | Kind::EnumsNullable) => (Scalar::Untyped, false),
        _ => return untyped_list(raw, Scalar::Untyped),
    };
    Value::List(
        raw.split([';', ','])
            .map(|element| {
                if nullable && element.trim() == "nil" {
                    Value::Nil
                } else {
                    parse_scalar(element, scalar)
                }
            })
            .collect(),
    )
}

/// An option without a registry vector kind: `;` or `,` separated values become a list.
fn untyped_list(raw: &str, scalar: Scalar) -> Value {
    let mut values = raw
        .split(';')
        .map(|value| parse_scalar(value, scalar))
        .collect::<Vec<_>>();
    if values.len() == 1 {
        let value = values.remove(0);
        let comma_values = value
            .as_string()
            .split(',')
            .map(|value| parse_scalar(value, scalar))
            .collect::<Vec<_>>();
        return if comma_values.len() == 1 {
            value
        } else {
            Value::List(comma_values)
        };
    }
    Value::List(values)
}

fn parse_scalar(raw: &str, scalar: Scalar) -> Value {
    let raw = raw.trim().trim_matches('"');
    match scalar {
        Scalar::Bool if matches!(raw, "0" | "1" | "false" | "true") => {
            Value::option_bool(matches!(raw, "1" | "true"))
        }
        Scalar::Int => Value::Integer(raw.parse().unwrap()),
        Scalar::Float => raw
            .trim_end_matches('%')
            .parse()
            .map_or_else(|_| untyped(raw), Value::Number),
        Scalar::Text => Value::String(raw.to_owned()),
        Scalar::Bool | Scalar::Untyped => untyped(raw),
    }
}

fn untyped(raw: &str) -> Value {
    if let Ok(value) = raw.parse::<f64>() {
        Value::Number(value)
    } else if raw == "true" || raw == "false" {
        Value::Bool(raw == "true")
    } else {
        Value::String(raw.to_owned())
    }
}
