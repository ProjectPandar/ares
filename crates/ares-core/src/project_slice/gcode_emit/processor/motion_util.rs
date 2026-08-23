//! G-code word parsing and kinematic clamp helpers shared by motion planning.

/// Parse a G-code word (letter + numeric value) from a command string.
pub(super) fn word(code: &str, letter: char) -> Option<f64> {
    let start = code.find(letter)? + letter.len_utf8();
    let value = &code[start..];
    let end = value
        .find(|character: char| character.is_ascii_alphabetic())
        .unwrap_or(value.len());
    value[..end].trim().parse().ok()
}

pub(super) fn clamped_word(code: &str, letter: char, current: f64, maximum: f64) -> f64 {
    word(code, letter).map_or(current, |value| clamp(value, maximum))
}

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

pub(super) fn scale(value: [f64; 4], factor: f64) -> [f64; 4] {
    [
        value[0] * factor,
        value[1] * factor,
        value[2] * factor,
        value[3] * factor,
    ]
}
