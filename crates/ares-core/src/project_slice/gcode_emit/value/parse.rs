use super::Value;

/// The `expr` type an option value takes: `scalar_variable_to_expr`
/// (`PlaceholderParser.cpp:1170-1232`) and `vector_element_to_expr` (`:1355-1361`).
#[derive(Clone, Copy)]
enum Scalar {
    Bool,
    Int,
    /// `coFloat`, `coPercent` and their vectors: the number without its `%`.
    Float,
    /// `coString`, `coEnum` and `coStrings`: the serialized text.
    Text,
    Untyped,
}

pub(super) fn parse_value(
    raw: &str,
    kind: Option<crate::options::registry::OptionValueKind>,
) -> Value {
    use crate::options::registry::OptionValueKind as Kind;
    // Registry-typed vector options keep their vector shape regardless of
    // element count — upstream vectors stay `is_vector()` even with a
    // single element, and a bare brace reference then enters
    // `vector_element_to_expr` (`PlaceholderParser.cpp:1086-1211`) instead
    // of scalar resolution.
    let (scalar, nullable) = match kind {
        Some(Kind::Int) => return parse_scalar(raw, Scalar::Int),
        Some(Kind::Float | Kind::Percent) => return parse_scalar(raw, Scalar::Float),
        Some(Kind::String | Kind::Enum) => return parse_scalar(raw, Scalar::Text),
        Some(Kind::Bool) => return untyped_list(raw, Scalar::Bool),
        Some(Kind::Ints) => (Scalar::Int, false),
        Some(Kind::IntsNullable) => (Scalar::Int, true),
        Some(Kind::Floats | Kind::Percents) => (Scalar::Float, false),
        Some(Kind::FloatsNullable | Kind::PercentsNullable) => (Scalar::Float, true),
        Some(Kind::Bools) => (Scalar::Bool, false),
        Some(Kind::BoolsNullable) => (Scalar::Bool, true),
        Some(Kind::Strings) => (Scalar::Text, false),
        Some(Kind::Enums | Kind::EnumsNullable) => (Scalar::Untyped, false),
        _ => return untyped_list(raw, Scalar::Untyped),
    };
    Value::List(
        raw.split([';', ','])
            .map(|element| {
                if nullable && element.trim() == "nil" {
                    Value::Nil
                } else {
                    parse_scalar(element, scalar)
                }
            })
            .collect(),
    )
}

/// An option without a registry vector kind: `;` or `,` separated values become a list.
fn untyped_list(raw: &str, scalar: Scalar) -> Value {
    let mut values = raw
        .split(';')
        .map(|value| parse_scalar(value, scalar))
        .collect::<Vec<_>>();
    if values.len() == 1 {
        let value = values.remove(0);
        let comma_values = value
            .as_string()
            .split(',')
            .map(|value| parse_scalar(value, scalar))
            .collect::<Vec<_>>();
        return if comma_values.len() == 1 {
            value
        } else {
            Value::List(comma_values)
        };
    }
    Value::List(values)
}

fn parse_scalar(raw: &str, scalar: Scalar) -> Value {
    let raw = raw.trim().trim_matches('"');
    match scalar {
        Scalar::Bool if matches!(raw, "0" | "1" | "false" | "true") => {
            Value::option_bool(matches!(raw, "1" | "true"))
        }
        Scalar::Int => Value::Integer(raw.parse().unwrap()),
        Scalar::Float => raw
            .trim_end_matches('%')
            .parse()
            .map_or_else(|_| untyped(raw), Value::Number),
        Scalar::Text => Value::String(raw.to_owned()),
        Scalar::Bool | Scalar::Untyped => untyped(raw),
    }
}

fn untyped(raw: &str) -> Value {
    if let Ok(value) = raw.parse::<f64>() {
        Value::Number(value)
    } else if raw == "true" || raw == "false" {
        Value::Bool(raw == "true")
    } else {
        Value::String(raw.to_owned())
    }
}
