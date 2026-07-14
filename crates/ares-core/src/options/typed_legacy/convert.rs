use std::fmt;

use serde::{
    Deserialize, Deserializer,
    de::{IgnoredAny, SeqAccess, Visitor},
};

use super::{Comparison, JsonArrayAllowance, LegacyAction, LegacyRule, Replacement, VectorType};

mod obsolete;

pub(crate) use obsolete::transform_obsolete;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LegacyOutcome {
    Assign {
        target: &'static str,
        value: String,
    },
    Consume,
    Deferred {
        source: &'static str,
        target: Option<&'static str>,
        recursive: bool,
    },
    Error(LegacyTransformError),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LegacyTransformError {
    InvalidArrayValue { source: &'static str },
}

pub(crate) fn transform_lexical(rule: &LegacyRule, value: &str) -> LegacyOutcome {
    match rule.action {
        LegacyAction::Rename { target } => assign(target, value),
        LegacyAction::FeatureFilament {
            target,
            legacy_inherit,
            canonical_inherit,
        } => assign(
            target,
            if value == legacy_inherit {
                canonical_inherit
            } else {
                value
            },
        ),
        LegacyAction::ConsumeIfContains { needle } => {
            if value.contains(needle) {
                LegacyOutcome::Consume
            } else {
                assign(rule.source, value)
            }
        }
        LegacyAction::TopOneWall {
            target,
            consume,
            replacement,
        } => {
            if value == consume {
                LegacyOutcome::Consume
            } else {
                assign(target, replacement)
            }
        }
        LegacyAction::PrimeTowerRib {
            target,
            trigger,
            replacement,
        } => {
            if value == trigger {
                assign(target, replacement)
            } else {
                LegacyOutcome::Consume
            }
        }
        LegacyAction::Rewrite {
            target,
            comparison,
            replacements,
        } => assign(target, rewrite(value, comparison, replacements)),
        LegacyAction::WallOrder {
            target,
            replacements,
        } => assign(target, replace_exact(value, replacements).unwrap_or(value)),
        LegacyAction::ReplaceAll {
            target,
            replacements,
        } => assign(target, replace_all(value, replacements)),
        LegacyAction::FilamentTokenRebuild { target, from, to } => {
            assign(target, rebuild_filament_tokens(value, from, to))
        }
        LegacyAction::DeferredProfileBookkeeping { target, recursive } => LegacyOutcome::Deferred {
            source: rule.source,
            target,
            recursive,
        },
    }
}

pub(crate) fn array_first_pass(rule: &LegacyRule) -> LegacyOutcome {
    transform_lexical(rule, "")
}

pub(crate) fn transform_json_array<'de, D>(rule: &LegacyRule, deserializer: D) -> LegacyOutcome
where
    D: Deserializer<'de>,
{
    let first_pass = array_first_pass(rule);
    match first_pass {
        LegacyOutcome::Consume => consume_json(deserializer, rule.source, LegacyOutcome::Consume),
        deferred @ LegacyOutcome::Deferred { .. } => {
            consume_json(deserializer, rule.source, deferred)
        }
        LegacyOutcome::Error(error) => LegacyOutcome::Error(error),
        LegacyOutcome::Assign { value, .. } => match rule.wire.json_array {
            JsonArrayAllowance::RejectAfterFirstPass => {
                consume_json(deserializer, rule.source, invalid_array(rule.source))
            }
            JsonArrayAllowance::Flatten(vector) => {
                flatten_json_array(deserializer, rule, vector, value)
            }
            JsonArrayAllowance::ConsumeFirstPass => {
                consume_json(deserializer, rule.source, LegacyOutcome::Consume)
            }
            JsonArrayAllowance::Deferred => consume_json(
                deserializer,
                rule.source,
                LegacyOutcome::Deferred {
                    source: rule.source,
                    target: None,
                    recursive: false,
                },
            ),
        },
    }
}

fn flatten_json_array<'de, D>(
    deserializer: D,
    rule: &LegacyRule,
    vector: VectorType,
    mut first_value: String,
) -> LegacyOutcome
where
    D: Deserializer<'de>,
{
    let Ok(values) = Vec::<ArrayItem>::deserialize(deserializer) else {
        return invalid_array(rule.source);
    };
    if !append_array(&mut first_value, &values, vector) {
        return invalid_array(rule.source);
    }
    transform_lexical(rule, &first_value)
}

enum ArrayItem {
    String(String),
    Array(Vec<String>),
}

impl<'de> Deserialize<'de> for ArrayItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ArrayItemVisitor)
    }
}

