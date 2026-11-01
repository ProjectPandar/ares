//! Decode `ConfigOption::serialize()`-form strings into typed JSON values.
//!
//! Mirrors `OrcaSlicer/src/libslic3r/Config.cpp` `ConfigOption*::deserialize`
//! element parsing so web consumers can render preset/3MF values into widgets
//! without reimplementing Orca's lexical rules. Nullable vector elements
//! decode `nil` to JSON `null`. Output is a JSON text buffer built directly —
//! value shapes are fixed per kind, so no dynamic `serde_json::Value`
//! intermediate is needed.

use serde::Deserialize;
use serde::de::value::{Error as ValueError, SeqDeserializer, StringDeserializer};

use super::config_types::{FloatOrPercent, Percent, Point2d, Point2dGroups};
use super::model_config_deserialize::wire::{WireValue, decode};
use super::registry::OptionValueKind;

fn scalar<T>(value: &str) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    T::deserialize(StringDeserializer::<ValueError>::new(value.to_owned()))
        .map_err(|error| error.to_string())
}

/// `std::istringstream >> double` semantics: parse the longest valid leading
/// numeric token and ignore the rest (`"0.4;0.6"` decodes to `0.4`, matching
/// how upstream presets carry `;`-joined values through `deserialize`).
/// Returns `(value, extracted)` — since C++11 `[facet.num.get.virtuals]`
/// Stage 3 sets the target to `0` when conversion fails.
fn cxx_float_prefix(value: &str) -> (f64, bool) {
    let value = value.trim();
    let bytes = value.as_bytes();
    let mut end = 0;
    let mut seen_digit = false;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        end = 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
        seen_digit = true;
    }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
            seen_digit = true;
        }
    }
    if seen_digit && end < bytes.len() && matches!(bytes[end], b'e' | b'E') {
        let mut cursor = end + 1;
        if matches!(bytes.get(cursor), Some(b'+' | b'-')) {
            cursor += 1;
        }
        let exponent_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        if cursor > exponent_start {
            end = cursor;
        }
    }
    if !seen_digit {
        return (0.0, false);
    }
    (value[..end].parse::<f64>().unwrap_or(0.0), true)
}

/// `std::istringstream >> int` semantics: leading signed digits only.
/// Returns `(value, extracted)` — on extraction failure C++11
/// `[facet.num.get.virtuals]` Stage 3 writes `0`.
fn cxx_int_prefix(value: &str) -> (i32, bool) {
    let value = value.trim();
    let bytes = value.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let digits = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == digits {
        return (0, false);
    }
    (value[..end].parse::<i32>().unwrap_or(0), true)
}

/// Numeric vector elements (`ConfigOptionFloatsTempl`, `ConfigOptionIntsTempl`,
/// `ConfigOptionPercentsTempl`) ignore `iss.fail()`: upstream pushes whatever
/// `>>` wrote, which since C++11 is `0` on conversion failure.
fn float_element(value: &str) -> f64 {
    cxx_float_prefix(value).0
}

fn int_element(value: &str) -> i32 {
    cxx_int_prefix(value).0
}

fn percent_vector_element(value: &str) -> f64 {
    cxx_float_prefix(value.trim_end_matches('%').trim_end()).0
}

/// Scalar `ConfigOptionSingle` types return `!iss.fail()` — a failed
/// extraction reports `false` instead of silently storing `0`.
fn scalar_float(value: &str) -> Result<f64, String> {
    let (number, extracted) = cxx_float_prefix(value);
    extracted
        .then_some(number)
        .ok_or_else(|| format!("expected Orca float, got {value:?}"))
}

fn scalar_int(value: &str) -> Result<i32, String> {
    let (number, extracted) = cxx_int_prefix(value);
    extracted
        .then_some(number)
        .ok_or_else(|| format!("expected Orca integer, got {value:?}"))
}

/// `ConfigOptionPercent::deserialize` accepts a trailing '%'.
fn scalar_percent(value: &str) -> Result<f64, String> {
    scalar_float(value.trim_end_matches('%').trim_end())
}

fn sequence<T>(elements: &[String]) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    T::deserialize(SeqDeserializer::<_, ValueError>::new(
        elements.iter().cloned(),
    ))
    .map_err(|error| error.to_string())
}

fn number(value: f64) -> String {
    // `Debug` on f64 always renders a fractional part or exponent, so the
    // emitted token reads as a JSON number either way.
    format!("{value:?}")
}

fn point_json(point: Point2d) -> String {
    format!("[{},{}]", number(point.x), number(point.y))
}

fn float_or_percent_json(value: FloatOrPercent) -> String {
    match value {
        FloatOrPercent::Float(value) => {
            format!("{{\"value\":{},\"percent\":false}}", number(value))
        }
        FloatOrPercent::Percent(Percent(value)) => {
            format!("{{\"value\":{},\"percent\":true}}", number(value))
        }
    }
}

fn string_json(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if (character as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => out.push(character),
        }
    }
    out.push('"');
    out
}

