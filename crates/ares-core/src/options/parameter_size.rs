use serde_json::Value;

use super::{SliceOptions, registry};
use crate::SliceError;

impl SliceOptions {
    pub fn parameter_size(
        &self,
        param_name: &str,
        extruder_nums: usize,
    ) -> Result<usize, SliceError> {
        let filament_variant_length = self
            .optional_string_array_len("filament_extruder_variant")?
            .unwrap_or(1);
        let process_variant_length = self
            .optional_string_array_len("print_extruder_variant")?
            .unwrap_or(1);
        let machine_variant_length = self
            .optional_string_array_len("printer_extruder_variant")?
            .unwrap_or(1);

        if registry::printer_options_with_variant_1().contains(&param_name) {
            Ok(machine_variant_length)
        } else if registry::printer_options_with_variant_2().contains(&param_name) {
            Ok(machine_variant_length * 2)
        } else if registry::filament_options_with_variant().contains(&param_name) {
            Ok(filament_variant_length)
        } else if registry::print_options_with_variant().contains(&param_name) {
            Ok(process_variant_length)
        } else {
            Ok(extruder_nums)
        }
    }

    fn optional_string_array_len(&self, key: &str) -> Result<Option<usize>, SliceError> {
        let Some(value) = self.values().get(key) else {
            return Ok(None);
        };
        let Value::Array(values) = value else {
            return Err(SliceError::InvalidInput(format!(
                "{key} must be a string array"
            )));
        };
        if values.iter().all(Value::is_string) {
            Ok(Some(values.len()))
        } else {
            Err(SliceError::InvalidInput(format!(
                "{key} must be a string array"
            )))
        }
    }
}
