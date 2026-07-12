use serde_json::Value;

use super::{SliceOptions, registry, vector_resize};
use crate::SliceError;

impl SliceOptions {
    pub fn set_num_extruders(&mut self, num_extruders: usize) -> Result<(), SliceError> {
        self.extend_extruder_variant(num_extruders)?;

        for key in registry::extruder_option_keys() {
            if *key == "default_filament_profile" {
                continue;
            }
            let target_size = self.parameter_size(key, num_extruders)?;
            let default = vector_resize::default_array_member(key)?;
            match self.values.get_mut(*key) {
                Some(Value::Array(values)) => {
                    vector_resize::resize_array(values, target_size, default)
                }
                Some(_) => return Err(SliceError::InvalidInput(format!("{key} must be an array"))),
                None => {
                    self.values
                        .insert((*key).to_owned(), Value::Array(vec![default; target_size]));
                }
            }
        }

        Ok(())
    }
}