struct ArrayItemVisitor;

impl<'de> Visitor<'de> for ArrayItemVisitor {
    type Value = ArrayItem;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string or one nested array of strings")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E> {
        Ok(ArrayItem::String(value.to_owned()))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(ArrayItem::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(ArrayItem::String(value))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(value) = sequence.next_element()? {
            values.push(value);
        }
        Ok(ArrayItem::Array(values))
    }
}

fn append_array(output: &mut String, values: &[ArrayItem], vector: VectorType) -> bool {
    let Some(first) = values.first() else {
        return true;
    };
    match first {
        ArrayItem::String(_) => {
            if values
                .iter()
                .any(|value| !matches!(value, ArrayItem::String(_)))
            {
                return false;
            }
            append_strings(
                output,
                values.iter().map(|value| match value {
                    ArrayItem::String(value) => value.as_str(),
                    ArrayItem::Array(_) => unreachable!(),
                }),
                vector,
            );
        }
        ArrayItem::Array(_) => {
            if values
                .iter()
                .any(|value| !matches!(value, ArrayItem::Array(_)))
            {
                return false;
            }
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push('#');
                }
                let ArrayItem::Array(value) = value else {
                    unreachable!();
                };
                append_strings(output, value.iter().map(String::as_str), vector);
            }
        }
    }
    true
}

fn append_strings<'a>(
    output: &mut String,
    values: impl Iterator<Item = &'a str>,
    vector: VectorType,
) {
    let separator = match vector {
        VectorType::Ints | VectorType::Bools | VectorType::Enums => ',',
        VectorType::Strings => ';',
    };
    for (index, value) in values.enumerate() {
        if index != 0 {
            output.push(separator);
        }
        match vector {
            VectorType::Ints | VectorType::Bools | VectorType::Enums => output.push_str(value),
            VectorType::Strings => push_cstyle_quoted(output, value),
        }
    }
}

fn consume_json<'de, D>(
    deserializer: D,
    source: &'static str,
    outcome: LegacyOutcome,
) -> LegacyOutcome
where
    D: Deserializer<'de>,
{
    match IgnoredAny::deserialize(deserializer) {
        Ok(_) => outcome,
        Err(_) => invalid_array(source),
    }
}

fn rewrite<'a>(value: &'a str, comparison: Comparison, replacements: &[Replacement]) -> &'a str {
    replacements
        .iter()
        .find(|replacement| match comparison {
            Comparison::Exact => value == replacement.from,
            Comparison::AsciiCaseInsensitive => value.eq_ignore_ascii_case(replacement.from),
            Comparison::Leading => value.starts_with(replacement.from),
        })
        .map_or(value, |replacement| replacement.to)
}

fn replace_exact(value: &str, replacements: &[Replacement]) -> Option<&'static str> {
    replacements
        .iter()
        .find(|replacement| value == replacement.from)
        .map(|replacement| replacement.to)
}

fn replace_all(value: &str, replacements: &[Replacement]) -> String {
    replacements
        .iter()
        .fold(value.to_owned(), |value, replacement| {
            value.replace(replacement.from, replacement.to)
        })
}

fn rebuild_filament_tokens(value: &str, from: &str, to: &str) -> String {
    let mut changed = false;
    let tokens = value
        .split_terminator(';')
        .map(|token| {
            let token = token
                .strip_prefix('"')
                .and_then(|token| token.strip_suffix('"'))
                .unwrap_or(token);
            if token == from {
                changed = true;
                to
            } else {
                token
            }
        })
        .collect::<Vec<_>>();

    if changed {
        tokens
            .into_iter()
            .map(|token| format!("\"{token}\""))
            .collect::<Vec<_>>()
            .join(";")
    } else {
        value.to_owned()
    }
}

fn push_cstyle_quoted(output: &mut String, value: &str) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            character => output.push(character),
        }
    }
    output.push('"');
}

fn assign(target: &'static str, value: impl Into<String>) -> LegacyOutcome {
    LegacyOutcome::Assign {
        target,
        value: value.into(),
    }
}

fn invalid_array(source: &'static str) -> LegacyOutcome {
    LegacyOutcome::Error(LegacyTransformError::InvalidArrayValue { source })
}
