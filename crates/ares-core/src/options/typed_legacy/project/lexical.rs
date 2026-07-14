use serde::de::{
    Error,
    value::{SeqDeserializer, StringDeserializer},
};

use super::super::{LegacyRule, VectorType};
use crate::options::project_settings::ProjectSettingsBuilder;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ValueOrigin {
    Scalar,
    Array,
}

pub(super) fn assign<E>(
    builder: &mut ProjectSettingsBuilder,
    rule: &LegacyRule,
    target: &str,
    value: String,
    origin: ValueOrigin,
) -> Result<(), E>
where
    E: Error,
{
    let source = rule.source;
    let handled = if let Some(vector) = rule.wire.vector {
        let values = parse_vector(&value, vector, origin).map_err(|reason| {
            E::custom(format_args!(
                "invalid legacy Orca option {source} for {target}: {reason}"
            ))
        })?;
        builder.deserialize_known_value(target, SeqDeserializer::<_, E>::new(values.into_iter()))
    } else {
        builder.deserialize_known_value(target, StringDeserializer::<E>::new(value))
    }
    .map_err(|error| {
        E::custom(format_args!(
            "invalid legacy Orca option {source} for {target}: {error}"
        ))
    })?;

    if handled {
        Ok(())
    } else {
        Err(E::custom(format_args!(
            "legacy Orca option {source} resolved to unknown target {target}"
        )))
    }
}

fn parse_vector(
    value: &str,
    vector: VectorType,
    origin: ValueOrigin,
) -> Result<Vec<String>, &'static str> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    match vector {
        VectorType::Ints | VectorType::Bools | VectorType::Enums => {
            if value.contains('#') {
                return Err("nested group is invalid for a flat concrete vector");
            }
            Ok(value.split(',').map(str::to_owned).collect())
        }
        VectorType::Strings => parse_strings(value, origin),
    }
}

fn parse_strings(value: &str, origin: ValueOrigin) -> Result<Vec<String>, &'static str> {
    enum Mode {
        Start,
        Unquoted,
        Quoted,
        QuotedEscape,
        QuotedClosed,
    }

    let mut values = Vec::new();
    let mut current = String::new();
    let mut mode = Mode::Start;
    let mut last_was_delimiter = false;
    let mut skipped_after_delimiter = false;

    for character in value.chars() {
        match mode {
            Mode::Start => match character {
                ' ' | '\t' => skipped_after_delimiter = last_was_delimiter,
                ';' => {
                    values.push(String::new());
                    last_was_delimiter = true;
                    skipped_after_delimiter = false;
                }
                '"' => {
                    mode = Mode::Quoted;
                    last_was_delimiter = false;
                    skipped_after_delimiter = false;
                }
                '#' if origin == ValueOrigin::Array => {
                    return Err("nested group is invalid for a flat concrete vector");
                }
                character => {
                    current.push(character);
                    mode = Mode::Unquoted;
                    last_was_delimiter = false;
                    skipped_after_delimiter = false;
                }
            },
            Mode::Unquoted => match character {
                ';' => {
                    values.push(std::mem::take(&mut current));
                    mode = Mode::Start;
                    last_was_delimiter = true;
                    skipped_after_delimiter = false;
                }
                '#' if origin == ValueOrigin::Array => {
                    return Err("nested group is invalid for a flat concrete vector");
                }
                character => current.push(character),
            },
            Mode::Quoted => match character {
                '\\' => mode = Mode::QuotedEscape,
                '"' => mode = Mode::QuotedClosed,
                character => current.push(character),
            },
            Mode::QuotedEscape => {
                current.push(match character {
                    'n' => '\n',
                    'r' => '\r',
                    character => character,
                });
                mode = Mode::Quoted;
            }
            Mode::QuotedClosed => match character {
                ' ' | '\t' => {}
                ';' => {
                    values.push(std::mem::take(&mut current));
                    mode = Mode::Start;
                    last_was_delimiter = true;
                    skipped_after_delimiter = false;
                }
                _ => return Err("characters after closing quote in string vector"),
            },
        }
    }

    match mode {
        Mode::Start => {
            if last_was_delimiter && !skipped_after_delimiter {
                values.push(String::new());
            }
        }
        Mode::Unquoted | Mode::QuotedClosed => values.push(current),
        Mode::Quoted | Mode::QuotedEscape => return Err("unterminated C-style string vector"),
    }
    Ok(values)
}
