use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

const DEFAULT_FILAMENT_MAX_VOLUMETRIC_SPEED: f64 = 2.0;

pub fn parse_filament_max_volumetric_speed(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    let Some(value) = values.get("filament_max_volumetric_speed") else {
        return Ok(DEFAULT_FILAMENT_MAX_VOLUMETRIC_SPEED);
    };
    let speeds =
        crate::options::parsing::parse_numeric_vector("filament_max_volumetric_speed", value)?;
    if speeds
        .iter()
        .all(|speed| speed.is_finite() && *speed >= 0.0)
    {
        Ok(speeds[0])
    } else {
        Err(SliceError::InvalidInput(
            "filament_max_volumetric_speed contains invalid value".to_owned(),
        ))
    }
}

pub fn parse_filament_adaptive_volumetric_speed(
    values: &BTreeMap<String, Value>,
) -> Result<bool, SliceError> {
    let Some(value) = values.get("filament_adaptive_volumetric_speed") else {
        return Ok(false);
    };
    match value {
        Value::Bool(value) => Ok(*value),
        Value::Array(values) => values.first().and_then(Value::as_bool).ok_or_else(|| {
            SliceError::InvalidInput(
                "filament_adaptive_volumetric_speed must be a boolean or boolean list".to_owned(),
            )
        }),
        _ => Err(SliceError::InvalidInput(
            "filament_adaptive_volumetric_speed must be a boolean or boolean list".to_owned(),
        )),
    }
}

pub fn parse_volumetric_speed_coefficients(values: &BTreeMap<String, Value>) -> Option<[f64; 6]> {
    let value = values.get("volumetric_speed_coefficients")?;
    let text = match value {
        Value::String(text) => text.as_str(),
        Value::Array(values) => values.first()?.as_str()?,
        _ => return None,
    };
    parse_coefficient_text(text)
}

pub fn parse_max_volumetric_extrusion_rate_slope(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    crate::options::parsing::parse_range_f64(
        "max_volumetric_extrusion_rate_slope",
        values.get("max_volumetric_extrusion_rate_slope"),
        0.0,
        0.0,
        f64::INFINITY,
    )
}

pub fn parse_max_volumetric_extrusion_rate_slope_segment_length(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    crate::options::parsing::parse_range_f64(
        "max_volumetric_extrusion_rate_slope_segment_length",
        values.get("max_volumetric_extrusion_rate_slope_segment_length"),
        3.0,
        0.5,
        5.0,
    )
}

pub fn parse_extrusion_rate_smoothing_external_perimeter_only(
    values: &BTreeMap<String, Value>,
) -> Result<bool, SliceError> {
    match values.get("extrusion_rate_smoothing_external_perimeter_only") {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(SliceError::InvalidInput(
            "extrusion_rate_smoothing_external_perimeter_only must be a boolean".to_owned(),
        )),
    }
}

fn parse_coefficient_text(text: &str) -> Option<[f64; 6]> {
    if text.contains('\t') || text.contains('\n') || text.contains('\r') {
        return None;
    }
    let values = text
        .split(' ')
        .filter(|part| !part.is_empty())
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let values: [f64; 6] = values.try_into().ok()?;
    if values.iter().all(|value| value.is_finite()) && values.iter().any(|value| *value != 0.0) {
        Some(values)
    } else {
        None
    }
}
