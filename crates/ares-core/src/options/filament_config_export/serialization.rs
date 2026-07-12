use serde_json::Value;

use crate::{SliceError, gcode_format::format_decimal};

pub(super) fn optional_string_vector_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?;
    let strings = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(serialize_config_option_strings(&strings)))
}

pub(super) fn optional_small_area_flow_model_export(
    value: Option<&Value>,
) -> Result<Option<String>, SliceError> {
    let Some(_) = value else { return Ok(None) };
    let strings = crate::options::small_area_infill_flow::model_entries(value)?;
    crate::extrusions::SmallAreaInfillFlowCompensation::parse(
        strings.clone(),
        false,
        false,
        false,
    )?;
    let refs = strings.iter().map(String::as_str).collect::<Vec<_>>();
    Ok(Some(serialize_config_option_strings(&refs)))
}

pub(super) fn optional_bool_vector_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool vector")))?;
    let bools = values
        .iter()
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain bools")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(serialize_config_option_bools(&bools)))
}

pub(super) fn optional_int_vector_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer vector")))?;
    let ints = values
        .iter()
        .map(|value| {
            let Some(number) = value.as_i64() else {
                return Err(SliceError::InvalidInput(format!(
                    "{key} must contain integers"
                )));
            };
            i32::try_from(number)
                .map_err(|_| SliceError::InvalidInput(format!("{key} must contain i32 integers")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(serialize_config_option_ints(&ints)))
}

pub(super) fn optional_int_vector_export_in_range(
    value: Option<&Value>,
    key: &str,
    min: i32,
    max: i32,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer vector")))?;
    let ints = values
        .iter()
        .map(|value| {
            let Some(number) = value.as_i64() else {
                return Err(SliceError::InvalidInput(format!(
                    "{key} must contain integers"
                )));
            };
            i32::try_from(number)
                .map_err(|_| SliceError::InvalidInput(format!("{key} must contain i32 integers")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if ints.iter().all(|value| (min..=max).contains(value)) {
        Ok(Some(serialize_config_option_ints(&ints)))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

pub(super) fn optional_float_vector_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a float vector")))?;
    let floats = values
        .iter()
        .map(|value| {
            value
                .as_f64()
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain numbers")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if floats
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
    {
        Ok(Some(serialize_config_option_floats(&floats)))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

pub(super) fn optional_wipe_tower_coordinate_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a float vector")))?;
    let Some(selected) = values.first() else {
        return Err(SliceError::InvalidInput(format!(
            "{key} must contain at least one value"
        )));
    };
    let selected = selected
        .as_f64()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain numbers")))?;
    if selected.is_finite()
        && values
            .iter()
            .skip(1)
            .all(|value| value.as_f64().is_some_and(f64::is_finite))
    {
        Ok(Some(format!("{selected:.3}")))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

pub(super) fn optional_scalar_float_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    optional_scalar_float_export_with_min(value, key, None)
}

pub(super) fn optional_non_negative_scalar_float_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    optional_scalar_float_export_with_min(value, key, Some(0.0))
}

pub(super) fn optional_scalar_float_export_with_bounds(
    value: Option<&Value>,
    key: &str,
    min: Option<f64>,
    max: Option<f64>,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let value = value
        .as_f64()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite()
        && min.is_none_or(|min| value >= min)
        && max.is_none_or(|max| value <= max)
    {
        Ok(Some(format_decimal(value)))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

fn optional_scalar_float_export_with_min(
    value: Option<&Value>,
    key: &str,
    min: Option<f64>,
) -> Result<Option<String>, SliceError> {
    optional_scalar_float_export_with_bounds(value, key, min, None)
}

pub(super) fn optional_filament_cooling_before_tower_export(
    value: Option<&Value>,
) -> Result<Option<String>, SliceError> {
    crate::options::filament_cooling_before_tower::filament_cooling_before_tower_config_export(value)
}

pub(super) fn optional_wipe_tower_type_export(
    value: Option<&Value>,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "wipe_tower_type must be an enum string".to_owned(),
        ));
    };
    match value {
        "type1" | "type2" => Ok(Some(value.to_owned())),
        _ => Err(SliceError::InvalidInput(
            "wipe_tower_type contains invalid value".to_owned(),
        )),
    }
}

pub(super) fn optional_wipe_tower_wall_type_export(
    value: Option<&Value>,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "wipe_tower_wall_type must be an enum string".to_owned(),
        ));
    };
    match value {
        "rectangle" | "cone" | "rib" => Ok(Some(value.to_owned())),
        _ => Err(SliceError::InvalidInput(
            "wipe_tower_wall_type contains invalid value".to_owned(),
        )),
    }
}

pub(super) fn optional_scalar_bool_export(
    value: Option<&Value>,
    key: &str,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    value
        .as_bool()
        .map(|value| {
            Some(if value {
                "1".to_owned()
            } else {
                "0".to_owned()
            })
        })
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a boolean")))
}

pub(super) fn optional_scalar_int_export_in_range(
    value: Option<&Value>,
    key: &str,
    min: i32,
    max: i32,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let number = value
        .as_i64()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an integer")))?;
    let number = i32::try_from(number)
        .map_err(|_| SliceError::InvalidInput(format!("{key} must be an i32 integer")))?;
    if (min..=max).contains(&number) {
        Ok(Some(number.to_string()))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}

fn serialize_config_option_strings(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| serialize_config_option_string(value, values.len()))
        .collect::<Vec<_>>()
        .join(";")
}

fn serialize_config_option_bools(values: &[bool]) -> String {
    values
        .iter()
        .map(|value| if *value { "1" } else { "0" })
        .collect::<Vec<_>>()
        .join(",")
}

fn serialize_config_option_ints(values: &[i32]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn serialize_config_option_floats(values: &[f64]) -> String {
    values
        .iter()
        .map(|value| format_decimal(*value))
        .collect::<Vec<_>>()
        .join(",")
}

fn serialize_config_option_string(value: &str, value_count: usize) -> String {
    let should_quote = (value_count == 1 && value.is_empty())
        || value
            .chars()
            .any(|ch| matches!(ch, ' ' | '\t' | '\\' | '"' | '\r' | '\n'));
    if !should_quote {
        return value.to_owned();
    }

    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for ch in value.chars() {
        match ch {
            '\\' | '"' => {
                output.push('\\');
                output.push(ch);
            }
            '\r' => output.push_str("\\r"),
            '\n' => output.push_str("\\n"),
            _ => output.push(ch),
        }
    }
    output.push('"');
    output
}
