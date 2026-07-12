use std::collections::BTreeMap;

use serde_json::Value;

use crate::SliceError;

use super::parsing;

pub(crate) fn parse_small_perimeter_threshold(
    values: &BTreeMap<String, Value>,
) -> Result<f64, SliceError> {
    parsing::parse_range_f64(
        "small_perimeter_threshold",
        values.get("small_perimeter_threshold"),
        0.0,
        0.0,
        f64::INFINITY,
    )
}

pub(crate) fn parse_small_perimeter_speed(
    values: &BTreeMap<String, Value>,
    outer_wall_speed_mm_s: f64,
) -> Result<f64, SliceError> {
    let speed = match values.get("small_perimeter_speed") {
        Some(value) => parsing::parse_non_negative_numeric_or_percent_over_base(
            "small_perimeter_speed",
            value,
            outer_wall_speed_mm_s,
        )?,
        None => outer_wall_speed_mm_s * 0.5,
    };
    if speed == 0.0 {
        Ok(outer_wall_speed_mm_s * 0.5)
    } else {
        Ok(speed)
    }
}
