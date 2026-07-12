use serde_json::Value;

use crate::SliceError;

const DEFAULT_PELLET_FLOW_COEFFICIENT: &[f64] = &[0.4157];
const DEFAULT_FILAMENT_SHRINKAGE_COMPENSATION_Z: f64 = 100.0;

pub(super) fn effective_filament_diameters(
    options: &super::SliceOptions,
) -> Result<Vec<f64>, SliceError> {
    if !options.bool_option("pellet_modded_printer", false)? {
        return options.numeric_vector(
            "filament_diameter",
            super::DEFAULT_FILAMENT_DIAMETERS,
            |value| value >= 1.0,
        );
    }

    let coefficients = options.numeric_vector(
        "pellet_flow_coefficient",
        DEFAULT_PELLET_FLOW_COEFFICIENT,
        |value| value > 0.0,
    )?;

    Ok(coefficients
        .into_iter()
        .map(pellet_flow_to_filament_diameter)
        .collect())
}

fn pellet_flow_to_filament_diameter(coefficient: f64) -> f64 {
    (4.0 / (std::f64::consts::PI * coefficient)).sqrt()
}

impl super::SliceOptions {
    pub(crate) fn filament_shrinkage_compensation_z(&self) -> Result<f64, SliceError> {
        let Some(value) = self.values.get("filament_shrinkage_compensation_z") else {
            return Ok(1.0);
        };
        let percentages =
            parse_filament_shrinkage_percent_vector("filament_shrinkage_compensation_z", value)?;
        Ok(DEFAULT_FILAMENT_SHRINKAGE_COMPENSATION_Z / percentages[0])
    }

    pub(crate) fn filament_shrink_xy(&self) -> Result<f64, SliceError> {
        let Some(value) = self.values.get("filament_shrink") else {
            return Ok(1.0);
        };
        let percentages = parse_filament_shrinkage_percent_vector("filament_shrink", value)?;
        Ok(DEFAULT_FILAMENT_SHRINKAGE_COMPENSATION_Z / percentages[0])
    }
}

fn parse_filament_shrinkage_percent_vector(
    key: &'static str,
    value: &Value,
) -> Result<Vec<f64>, SliceError> {
    let percentages = match value {
        Value::Number(number) => number
            .as_f64()
            .map(|value| vec![value])
            .ok_or_else(|| invalid_filament_shrinkage_percent(key))?,
        Value::String(text) => parse_filament_shrinkage_percent_text(key, text)?,
        Value::Array(values) => {
            if values.is_empty() {
                return Err(invalid_filament_shrinkage_percent(key));
            }
            values
                .iter()
                .map(|value| parse_filament_shrinkage_percent_value(key, value))
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => return Err(invalid_filament_shrinkage_percent(key)),
    };
    if percentages
        .iter()
        .all(|value| value.is_finite() && (50.0..=150.0).contains(value))
    {
        Ok(percentages)
    } else {
        Err(invalid_filament_shrinkage_percent(key))
    }
}

fn parse_filament_shrinkage_percent_text(
    key: &'static str,
    text: &str,
) -> Result<Vec<f64>, SliceError> {
    let parts = text.split([';', ',']).map(str::trim).collect::<Vec<_>>();
    if parts.iter().any(|part| part.is_empty()) {
        return Err(invalid_filament_shrinkage_percent(key));
    }
    parts
        .into_iter()
        .map(|part| parse_filament_shrinkage_percent_part(key, part))
        .collect()
}

fn parse_filament_shrinkage_percent_value(
    key: &'static str,
    value: &Value,
) -> Result<f64, SliceError> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .ok_or_else(|| invalid_filament_shrinkage_percent(key)),
        Value::String(text) => parse_filament_shrinkage_percent_part(key, text),
        _ => Err(invalid_filament_shrinkage_percent(key)),
    }
}

fn parse_filament_shrinkage_percent_part(key: &'static str, text: &str) -> Result<f64, SliceError> {
    text.trim()
        .strip_suffix('%')
        .unwrap_or(text.trim())
        .trim()
        .parse::<f64>()
        .map_err(|_| invalid_filament_shrinkage_percent(key))
}

fn invalid_filament_shrinkage_percent(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} must contain finite percentages in 50..=150"))
}