fn bool_element(value: &str) -> Result<&'static str, String> {
    // `ConfigOptionBool::deserialize` only accepts "0"/"1", trimming and
    // reading only the first comma-separated element of a scalar value.
    // `ConfigOptionBoolsTempl::deserialize_with_substitutions` applies the
    // same per-element rule; anything else fails, including "".
    let trimmed = value.split(',').next().unwrap_or_default().trim();
    match trimmed {
        "0" => Ok("false"),
        "1" => Ok("true"),
        _ => Err(format!("expected Orca boolean 0 or 1, got {value:?}")),
    }
}

fn join_elements<F>(elements: &[String], convert: F) -> Result<String, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    let mut out = String::from("[");
    for (index, element) in elements.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&convert(element)?);
    }
    out.push(']');
    Ok(out)
}

fn nullable<F>(element: &str, convert: F) -> Result<String, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    if element.trim() == "nil" {
        Ok("null".to_owned())
    } else {
        convert(element)
    }
}

/// Decodes `serialized` (a `ConfigOption::serialize()`-form value from a preset
/// or 3MF) into typed JSON text according to `kind`.
pub fn decode_config_value(kind: OptionValueKind, serialized: &str) -> Result<String, String> {
    let wire = decode(kind, serialized)?;
    match (kind, wire) {
        (OptionValueKind::Bool, WireValue::Scalar(value)) => {
            bool_element(&value).map(str::to_owned)
        }
        (OptionValueKind::Int, WireValue::Scalar(value)) => {
            scalar_int(&value).map(|v| v.to_string())
        }
        (OptionValueKind::Float, WireValue::Scalar(value)) => scalar_float(&value).map(number),
        (OptionValueKind::Percent, WireValue::Scalar(value)) => scalar_percent(&value).map(number),
        (OptionValueKind::FloatOrPercent, WireValue::Scalar(value)) => {
            // `ConfigOptionFloatOrPercent::deserialize` streams a double
            // prefix and flags percent when the whole input contains '%'.
            scalar_float(&value).map(|v| {
                if value.contains('%') {
                    float_or_percent_json(FloatOrPercent::Percent(Percent(v)))
                } else {
                    float_or_percent_json(FloatOrPercent::Float(v))
                }
            })
        }
        (OptionValueKind::Enum, WireValue::Scalar(value))
        | (OptionValueKind::String, WireValue::Scalar(value)) => Ok(string_json(&value)),
        (OptionValueKind::Point, WireValue::Scalar(value)) => {
            scalar::<Point2d>(&value).map(point_json)
        }
        (OptionValueKind::Bools, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                bool_element(element).map(str::to_owned)
            })
        }
        (OptionValueKind::BoolsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| bool_element(e).map(str::to_owned))
            })
        }
        (OptionValueKind::Ints, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| Ok(int_element(element).to_string()))
        }
        (OptionValueKind::IntsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| Ok(int_element(e).to_string()))
            })
        }
        (OptionValueKind::FloatsOrPercents, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                decode_config_value(OptionValueKind::FloatOrPercent, element)
            })
        }
        (OptionValueKind::Floats, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| Ok(number(float_element(element))))
        }
        (OptionValueKind::Percents, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                Ok(number(percent_vector_element(element)))
            })
        }
        (OptionValueKind::FloatsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| Ok(number(float_element(e))))
            })
        }
        (OptionValueKind::PercentsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| Ok(number(percent_vector_element(e))))
            })
        }
        (OptionValueKind::Enums, WireValue::Sequence(elements))
        | (OptionValueKind::Strings, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| Ok(string_json(element)))
        }
        (OptionValueKind::EnumsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| Ok(string_json(e)))
            })
        }
        (OptionValueKind::Points, WireValue::Sequence(elements)) => {
            // `ConfigOptionPoints::deserialize` splits elements on ',',
            // each element on 'x', and extracts coordinates with
            // `istringstream >> double` (missing coordinate -> 0).
            join_elements(&elements, |element| {
                let mut coordinates = element.split('x');
                let x = cxx_float_prefix(coordinates.next().unwrap_or_default()).0;
                let y = match coordinates.next() {
                    Some(rest) => cxx_float_prefix(rest).0,
                    None => 0.0,
                };
                Ok(format!("[{},{}]", number(x), number(y)))
            })
        }
        (OptionValueKind::PointsGroups, WireValue::Sequence(elements)) => {
            sequence::<Point2dGroups>(&elements).map(|groups| {
                let rendered = groups
                    .0
                    .into_iter()
                    .map(|group| {
                        format!(
                            "[{}]",
                            group
                                .into_iter()
                                .map(point_json)
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!("[{rendered}]")
            })
        }
        (kind, wire) => Err(format!("decode shape mismatch for {kind:?}: {wire:?}")),
    }
}

/// Resolves `key` in the option registry and decodes `serialized` into typed
/// JSON text. Unknown keys error.
pub fn decode_config_value_by_key(key: &str, serialized: &str) -> Result<String, String> {
    use super::registry::option_definition;
    let definition =
        option_definition(key).ok_or_else(|| format!("unknown config option key {key:?}"))?;
    decode_config_value(definition.kind, serialized)
}
