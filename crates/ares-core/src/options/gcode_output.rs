use super::SliceOptions;
use crate::SliceError;

impl SliceOptions {
    #[rustfmt::skip]
    pub(crate) fn gcode_comments(&self) -> Result<bool, SliceError> { self.bool_option("gcode_comments", false) }
    #[rustfmt::skip]
    pub(crate) fn wipe_before_external_loop(&self) -> Result<bool, SliceError> { self.bool_option("wipe_before_external_loop", false) }
    #[rustfmt::skip]
    pub(crate) fn wipe_on_loops(&self) -> Result<bool, SliceError> { self.bool_option("wipe_on_loops", false) }
    #[rustfmt::skip]
    pub(crate) fn disable_m73(&self) -> Result<bool, SliceError> { self.bool_option("disable_m73", false) }
    #[rustfmt::skip]
    pub(crate) fn silent_mode(&self) -> Result<bool, SliceError> { self.bool_option("silent_mode", false) }

    pub(crate) fn resolution(&self) -> Result<f64, SliceError> {
        let Some(value) = self.values().get("resolution") else {
            return Ok(0.01);
        };
        let value = match value {
            serde_json::Value::Number(number) => number.as_f64(),
            serde_json::Value::String(text) => text.parse().ok(),
            _ => None,
        }
        .ok_or_else(|| SliceError::InvalidInput("resolution must be a number".to_owned()))?;
        if value.is_finite() && value >= 0.0 {
            Ok(value.max(0.001))
        } else {
            Err(SliceError::InvalidInput(
                "resolution must be non-negative".to_owned(),
            ))
        }
    }
}
