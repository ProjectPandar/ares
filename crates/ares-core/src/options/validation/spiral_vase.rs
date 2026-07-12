use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;
const SPIRAL_VASE_MESSAGE_PREFIX: &str = "Invalid value when spiral vase mode is enabled";

impl SliceOptions {
    pub fn validate_spiral_vase_cli_options(&self) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();

        if !self.bool_or_default("spiral_mode")? {
            return Ok(errors);
        }

        let wall_loops = self.scalar_i64_or_default("wall_loops")?;
        if wall_loops != 1 {
            errors.insert(
                "wall_loops".to_owned(),
                invalid_spiral_vase_value(wall_loops),
            );
        }

        let sparse_infill_density = self.scalar_f64_or_default("sparse_infill_density")?;
        if sparse_infill_density > 0.0 {
            errors.insert(
                "sparse_infill_density".to_owned(),
                invalid_spiral_vase_float_value(sparse_infill_density),
            );
        }

        let top_shell_layers = self.scalar_i64_or_default("top_shell_layers")?;
        if top_shell_layers > 0 {
            errors.insert(
                "top_shell_layers".to_owned(),
                invalid_spiral_vase_value(top_shell_layers),
            );
        }

        if self.bool_or_default("enable_support")? {
            errors.insert("enable_support".to_owned(), invalid_spiral_vase_value(1));
        }

        let enforce_support_layers = self.scalar_i64_or_default("enforce_support_layers")?;
        if enforce_support_layers > 0 {
            errors.insert(
                "enforce_support_layers".to_owned(),
                invalid_spiral_vase_value(enforce_support_layers),
            );
        }

        Ok(errors)
    }
}

fn invalid_spiral_vase_value(value: impl ToString) -> String {
    format!("{SPIRAL_VASE_MESSAGE_PREFIX}: {}", value.to_string())
}

fn invalid_spiral_vase_float_value(value: f64) -> String {
    format!("{SPIRAL_VASE_MESSAGE_PREFIX}: {value:.6}")
}
