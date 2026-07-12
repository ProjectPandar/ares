use serde_json::Value;

use crate::SliceError;

use super::SliceOptions;

impl SliceOptions {
    pub(crate) fn validate_slicing_different_extruders(&self) -> Result<(), SliceError> {
        if self.is_using_different_extruders()? {
            let _ = self.support_different_extruders()?;
            return Err(SliceError::InvalidInput(
                "different extruders are not supported by this Ares slicing boundary".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn is_using_different_extruders(&self) -> Result<bool, SliceError> {
        let nozzle_count = self.nozzle_diameters()?.len();
        if nozzle_count <= 1 {
            return Ok(false);
        }

        let Some(extruder_types) = optional_enum_vector(
            self.values().get("extruder_type"),
            "extruder_type",
            &["Direct Drive", "Bowden"],
        )?
        else {
            return Ok(false);
        };
        let Some(nozzle_volume_types) = optional_enum_vector(
            self.values().get("nozzle_volume_type"),
            "nozzle_volume_type",
            &["Standard", "High Flow"],
        )?
        else {
            return Ok(false);
        };

        let first_extruder_type = enum_get_at(&extruder_types, "extruder_type", 0)?;
        let first_nozzle_volume_type = enum_get_at(&nozzle_volume_types, "nozzle_volume_type", 0)?;

        for index in 1..nozzle_count {
            if enum_get_at(&extruder_types, "extruder_type", index)? != first_extruder_type
                || enum_get_at(&nozzle_volume_types, "nozzle_volume_type", index)?
                    != first_nozzle_volume_type
            {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

fn optional_enum_vector(
    value: Option<&Value>,
    key: &str,
    allowed: &[&str],
) -> Result<Option<Vec<String>>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be an enum vector")))?;
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!("{key} must not be empty")));
    }

    values
        .iter()
        .map(|value| {
            let text = value
                .as_str()
                .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))?;
            if allowed.contains(&text) {
                Ok(text.to_owned())
            } else {
                Err(SliceError::InvalidInput(format!(
                    "{key} has unknown enum value"
                )))
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn enum_get_at<'a>(values: &'a [String], key: &str, id: usize) -> Result<&'a str, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .map(String::as_str)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}
