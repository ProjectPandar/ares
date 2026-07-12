use serde_json::Value;

use super::SliceOptions;

use crate::SliceError;

const KEY: &str = "filament_cooling_before_tower";
const DEFAULT_VALUE: NullableFloat = NullableFloat::Value(10.0);

#[derive(Clone, Copy, Debug, PartialEq)]
enum NullableFloat {
    Value(f64),
    Nil,
}

impl SliceOptions {
    #[cfg(test)]
    pub(crate) fn filament_cooling_before_tower(&self) -> Result<Vec<f64>, SliceError> {
        self.filament_cooling_before_tower_values()?
            .into_iter()
            .map(|value| match value {
                NullableFloat::Value(value) => Ok(value),
                NullableFloat::Nil => Err(SliceError::InvalidInput(format!(
                    "{KEY} contains invalid value"
                ))),
            })
            .collect()
    }

    pub(crate) fn filament_cooling_before_tower_placeholder(&self) -> Result<String, SliceError> {
        Ok(format_nullable_floats(
            &self.filament_cooling_before_tower_values()?,
        ))
    }

    #[cfg(test)]
    pub(crate) fn filament_cooling_before_tower_config_export(
        &self,
    ) -> Result<Option<String>, SliceError> {
        filament_cooling_before_tower_config_export(self.values().get(KEY))
    }

    fn filament_cooling_before_tower_values(&self) -> Result<Vec<NullableFloat>, SliceError> {
        parse_nullable_float_vector(self.values().get(KEY))
    }
}

pub(crate) fn filament_cooling_before_tower_config_export(
    value: Option<&Value>,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    if matches!(value, Value::Array(values) if values.is_empty()) {
        return Ok(Some(String::new()));
    }
    let values = parse_nullable_float_vector(Some(value))?;
    if values.iter().all(|value| matches!(value, NullableFloat::Nil)) {
        Ok(None)
    } else {
        Ok(Some(format_nullable_floats(&values)))
    }
}

fn parse_nullable_float_vector(value: Option<&Value>) -> Result<Vec<NullableFloat>, SliceError> {
    let Some(value) = value else {
        return Ok(vec![DEFAULT_VALUE]);
    };
    match value {
        Value::Null => Ok(vec![NullableFloat::Nil]),
        Value::Number(number) => parse_number(number.as_f64()),
        Value::String(text) => parse_text(text),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(SliceError::InvalidInput(format!("{KEY} must not be empty")));
            }
            values.iter().map(parse_value).collect()
        }
        _ => Err(SliceError::InvalidInput(format!(
            "{KEY} must be a number or numeric list"
        ))),
    }
}

fn parse_value(value: &Value) -> Result<NullableFloat, SliceError> {
    match value {
        Value::Null => Ok(NullableFloat::Nil),
        Value::Number(number) => parse_single_number(number.as_f64()),
        Value::String(text) => parse_single_text(text),
        _ => Err(SliceError::InvalidInput(format!(
            "{KEY} must contain only numeric values or nil"
        ))),
    }
}

fn parse_text(text: &str) -> Result<Vec<NullableFloat>, SliceError> {
    let parts = text.split([',', ';']).map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(SliceError::InvalidInput(format!("{KEY} must not be empty")));
    }
    parts.into_iter().map(parse_single_text).collect()
}

fn parse_single_text(text: &str) -> Result<NullableFloat, SliceError> {
    if text == "nil" {
        Ok(NullableFloat::Nil)
    } else {
        parse_single_number(text.parse::<f64>().ok())
    }
}

fn parse_number(value: Option<f64>) -> Result<Vec<NullableFloat>, SliceError> {
    parse_single_number(value).map(|value| vec![value])
}

fn parse_single_number(value: Option<f64>) -> Result<NullableFloat, SliceError> {
    match value {
        Some(value) if value.is_finite() && value >= 0.0 => Ok(NullableFloat::Value(value)),
        _ => Err(SliceError::InvalidInput(format!(
            "{KEY} contains invalid value"
        ))),
    }
}

fn format_nullable_floats(values: &[NullableFloat]) -> String {
    values
        .iter()
        .map(|value| match value {
            NullableFloat::Value(value) => crate::gcode_format::format_decimal(*value),
            NullableFloat::Nil => "nil".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(",")
}
