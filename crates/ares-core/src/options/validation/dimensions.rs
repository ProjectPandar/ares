use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;
use super::helpers::invalid_float_value_message;

impl SliceOptions {
    pub fn validate_extruder_clearance_options(
        &self,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();

        for key in [
            "extruder_clearance_radius",
            "extruder_clearance_height_to_rod",
            "extruder_clearance_height_to_lid",
            "nozzle_height",
        ] {
            let value = self.scalar_f64_or_default(key)?;
            if value <= 0.0 {
                errors.insert(key.to_owned(), invalid_float_value_message(value));
            }
        }

        Ok(errors)
    }
}
