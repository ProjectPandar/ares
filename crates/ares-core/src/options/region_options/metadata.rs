#![cfg_attr(not(test), allow(dead_code))]

use std::{iter::Peekable, str::Chars};

use serde::{
    Deserialize,
    de::value::{BorrowedStrDeserializer, Error as ValueError},
};

use super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, OrcaInts, OrcaString, OrcaStrings, Percent,
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessInfillPattern, ProcessIroningType, ProcessNoiseType,
    ProcessSeamScarfType, ProcessWallDirection, ProcessWallSequence,
};

pub(super) trait RegionMetadataCodec: Sized {
    fn deserialize_metadata(value: &str) -> Result<Self, String>;
}

macro_rules! lexical_codecs {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl RegionMetadataCodec for $ty {
                fn deserialize_metadata(value: &str) -> Result<Self, String> {
                    Self::deserialize(BorrowedStrDeserializer::<ValueError>::new(value))
                        .map_err(|error| error.to_string())
                }
            }
        )+
    };
}

lexical_codecs!(
    FloatOrPercent,
    OrcaBool,
    OrcaFloat,
    OrcaInt,
    Percent,
    ProcessCounterboreHoleBridging,
    ProcessEnsureVerticalShellThickness,
    ProcessFuzzySkinMode,
    ProcessFuzzySkinType,
    ProcessInfillPattern,
    ProcessIroningType,
    ProcessNoiseType,
    ProcessSeamScarfType,
    ProcessWallDirection,
    ProcessWallSequence,
);

impl RegionMetadataCodec for OrcaInts {
    fn deserialize_metadata(value: &str) -> Result<Self, String> {
        if value.is_empty() {
            return Ok(Self(Vec::new()));
        }
        value
            .split(',')
            .map(|item| {
                OrcaInt::deserialize_metadata(item)
                    .map_err(|error| format!("invalid integer vector element: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

impl RegionMetadataCodec for OrcaString {
    fn deserialize_metadata(value: &str) -> Result<Self, String> {
        unescape_string_cstyle(value).map(Self)
    }
}

impl RegionMetadataCodec for OrcaStrings {
    fn deserialize_metadata(value: &str) -> Result<Self, String> {
        unescape_strings_cstyle(value).map(Self)
    }
}

fn unescape_string_cstyle(value: &str) -> Result<String, String> {
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

fn unescape_strings_cstyle(value: &str) -> Result<Vec<String>, String> {
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
