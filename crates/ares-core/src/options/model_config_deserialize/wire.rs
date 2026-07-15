use std::{iter::Peekable, str::Chars};

use serde::de::{
    DeserializeOwned,
    value::{Error as ValueError, SeqDeserializer, StringDeserializer},
};

use super::super::{
    FloatOrPercent, Nullable, OrcaBool, OrcaBools, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts,
    OrcaPercents, OrcaString, OrcaStrings, Percent, Point2d, Point2dGroups, Point2dList,
};
use super::OptionValueKind;

pub(super) enum WireValue {
    Scalar(String),
    Sequence(Vec<String>),
}

pub(super) fn decode(kind: OptionValueKind, value: &str) -> Result<WireValue, String> {
    match kind {
        OptionValueKind::String => decode_scalar_string(value).map(WireValue::Scalar),
        OptionValueKind::Strings => decode_string_vector(value).map(WireValue::Sequence),
        OptionValueKind::Point => Ok(WireValue::Scalar(value.to_owned())),
        OptionValueKind::Points => Ok(WireValue::Sequence(split_flat(value))),
        OptionValueKind::PointsGroups => Ok(WireValue::Sequence(split_groups(value))),
        OptionValueKind::Floats if value.is_empty() => {
            Ok(WireValue::Sequence(vec!["0".to_owned()]))
        }
        OptionValueKind::Percents
        | OptionValueKind::PercentsNullable
        | OptionValueKind::Bools
        | OptionValueKind::BoolsNullable
        | OptionValueKind::Enums
        | OptionValueKind::EnumsNullable
        | OptionValueKind::Floats
        | OptionValueKind::FloatsNullable
        | OptionValueKind::IntsNullable
        | OptionValueKind::Ints => Ok(WireValue::Sequence(split_flat(value))),
        OptionValueKind::Float
        | OptionValueKind::FloatOrPercent
        | OptionValueKind::Percent
        | OptionValueKind::Int
        | OptionValueKind::Bool
        | OptionValueKind::Enum => Ok(WireValue::Scalar(value.to_owned())),
    }
}

fn split_flat(value: &str) -> Vec<String> {
    value
        .split_terminator(',')
        .map(str::trim)
        .map(str::to_owned)
        .collect()
}

fn split_groups(value: &str) -> Vec<String> {
    value
        .split_terminator('#')
        .map(str::trim)
        .map(str::to_owned)
        .collect()
}

pub(super) fn decode_scalar_string(value: &str) -> Result<String, String> {
    let mut input = value.chars();
    let mut output = String::with_capacity(value.len());
    while let Some(character) = input.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        output.push(unescape_character(
            input
                .next()
                .ok_or_else(|| "trailing escape in Orca string".to_owned())?,
        ));
    }
    Ok(output)
}

fn decode_string_vector(value: &str) -> Result<Vec<String>, String> {
    if value.is_empty() {
        return Ok(Vec::new());
    }

    let mut input = value.chars().peekable();
    let mut output = Vec::new();
    loop {
        while matches!(input.peek(), Some(' ' | '\t')) {
            input.next();
        }
        let Some(&first) = input.peek() else {
            return Ok(output);
        };
        let item = if first == '"' {
            input.next();
            read_quoted_string(&mut input)?
        } else {
            read_unquoted_string(&mut input)
        };
        output.push(item);

        if first == '"' {
            while matches!(input.peek(), Some(' ' | '\t')) {
                input.next();
            }
        }
        match input.next() {
            None => return Ok(output),
            Some(';') if input.peek().is_none() => {
                output.push(String::new());
                return Ok(output);
            }
            Some(';') => {}
            Some(_) => return Err("expected semicolon after quoted Orca string".to_owned()),
        }
    }
}

fn read_quoted_string(input: &mut Peekable<Chars<'_>>) -> Result<String, String> {
    let mut output = String::new();
    loop {
        match input.next() {
            Some('"') => return Ok(output),
            Some('\\') => {
                output.push(unescape_character(input.next().ok_or_else(|| {
                    "trailing escape in quoted Orca string".to_owned()
                })?))
            }
            Some(character) => output.push(character),
            None => return Err("unterminated quoted Orca string".to_owned()),
        }
    }
}

fn read_unquoted_string(input: &mut Peekable<Chars<'_>>) -> String {
    let mut output = String::new();
    while input.peek().is_some_and(|character| *character != ';') {
        output.push(input.next().unwrap());
    }
    output
}

fn unescape_character(character: char) -> char {
    match character {
        'r' => '\r',
        'n' => '\n',
        other => other,
    }
}

pub(super) fn deserialize_scalar<T>(value: &str) -> Result<(), String>
where
    T: DeserializeOwned,
{
    T::deserialize(StringDeserializer::<ValueError>::new(value.to_owned()))
        .map(drop)
        .map_err(|error| error.to_string())
}

pub(super) fn deserialize_sequence<T>(kind: OptionValueKind, value: &str) -> Result<(), String>
where
    T: DeserializeOwned,
{
    let WireValue::Sequence(values) = decode(kind, value)? else {
        unreachable!("sequence kind decoded as scalar")
    };
    T::deserialize(SeqDeserializer::<_, ValueError>::new(values.into_iter()))
        .map(drop)
        .map_err(|error| error.to_string())
}

pub(super) fn validate_lexical(kind: OptionValueKind, value: &str) -> Result<(), String> {
    match kind {
        OptionValueKind::Float => deserialize_scalar::<OrcaFloat>(value),
        OptionValueKind::FloatOrPercent => deserialize_scalar::<FloatOrPercent>(value),
        OptionValueKind::Percent => deserialize_scalar::<Percent>(value),
        OptionValueKind::Percents => deserialize_sequence::<OrcaPercents>(kind, value),
        OptionValueKind::PercentsNullable => {
            deserialize_sequence::<Vec<Nullable<Percent>>>(kind, value)
        }
        OptionValueKind::Int => deserialize_scalar::<OrcaInt>(value),
        OptionValueKind::Bool => deserialize_scalar::<OrcaBool>(value),
        OptionValueKind::Bools => deserialize_sequence::<OrcaBools>(kind, value),
        OptionValueKind::BoolsNullable => {
            deserialize_sequence::<Vec<Nullable<OrcaBool>>>(kind, value)
        }
        OptionValueKind::Enum => Ok(()),
        OptionValueKind::Enums => Ok(()),
        OptionValueKind::EnumsNullable => Ok(()),
        OptionValueKind::Floats => deserialize_sequence::<OrcaFloats>(kind, value),
        OptionValueKind::FloatsNullable => {
            deserialize_sequence::<Vec<Nullable<OrcaFloat>>>(kind, value)
        }
        OptionValueKind::IntsNullable => {
            deserialize_sequence::<Vec<Nullable<OrcaInt>>>(kind, value)
        }
        OptionValueKind::Ints => deserialize_sequence::<OrcaInts>(kind, value),
        OptionValueKind::Strings => deserialize_sequence::<OrcaStrings>(kind, value),
        OptionValueKind::String => deserialize_scalar::<OrcaString>(&decode_scalar_string(value)?),
        OptionValueKind::Point => deserialize_scalar::<Point2d>(value),
        OptionValueKind::Points => deserialize_sequence::<Point2dList>(kind, value),
        OptionValueKind::PointsGroups => deserialize_sequence::<Point2dGroups>(kind, value),
    }
}
