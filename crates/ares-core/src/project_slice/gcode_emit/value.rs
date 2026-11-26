use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

mod parse;
mod random;
pub(super) mod variables;
use parse::parse_value;
use variables::{Scope, Variable, Variables, VectorKind};

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
    base: Option<Arc<HashMap<String, Value>>>,
    values: HashMap<String, Option<Value>>,
    pub(crate) random: Option<TemplateRng>,
    pub(super) globals: Variables,
    pub(super) locals: Variables,
}

impl Config {
    pub(crate) fn from_shared_base(
        base: Arc<HashMap<String, Value>>,
        random: Option<TemplateRng>,
    ) -> Self {
        Self {
            base: Some(base),
            values: HashMap::new(),
            random,
            globals: Variables::default(),
            locals: Variables::default(),
        }
    }

    pub(crate) fn into_shared_map(self) -> Arc<HashMap<String, Value>> {
        let mut map = self
            .base
            .map(|base| (*base).clone())
            .unwrap_or_else(|| HashMap::with_capacity(self.values.len()));
        for (key, value) in self.values {
            match value {
                Some(value) => {
                    map.insert(key, value);
                }
                None => {
                    map.remove(&key);
                }
            }
        }
        Arc::new(map)
    }

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
                self.values
                    .insert(format!("{key}_{index}"), Some(item.clone()));
            }
        }
        self.values.insert(key, Some(value));
    }

    /// Stores an assignment target (`{var[idx] = value}`): lists grow with
    /// zero padding like the upstream per-extruder vectors.
    pub(crate) fn assign(&mut self, name: &str, index: Option<usize>, value: Value) {
        let value = match index {
            None => value,
            Some(index) => {
                let mut values = match self.resolve(name) {
                    Some(Value::List(existing)) => existing.clone(),
                    Some(other) => vec![other.clone()],
                    None => Vec::new(),
                };
                if values.len() <= index {
                    values.resize(index + 1, Value::Integer(0));
                }
                values[index] = value;
                Value::List(values)
            }
        };
        if self.get(name).is_none()
            && let Some(variable) = self
                .globals
                .get_mut(name)
                .or_else(|| self.locals.get_mut(name))
        {
            variable.value = value;
        } else {
            if index.is_none() {
                self.remove(name);
            }
            self.insert(name, value);
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        match self.values.get(key) {
            Some(value) => value.as_ref(),
            None => self.base.as_deref().and_then(|base| base.get(key)),
        }
    }

    /// Brace resolution (`PlaceholderParser.cpp:992-1022`). Bracket expansion
    /// continues to use `get`, excluding script declarations.
    pub(super) fn resolve(&self, name: &str) -> Option<&Value> {
        self.get(name)
            .or_else(|| self.globals.get(name).map(|variable| &variable.value))
            .or_else(|| self.locals.get(name).map(|variable| &variable.value))
    }

    pub(super) fn declaration(&self, name: &str) -> Option<(Scope, Option<VectorKind>)> {
        if self.get(name).is_some() {
            return None;
        }
        self.globals
            .get(name)
            .map(|var| (Scope::Global, var.kind))
            .or_else(|| self.locals.get(name).map(|var| (Scope::Local, var.kind)))
    }

    pub(super) fn declare(
        &mut self,
        name: &str,
        scope: Scope,
        kind: Option<VectorKind>,
        value: Value,
    ) {
        let variables = match scope {
            Scope::Global => &mut self.globals,
            Scope::Local => &mut self.locals,
        };
        variables.insert(name.to_owned(), Variable { value, kind });
    }

    fn remove(&mut self, key: &str) {
        let list_len = match self.get(key) {
            Some(Value::List(values)) => Some(values.len()),
            _ => None,
        };
        self.remove_one(key);
        if let Some(len) = list_len {
            for index in 0..len {
                self.remove_one(&format!("{key}_{index}"));
            }
        }
    }

    fn remove_one(&mut self, key: &str) {
        if self
            .base
            .as_deref()
            .is_some_and(|base| base.contains_key(key))
        {
            self.values.insert(key.to_owned(), None);
        } else {
            self.values.remove(key);
        }
    }
}
