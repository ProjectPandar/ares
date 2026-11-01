//! Config.hpp ConfigOption numeric/enum vectors: comma join and type-owned nil.
use serde::{
    Deserialize,
    de::value::{Error, StrDeserializer},
};

use super::RawConfigValue;
use crate::options::{FloatOrPercent, Percent, format_number, registry::OptionValueKind as Kind};

pub(super) fn is_vector(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Floats
            | Kind::FloatsNullable
            | Kind::FloatsOrPercents
            | Kind::Ints
            | Kind::IntsNullable
            | Kind::Bools
            | Kind::BoolsNullable
            | Kind::Percents
            | Kind::PercentsNullable
            | Kind::Enums
            | Kind::EnumsNullable
    )
}

pub(super) fn token(
    values: &[RawConfigValue],
    kind: Kind,
    key: &str,
) -> Result<Option<String>, String> {
    let nullable = matches!(
        kind,
        Kind::FloatsNullable
            | Kind::IntsNullable
            | Kind::BoolsNullable
            | Kind::PercentsNullable
            | Kind::EnumsNullable
    );
    let rendered = values
        .iter()
        .map(|value| element(value, kind, key, nullable))
        .collect::<Result<Vec<_>, _>>()?;
    let nil = match kind {
        Kind::Enums => false,
        Kind::EnumsNullable => rendered.iter().all(|value| value == "nil"),
        Kind::Ints | Kind::IntsNullable => rendered
            .iter()
            .all(|value| value == "nil" || value == "2147483647"),
        _ => rendered.iter().all(|value| value == "nil"),
    };
    Ok((!nil).then(|| rendered.join(",")))
}

fn element(
    value: &RawConfigValue,
    kind: Kind,
    key: &str,
    nullable: bool,
) -> Result<String, String> {
    let text = match value {
        RawConfigValue::Text(text) => text.trim().to_owned(),
        RawConfigValue::Number(number) => number.to_string(),
        RawConfigValue::Flag(flag) => u8::from(*flag).to_string(),
        _ => {
            return Err(format!(
                "invalid Orca option {key}: expected vector element"
            ));
        }
    };
    if nullable && text == "nil" {
        return Ok(text);
    }
    let invalid = || format!("invalid Orca option {key}: invalid {kind:?} element {text:?}");
    match kind {
        Kind::Floats | Kind::FloatsNullable => text
            .parse::<f64>()
            .map(format_number)
            .map_err(|_| invalid()),
        Kind::Ints | Kind::IntsNullable => text
            .parse::<i32>()
            .map(|value| value.to_string())
            .map_err(|_| invalid()),
        Kind::Percents | Kind::PercentsNullable => text
            .trim_end_matches('%')
            .parse::<f64>()
            .map(|value| format!("{}%", format_number(value)))
            .map_err(|_| invalid()),
        Kind::Bools | Kind::BoolsNullable => match text.as_str() {
            "0" | "1" => Ok(text),
            _ => Err(invalid()),
        },
        Kind::FloatsOrPercents => {
            let value = FloatOrPercent::deserialize(StrDeserializer::<Error>::new(&text))
                .map_err(|_| invalid())?;
            Ok(match value {
                FloatOrPercent::Float(value) => format_number(value),
                FloatOrPercent::Percent(Percent(value)) => format!("{}%", format_number(value)),
            })
        }
        Kind::Enums | Kind::EnumsNullable => Ok(text),
        _ => unreachable!("numeric vector kind"),
    }
}
