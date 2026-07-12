use serde_json::Value;

use super::SliceOptions;
use crate::SliceError;

const MIN_OBJECT_DISTANCE: f64 = 6.0;

impl SliceOptions {
    pub(crate) fn validate_slicing_print_sequence(&self) -> Result<(), SliceError> {
        let _ = self.min_object_distance()?;

        if matches!(
            optional_string(&self.values, "print_sequence")?,
            Some("by object")
        ) {
            return Err(SliceError::InvalidInput(
                "print_sequence by object is not supported by Ares slicing yet".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn min_object_distance(&self) -> Result<f64, SliceError> {
        if matches!(
            optional_string(&self.values, "printer_technology")?,
            Some("SLA")
        ) {
            return Ok(MIN_OBJECT_DISTANCE);
        }

        let print_sequence = optional_string(&self.values, "print_sequence")?;
        let clearance_radius =
            optional_non_negative_f64(&self.values, "extruder_clearance_radius")?;
        let (Some(clearance_radius), Some(print_sequence)) = (clearance_radius, print_sequence)
        else {
            return Ok(0.0);
        };

        match print_sequence {
            "by object" => Ok(clearance_radius.max(MIN_OBJECT_DISTANCE)),
            "by layer" | "by default" => Ok(MIN_OBJECT_DISTANCE),
            _ => Err(SliceError::InvalidInput(
                "print_sequence must be a supported Orca print sequence".to_owned(),
            )),
        }
    }
}

fn optional_string<'a>(
    values: &'a std::collections::BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(None);
    };
    value
        .as_str()
        .map(Some)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string")))
}

fn optional_non_negative_f64(
    values: &std::collections::BTreeMap<String, Value>,
    key: &str,
) -> Result<Option<f64>, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(None);
    };
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a number")))?;
    if value.is_finite() && value >= 0.0 {
        Ok(Some(value))
    } else {
        Err(SliceError::InvalidInput(format!(
            "{key} contains invalid value"
        )))
    }
}
