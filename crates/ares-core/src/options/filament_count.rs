use serde_json::Value;

use super::{SliceOptions, registry, vector_resize};
use crate::SliceError;

impl SliceOptions {
    pub fn set_num_filaments(&mut self, num_filaments: usize) -> Result<(), SliceError> {
        for key in registry::filament_option_keys() {
            if *key == "default_filament_profile" {
                continue;
            }
            let default = vector_resize::default_array_member(key)?;
            match self.values.get_mut(*key) {
                Some(Value::Array(values)) => {
                    vector_resize::resize_array(values, num_filaments, default)
                }
                Some(_) => return Err(SliceError::InvalidInput(format!("{key} must be an array"))),
                None => {
                    self.values.insert(
                        (*key).to_owned(),
                        Value::Array(vec![default; num_filaments]),
                    );
                }
            }
        }

        Ok(())
    }
}
