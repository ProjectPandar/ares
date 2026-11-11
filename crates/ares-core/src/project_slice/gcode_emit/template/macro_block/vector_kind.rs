//! The vector option types a placeholder script copies and fills (`ConfigOptionType`,
//! `PlaceholderParser.cpp:1428-1818`).

use super::super::super::value::Value;
use crate::options::registry::{OptionValueKind as Kind, option_definition};

/// The type of a vector variable: the four a script creates, and those it only reads.
#[derive(Clone, Copy, PartialEq)]
pub(in super::super) enum VectorKind {
    Floats,
    Ints,
    Strings,
    Bools,
    Percents,
    /// `coFloatsOrPercents`, `coEnums`, `coPoints` and `coPointsGroups`.
    Other,
}

impl VectorKind {
    /// The type of a system vector: its option type, or for a runtime value the type of its
    /// elements. A scalar has none.
    pub(super) fn of_config(name: &str, value: &Value) -> Option<Self> {
        match option_definition(name).map(|definition| definition.kind) {
            Some(Kind::Floats | Kind::FloatsNullable) => Some(Self::Floats),
            Some(Kind::Ints | Kind::IntsNullable) => Some(Self::Ints),
            Some(Kind::Strings) => Some(Self::Strings),
            Some(Kind::Bools | Kind::BoolsNullable) => Some(Self::Bools),
            Some(Kind::Percents | Kind::PercentsNullable) => Some(Self::Percents),
            Some(
                Kind::FloatsOrPercents
                | Kind::Enums
                | Kind::EnumsNullable
                | Kind::Points
                | Kind::PointsGroups,
            ) => Some(Self::Other),
            Some(
                Kind::Float
                | Kind::FloatOrPercent
                | Kind::Percent
                | Kind::Int
                | Kind::Bool
                | Kind::Enum
                | Kind::String
                | Kind::Point,
            ) => None,
            None => match value {
                Value::List(values) => Some(values.first().map_or(Self::Other, Self::of_element)),
                _ => None,
            },
        }
    }

    /// The vector `repeat` creates from a value of this type (`vector_variable_new_from_array`,
    /// `:1617-1622`).
    pub(super) fn of_element(value: &Value) -> Self {
        match value.scalar() {
            Value::Integer(_) => Self::Ints,
            Value::Nil | Value::Number(_) => Self::Floats,
            Value::String(_) => Self::Strings,
            Value::Bool(_) | Value::OptionBool(_) => Self::Bools,
            Value::List(_) => Self::Other,
        }
    }

    /// A value of the element type, which an assigned element is converted to.
    pub(super) fn prototype(self) -> Option<Value> {
        match self {
            Self::Floats | Self::Percents => Some(Value::Number(0.0)),
            Self::Ints => Some(Value::Integer(0)),
            Self::Strings => Some(Value::String(String::new())),
            Self::Bools => Some(Value::Bool(false)),
            Self::Other => None,
        }
    }

    /// `ConfigOptionVector::is_nil` (`Config.hpp:901, 1085, 1978`): every element is nil, as in
    /// an empty vector; a string or other vector is never nil.
    pub(super) fn is_nil(self, values: &[Value]) -> bool {
        matches!(
            self,
            Self::Floats | Self::Ints | Self::Bools | Self::Percents
        ) && values.iter().all(|value| matches!(value, Value::Nil))
    }

    /// The vector a copy declares (`vector_variable_new_from_copy`, `:1806-1813`): percents
    /// become floats.
    pub(super) fn duplicate(self) -> Option<Self> {
        match self {
            Self::Floats | Self::Percents => Some(Self::Floats),
            Self::Ints | Self::Strings | Self::Bools => Some(self),
            Self::Other => None,
        }
    }

    /// The failure of copying another vector type into this one
    /// (`copy_vector_variable_to_vector_variable`, `:1767-1782`).
    pub(super) fn mismatch(self) -> &'static str {
        match self {
            Self::Floats => "Left hand side is a float vector, while the right hand side is not.",
            Self::Ints => "Left hand side is an int vector, while the right hand side is not.",
            Self::Strings => "Left hand side is a string vector, while the right hand side is not.",
            Self::Bools => "Left hand side is a bool vector, while the right hand side is not.",
            Self::Percents | Self::Other => {
                "Left hand side / right hand side vectors are not compatible."
            }
        }
    }
}
