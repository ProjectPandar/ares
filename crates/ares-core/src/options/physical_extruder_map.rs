use serde_json::Value;

use crate::{SliceError, SliceOptions};

const KEY: &str = "physical_extruder_map";

impl SliceOptions {
    pub(crate) fn validate_slicing_physical_extruder_map(&self) -> Result<(), SliceError> {
        if self.values().contains_key(KEY) {
            let _ = self.physical_extruder_id_for_logical(0)?;
        }
        Ok(())
    }

    pub(crate) fn physical_extruder_id_for_logical(
        &self,
        logical_extruder_id: usize,
    ) -> Result<u32, SliceError> {
        let Some(value) = self.values().get(KEY) else {
            return Ok(0);
        };
        let values = parse_map(value)?;
        let index = logical_extruder_id.min(values.len() - 1);
        Ok(values[index])
    }
}

fn parse_map(value: &Value) -> Result<Vec<u32>, SliceError> {
    match value {
        Value::Array(values) => {
            if values.is_empty() {
                return Err(invalid());
            }
            values.iter().map(parse_value).collect()
        }
        _ => Ok(vec![parse_value(value)?]),
    }
}

fn parse_value(value: &Value) -> Result<u32, SliceError> {
    match value {
        Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
    .ok_or_else(invalid)
}

fn invalid() -> SliceError {
    SliceError::InvalidInput(format!(
        "{KEY} must be a non-empty list of non-negative integers"
    ))
}
