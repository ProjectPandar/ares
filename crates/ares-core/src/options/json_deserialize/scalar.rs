//! Config.hpp scalar stream extraction, confined to Config.cpp preset JSON inputs.
use std::borrow::Cow;

use super::OptionValueKind;

pub(super) fn is_scalar(kind: OptionValueKind) -> bool {
    matches!(
        kind,
        OptionValueKind::Float
            | OptionValueKind::Int
            | OptionValueKind::Percent
            | OptionValueKind::FloatOrPercent
            | OptionValueKind::Bool
            | OptionValueKind::Enum
    )
}

pub(super) fn normalize(kind: OptionValueKind, value: &str) -> Result<Cow<'_, str>, String> {
    match kind {
        OptionValueKind::Float
        | OptionValueKind::Int
        | OptionValueKind::Percent
        | OptionValueKind::FloatOrPercent => {
            let prefix = numeric_prefix(value, kind == OptionValueKind::Int)?;
            if kind == OptionValueKind::FloatOrPercent && value.contains('%') {
                Ok(Cow::Owned(format!("{prefix}%")))
            } else {
                Ok(Cow::Borrowed(prefix))
            }
        }
        OptionValueKind::Bool => {
            let first = value
                .split(',')
                .next()
                .unwrap()
                .trim_matches(|c: char| c.is_ascii_whitespace());
            match first {
                "0" | "1" => Ok(Cow::Borrowed(first)),
                _ => Err("invalid Orca boolean".to_owned()),
            }
        }
        _ => Ok(Cow::Borrowed(value)),
    }
}

fn numeric_prefix(value: &str, integer: bool) -> Result<&str, String> {
    let value = value.trim_start_matches(|c: char| c.is_ascii_whitespace());
    let bytes = value.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let digits_start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut has_digits = end != digits_start;
    if !integer && bytes.get(end) == Some(&b'.') {
        end += 1;
        let fractional_start = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        has_digits |= end != fractional_start;
    }
    if !has_digits {
        return Err("invalid numeric prefix".to_owned());
    }
    if !integer && matches!(bytes.get(end), Some(b'e' | b'E')) {
        end += 1;
        if matches!(bytes.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let exponent_start = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        if end == exponent_start {
            return Err("invalid numeric exponent".to_owned());
        }
    }
    Ok(&value[..end])
}
