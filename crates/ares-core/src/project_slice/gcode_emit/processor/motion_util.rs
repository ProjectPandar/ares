//! G-code word parsing and kinematic clamp helpers shared by motion planning.

/// Parse a complete argument word like `GCodeReader.cpp:29-84`.
/// A leading `+` carries no value: `fast_float::from_chars` rejects it.
pub(super) fn word(code: &str, letter: char) -> Option<f64> {
    precise_word(code, letter).map(|value| f64::from(value as f32))
}

/// `GCodeReader::precise_value`, selected by the `process_G2_G3` end position
/// (`GCodeProcessor.cpp:6077`, `:8180`) and Klipper linear `process_G1`
/// (`:5195-5202`), retains the parsed decimal double.
pub(super) fn precise_word(code: &str, letter: char) -> Option<f64> {
    let code = code.split([';', '\r', '\n', '\0']).next()?;
    let mut result = None;
    for token in code
        .split([' ', '\t'])
        .filter(|word| !word.is_empty())
        .skip(1)
    {
        let Some(value) = token.strip_prefix(letter) else {
            continue;
        };
        if value.starts_with('+') {
            continue;
        }
        // A numeric prefix followed by another axis is not a word:
        // discard the whole token, then continue at the next word.
        if let Ok(value) = value.parse::<f64>() {
            result = Some(value);
        }
    }
    result
}

pub(super) fn clamped_word(code: &str, letter: char, current: f64, maximum: f64) -> f64 {
    word(code, letter).map_or(current, |value| clamp(value, maximum))
}

/// Parse a `KEY=value` assignment such as Klipper's `SET_VELOCITY_LIMIT
/// ACCEL=500 ACCEL_TO_DECEL=250`.
pub(super) fn assignment(code: &str, key: &str) -> Option<f64> {
    let start = code.find(key)? + key.len();
    let value = &code[start..];
    let end = value
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(value.len());
    value[..end].trim().parse::<f32>().ok().map(f64::from)
}

/// `GCodeProcessor.cpp:41`: the feed-rate conversion multiplies by the f32
/// reciprocal, not by dividing by 60.
pub(super) const MMMIN_TO_MMSEC: f32 = 1.0 / 60.0;

pub(super) fn clamp(value: f64, maximum: f64) -> f64 {
    if maximum > 0.0 {
        value.min(maximum)
    } else {
        value
    }
}

pub(super) fn norm(value: [f64; 4]) -> f64 {
    value
        .iter()
        .map(|component| component * component)
        .sum::<f64>()
        .sqrt()
}
