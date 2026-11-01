//! Encode typed JSON values into `ConfigOption::serialize()`-form strings.
//!
//! The inverse of [`super::decode`]: the typed JSON shapes `decode_config_value`
//! emits are parsed into the `config_types` values the `CONFIG_BLOCK` writer
//! serializes, then rendered by the same `serialize_config_value` — so each
//! kind follows `OrcaSlicer/src/libslic3r/Config.hpp` `ConfigOption*::serialize`
//! (Float 777, Floats 845, Int 968, Ints 1023, String 1101, Strings 1135,
//! Percent 1180, Percents 1220, FloatOrPercent 1294, Point 1465, Points 1504,
//! PointsGroups 1581, Bool 1823, Bools 1894, Enum 2006, EnumGeneric 2076,
//! EnumsGeneric 2133) without a second copy of the lexical rules.

use serde::{Deserialize, Serialize};

use super::config_export::value::serialize_config_value;
use super::config_types::{
    FloatOrPercent, Nullable, OrcaBool, OrcaFloat, OrcaInt, OrcaStrings, Percent, Point2d,
    Point2dGroups, Point2dList, semantic::NullableVectorRef,
};
use super::registry::OptionValueKind;

/// `{"value": N, "percent": B}`, the decoded `float_or_percent` shape.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FloatOrPercentJson {
    value: f64,
    percent: bool,
}

impl From<FloatOrPercentJson> for FloatOrPercent {
    fn from(json: FloatOrPercentJson) -> Self {
        if json.percent {
            Self::Percent(Percent(json.value))
        } else {
            Self::Float(json.value)
        }
    }
}

fn parse<T: for<'de> Deserialize<'de>>(typed_json: &str) -> Result<T, String> {
    serde_json::from_str(typed_json).map_err(|error| error.to_string())
}

fn render<T: Serialize + ?Sized>(value: &T) -> Result<String, String> {
    serialize_config_value(value)
        .map(|serialized| serialized.token)
        .map_err(|error| error.to_string())
}

/// Elements of a nullable vector: JSON `null` is `nil`.
fn nullable<T, U>(typed_json: &str, convert: impl Fn(T) -> U) -> Result<String, String>
where
    T: for<'de> Deserialize<'de>,
    U: Serialize,
{
    let values: Vec<Nullable<U>> = parse::<Vec<Option<T>>>(typed_json)?
        .into_iter()
        .map(|value| value.map_or(Nullable::Nil, |value| Nullable::Value(convert(value))))
        .collect();
    render(&NullableVectorRef::new(&values))
}

fn vector<T, U>(typed_json: &str, convert: impl Fn(T) -> U) -> Result<String, String>
where
    T: for<'de> Deserialize<'de>,
    U: Serialize,
{
    let values: Vec<U> = parse::<Vec<T>>(typed_json)?
        .into_iter()
        .map(convert)
        .collect();
    render(&values)
}

fn point([x, y]: [f64; 2]) -> Point2d {
    Point2d::new(x, y)
}

/// Encodes `typed_json` (the JSON shape [`super::decode_config_value`] emits
/// for `kind`) into its `ConfigOption::serialize()` text.
pub fn encode_config_value(kind: OptionValueKind, typed_json: &str) -> Result<String, String> {
    use OptionValueKind as Kind;
    match kind {
        Kind::Bool => render(&OrcaBool(parse(typed_json)?)),
        Kind::Int => render(&OrcaInt(parse(typed_json)?)),
        Kind::Float => render(&OrcaFloat(parse(typed_json)?)),
        Kind::Percent => render(&Percent(parse(typed_json)?)),
        Kind::FloatOrPercent => render(&FloatOrPercent::from(parse::<FloatOrPercentJson>(
            typed_json,
        )?)),
        Kind::Enum | Kind::String => render(&parse::<String>(typed_json)?),
        Kind::Point => render(&point(parse(typed_json)?)),
        Kind::Bools => vector(typed_json, OrcaBool),
        Kind::BoolsNullable => nullable(typed_json, OrcaBool),
        Kind::Ints => vector(typed_json, OrcaInt),
        Kind::IntsNullable => nullable(typed_json, OrcaInt),
        Kind::Floats => vector(typed_json, OrcaFloat),
        Kind::FloatsOrPercents => vector(typed_json, |value: FloatOrPercentJson| {
            FloatOrPercent::from(value)
        }),
        Kind::FloatsNullable => nullable(typed_json, OrcaFloat),
        Kind::Percents => vector(typed_json, Percent),
        Kind::PercentsNullable => nullable(typed_json, Percent),
        Kind::Enums => vector(typed_json, |name: String| name),
        Kind::EnumsNullable => nullable(typed_json, |name: String| name),
        Kind::Strings => render(&OrcaStrings(parse(typed_json)?)),
        Kind::Points => render(&Point2dList(
            parse::<Vec<[f64; 2]>>(typed_json)?
                .into_iter()
                .map(point)
                .collect(),
        )),
        Kind::PointsGroups => render(&Point2dGroups(
            parse::<Vec<Vec<[f64; 2]>>>(typed_json)?
                .into_iter()
                .map(|group| group.into_iter().map(point).collect())
                .collect(),
        )),
    }
}

/// Resolves `key` in the option registry and encodes `typed_json` into its
/// `ConfigOption::serialize()` text. Unknown keys error.
pub fn encode_config_value_by_key(key: &str, typed_json: &str) -> Result<String, String> {
    use super::registry::option_definition;
    let definition =
        option_definition(key).ok_or_else(|| format!("unknown config option key {key:?}"))?;
    encode_config_value(definition.kind, typed_json)
}
