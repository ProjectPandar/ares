use std::collections::BTreeMap;

use serde_json::Value;

use crate::{OverhangSpeedBands, SliceError};

pub(crate) fn parse_overhang_perimeter_speed(
    values: &BTreeMap<String, Value>,
    outer_wall_speed: f64,
    bridge_speed: f64,
) -> Result<f64, SliceError> {
    let enable = match values.get("enable_overhang_speed") {
        Some(value) => value.as_bool().ok_or_else(|| {
            SliceError::InvalidInput("enable_overhang_speed must be a boolean".to_owned())
        })?,
        None => true,
    };
    let speed = match values.get("overhang_4_4_speed") {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            "overhang_4_4_speed",
            value,
            outer_wall_speed,
        )?,
        None => 0.0,
    };
    if enable && speed > 0.0 {
        Ok(speed)
    } else {
        Ok(bridge_speed)
    }
}

pub(crate) fn parse_overhang_speed_bands(
    values: &BTreeMap<String, Value>,
    external_line_width_mm: f64,
    outer_wall_speed: f64,
    bridge_speed: f64,
) -> Result<OverhangSpeedBands, SliceError> {
    let enable = match values.get("enable_overhang_speed") {
        Some(value) => value.as_bool().ok_or_else(|| {
            SliceError::InvalidInput("enable_overhang_speed must be a boolean".to_owned())
        })?,
        None => true,
    };
    let slowdown_for_curled_perimeters = match values.get("slowdown_for_curled_perimeters") {
        Some(value) => value.as_bool().ok_or_else(|| {
            SliceError::InvalidInput("slowdown_for_curled_perimeters must be a boolean".to_owned())
        })?,
        None => true,
    };
    if !enable {
        return Ok(OverhangSpeedBands::disabled(external_line_width_mm));
    }
    let overhang_4_4_speed = parse_band(values, "overhang_4_4_speed", outer_wall_speed)?;
    let final_severe_speed_mm_s = if slowdown_for_curled_perimeters {
        overhang_4_4_speed
    } else {
        Some(bridge_speed)
    };

    Ok(OverhangSpeedBands::new(
        external_line_width_mm,
        [
            parse_band(values, "overhang_1_4_speed", outer_wall_speed)?,
            parse_band(values, "overhang_2_4_speed", outer_wall_speed)?,
            parse_band(values, "overhang_3_4_speed", outer_wall_speed)?,
            overhang_4_4_speed,
        ],
        final_severe_speed_mm_s,
    ))
}

fn parse_band(
    values: &BTreeMap<String, Value>,
    key: &str,
    outer_wall_speed: f64,
) -> Result<Option<f64>, SliceError> {
    values
        .get(key)
        .map(|value| {
            crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
                key,
                value,
                outer_wall_speed,
            )
            .map(Some)
        })
        .unwrap_or(Ok(None))
}
