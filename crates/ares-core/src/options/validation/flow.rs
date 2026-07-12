use std::collections::BTreeMap;

use crate::SliceError;

use super::super::SliceOptions;
use super::helpers::{invalid_float_value_message, invalid_value_message, serialize_numbers};

impl SliceOptions {
    pub fn validate_skirt_and_bridge_flow_options(
        &self,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        let skirt_height = self.scalar_i64_or_default("skirt_height")?;
        let bridge_flow = self.scalar_f64_or_default("bridge_flow")?;
        let internal_bridge_flow = self.scalar_f64_or_default("internal_bridge_flow")?;

        if skirt_height < 0 {
            errors.insert(
                "skirt_height".to_owned(),
                invalid_value_message(skirt_height),
            );
        }

        if bridge_flow <= 0.0 {
            errors.insert(
                "bridge_flow".to_owned(),
                invalid_float_value_message(bridge_flow),
            );
            errors.insert(
                "internal_bridge_flow".to_owned(),
                invalid_float_value_message(internal_bridge_flow),
            );
        }

        Ok(errors)
    }

    pub fn validate_filament_flow_ratio_options(
        &self,
    ) -> Result<BTreeMap<String, String>, SliceError> {
        let mut errors = BTreeMap::new();
        let filament_flow_ratio = self.numeric_vector_or_default("filament_flow_ratio")?;

        if filament_flow_ratio.iter().any(|value| *value <= 0.0) {
            errors.insert(
                "filament_flow_ratio".to_owned(),
                invalid_value_message(serialize_numbers(&filament_flow_ratio)),
            );
        }

        Ok(errors)
    }
}
