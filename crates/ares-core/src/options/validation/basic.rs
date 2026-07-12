use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;
use super::helpers::{SCALING_FACTOR, invalid_value_message, serialize_numbers};

impl SliceOptions {
    pub fn validate_basic_fdm_options(&self) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();

        let layer_height = self.scalar_f64_or_default("layer_height")?;
        if layer_height <= 0.0 || layer_height.rem_euclid(SCALING_FACTOR).abs() > 1e-4 {
            errors.insert(
                "layer_height".to_owned(),
                invalid_value_message(layer_height),
            );
        }

        let initial_layer_height = self.scalar_f64_or_default("initial_layer_print_height")?;
        if initial_layer_height <= 0.0 {
            errors.insert(
                "initial_layer_print_height".to_owned(),
                invalid_value_message(initial_layer_height),
            );
        }

        let filament_diameters = self.numeric_vector_or_default("filament_diameter")?;
        if filament_diameters.iter().any(|value| *value < 1.0) {
            errors.insert(
                "filament_diameter".to_owned(),
                invalid_value_message(serialize_numbers(&filament_diameters)),
            );
        }

        let nozzle_diameters = self.numeric_vector_or_default("nozzle_diameter")?;
        if nozzle_diameters.iter().any(|value| *value < 0.005) {
            errors.insert(
                "nozzle_diameter".to_owned(),
                invalid_value_message(serialize_numbers(&nozzle_diameters)),
            );
        }

        for key in ["wall_loops", "top_shell_layers", "bottom_shell_layers"] {
            let value = self.scalar_i64_or_default(key)?;
            if value < 0 {
                errors.insert(key.to_owned(), invalid_value_message(value));
            }
        }

        Ok(errors)
    }
}
