use serde::Deserialize;

use crate::{OptionValueKind, SliceError};

#[derive(Debug, PartialEq)]
enum Value {
    Number(f64),
    Int(i32),
    Bool(bool),
    Text(String),
    Ratio(Ratio),
    Point([f64; 2]),
    Numbers(Vec<Option<f64>>),
    Ints(Vec<Option<i32>>),
    Bools(Vec<Option<bool>>),
    Texts(Vec<Option<String>>),
    Ratios(Vec<Ratio>),
    Points(Vec<[f64; 2]>),
    Groups(Vec<Vec<[f64; 2]>>),
}

#[derive(Debug, Deserialize, PartialEq)]
struct Ratio {
    value: f64,
    percent: bool,
}

pub(super) struct Typed {
    kind: OptionValueKind,
    value: Value,
}

fn decode<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, SliceError> {
    serde_json::from_str(json).map_err(|error| SliceError::InvalidInput(error.to_string()))
}

impl Typed {
    pub(super) fn parse(kind: OptionValueKind, wire: &str) -> Result<Self, SliceError> {
        use OptionValueKind as K;
        let json = crate::decode_config_value(kind, wire).map_err(SliceError::InvalidInput)?;
        let value = match kind {
            K::Float | K::Percent => Value::Number(decode(&json)?),
            K::Int => Value::Int(decode(&json)?),
            K::Bool => Value::Bool(decode(&json)?),
            K::String | K::Enum => Value::Text(decode(&json)?),
            K::FloatOrPercent => Value::Ratio(decode(&json)?),
            K::Point => Value::Point(decode(&json)?),
            K::Floats | K::FloatsNullable | K::Percents | K::PercentsNullable => {
                Value::Numbers(decode(&json)?)
            }
            K::Ints => Value::Ints(decode(&json)?),
            K::IntsNullable => Value::Ints(
                decode::<Vec<Option<i32>>>(&json)?
                    .into_iter()
                    .map(|value| value.filter(|value| *value != i32::MAX))
                    .collect(),
            ),
            K::Bools | K::BoolsNullable => Value::Bools(decode(&json)?),
            K::Strings | K::Enums | K::EnumsNullable => Value::Texts(decode(&json)?),
            K::FloatsOrPercents => Value::Ratios(decode(&json)?),
            K::Points => Value::Points(decode(&json)?),
            K::PointsGroups => Value::Groups(decode(&json)?),
        };
        Ok(Self { kind, value })
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        if self.kind != other.kind {
            return false;
        }
        match (&self.value, &other.value) {
            (Value::Number(left), Value::Number(right)) => approximate(*left, *right),
            (Value::Ratio(left), Value::Ratio(right)) => {
                left.percent == right.percent && approximate(left.value, right.value)
            }
            (Value::Numbers(left), Value::Numbers(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right)
                        .all(|(left, right)| match (left, right) {
                            (Some(left), Some(right)) => approximate(*left, *right),
                            (None, None) => true,
                            _ => false,
                        })
            }
            (Value::Ratios(left), Value::Ratios(right)) => {
                left.len() == right.len()
                    && left.iter().zip(right).all(|(left, right)| {
                        left.percent == right.percent && approximate(left.value, right.value)
                    })
            }
            _ => self.value == other.value,
        }
    }
}

// Config.hpp:834,985-995,1355-1356,1501-1512; libslic3r.h:57,319-323.
fn approximate(left: f64, right: f64) -> bool {
    (left - right).abs() < 1e-4
}
