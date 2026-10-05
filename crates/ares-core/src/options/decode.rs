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
/// Fails only when no numeric prefix exists, mirroring `iss.fail()`.
fn cxx_float(value: &str) -> Result<f64, String> {
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
        return Err(format!("expected Orca float prefix, got {value:?}"));
    }
    value[..end]
        .parse::<f64>()
        .map_err(|error| format!("invalid Orca float {value:?}: {error}"))
}

/// `std::istringstream >> int` semantics: leading signed digits only.
fn cxx_int(value: &str) -> Result<i32, String> {
    let value = value.trim();
    let bytes = value.as_bytes();
    let mut end = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let digits = end;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == digits {
        return Err(format!("expected Orca integer prefix, got {value:?}"));
    }
    value[..end]
        .parse::<i32>()
        .map_err(|error| format!("invalid Orca int {value:?}: {error}"))
}

/// `ConfigOptionPercent::deserialize` accepts a trailing '%'.
fn percent_element(value: &str) -> Result<f64, String> {
    cxx_float(value.trim_end_matches('%').trim_end())
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
        (OptionValueKind::Int, WireValue::Scalar(value)) => cxx_int(&value).map(|v| v.to_string()),
        (OptionValueKind::Float, WireValue::Scalar(value)) => cxx_float(&value).map(number),
        (OptionValueKind::Percent, WireValue::Scalar(value)) => percent_element(&value).map(number),
        (OptionValueKind::FloatOrPercent, WireValue::Scalar(value)) => {
            // `ConfigOptionFloatOrPercent::deserialize` streams a double
            // prefix and flags percent when the whole input contains '%'.
            cxx_float(&value).map(|v| {
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
            join_elements(&elements, |element| cxx_int(element).map(|v| v.to_string()))
        }
        (OptionValueKind::IntsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| cxx_int(e).map(|v| v.to_string()))
            })
        }
        (OptionValueKind::Floats, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| cxx_float(element).map(number))
        }
        (OptionValueKind::Percents, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| percent_element(element).map(number))
        }
        (OptionValueKind::FloatsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| cxx_float(e).map(number))
            })
        }
        (OptionValueKind::PercentsNullable, WireValue::Sequence(elements)) => {
            join_elements(&elements, |element| {
                nullable(element, |e| percent_element(e).map(number))
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
                let x = cxx_float(coordinates.next().unwrap_or_default())?;
                let y = match coordinates.next() {
                    Some(rest) => cxx_float(rest)?,
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn decode_by_kind(kind: OptionValueKind, serialized: &str) -> Value {
        serde_json::from_str(&decode_config_value(kind, serialized).unwrap()).unwrap()
    }

    #[test]
    fn scalar_decoding() {
        assert_eq!(decode_by_kind(OptionValueKind::Bool, "1"), json!(true));
        assert_eq!(decode_by_kind(OptionValueKind::Bool, "0"), json!(false));
        assert!(decode_config_value(OptionValueKind::Bool, "true").is_err());
        assert!(decode_config_value(OptionValueKind::Bool, "false").is_err());
        assert!(decode_config_value(OptionValueKind::Bool, "").is_err());
        assert!(decode_config_value(OptionValueKind::Bools, "1,,0").is_err());
        assert!(decode_config_value(OptionValueKind::Bools, "1,true").is_err());
        assert!(decode_config_value(OptionValueKind::Bools, "nil").is_err());
        assert_eq!(decode_by_kind(OptionValueKind::Int, "42"), json!(42));
        assert_eq!(decode_by_kind(OptionValueKind::Float, "0.4"), json!(0.4));
        assert_eq!(decode_by_kind(OptionValueKind::Percent, "50%"), json!(50.0));
        assert_eq!(decode_by_kind(OptionValueKind::Percent, "50"), json!(50.0));
        assert_eq!(
            decode_by_kind(OptionValueKind::FloatOrPercent, "12%"),
            json!({ "value": 12.0, "percent": true })
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::FloatOrPercent, "12"),
            json!({ "value": 12.0, "percent": false })
        );
        // Upstream bool scalar takes the first comma element.
        assert_eq!(decode_by_kind(OptionValueKind::Bool, "1,0,1"), json!(true));
    }

    #[test]
    fn vector_decoding() {
        assert_eq!(
            decode_by_kind(OptionValueKind::Bools, "1,0,1"),
            json!([true, false, true])
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::Ints, "1, 2,3"),
            json!([1, 2, 3])
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::Floats, "0.2,0.4"),
            json!([0.2, 0.4])
        );
        // Upstream empty Floats decode yields [0].
        assert_eq!(decode_by_kind(OptionValueKind::Floats, ""), json!([0.0]));
        assert_eq!(
            decode_by_kind(OptionValueKind::Percents, "10%,20"),
            json!([10.0, 20.0])
        );
    }

    #[test]
    fn nullable_vectors_map_nil_to_null() {
        assert_eq!(
            decode_by_kind(OptionValueKind::IntsNullable, "1,nil,3"),
            json!([1, null, 3])
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::FloatsNullable, "nil,0.5"),
            json!([null, 0.5])
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::BoolsNullable, "1,nil"),
            json!([true, null])
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::EnumsNullable, "nil,arc"),
            json!([null, "arc"])
        );
    }

    #[test]
    fn strings_unescape_and_split() {
        assert_eq!(
            decode_by_kind(OptionValueKind::String, "pla\\;b\\n"),
            json!("pla;b\n")
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::Strings, "\"PLA;a\";PETG"),
            json!(["PLA;a", "PETG"])
        );
        assert_eq!(decode_by_kind(OptionValueKind::Strings, ""), json!([]));
        // Upstream unescape keeps the escaped char verbatim (`\c` -> `c`),
        // and the decoded quote is escaped again in output JSON text.
        assert_eq!(
            decode_by_kind(OptionValueKind::String, "a\"b\\c"),
            json!("a\"bc")
        );
    }

    #[test]
    fn points_decode_to_pairs() {
        assert_eq!(
            decode_by_kind(OptionValueKind::Point, "100x100"),
            json!([100.0, 100.0])
        );
        assert_eq!(
            decode_by_kind(OptionValueKind::Points, "0x0,200x0"),
            json!([[0.0, 0.0], [200.0, 0.0]])
        );
    }

    #[test]
    fn key_based_decode_uses_registry() {
        // layer_height is a Float option.
        assert_eq!(
            decode_config_value_by_key("layer_height", "0.2").unwrap(),
            "0.2"
        );
        assert!(decode_config_value_by_key("definitely_not_a_key", "1").is_err());
    }
}
