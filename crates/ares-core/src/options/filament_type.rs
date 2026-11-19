use serde::Serialize;
use serde_json::Value;

use crate::SliceError;

use super::SliceOptions;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FilamentTypeDisplay {
    pub value: String,
    pub displayed: String,
}

impl SliceOptions {
    pub fn filament_vendor(&self) -> Result<String, SliceError> {
        first_string_entry(self.values().get("filament_vendor"), "filament_vendor")
    }

    pub fn filament_type(&self) -> Result<String, SliceError> {
        first_string_entry(self.values().get("filament_type"), "filament_type")
    }

    pub fn filament_type_display(&self, id: usize) -> Result<FilamentTypeDisplay, SliceError> {
        let Some(filament_type) =
            optional_string_get_at(self.values().get("filament_type"), "filament_type", id)?
        else {
            return Ok(display("", ""));
        };

        let is_support = optional_bool_get_at(
            self.values().get("filament_is_support"),
            "filament_is_support",
            id,
        )?
        .unwrap_or(false);

        if !is_support {
            return Ok(display(&filament_type, &filament_type));
        }

        if let Some(filament_id) =
            optional_string_get_at(self.values().get("filament_id"), "filament_id", id)?
        {
            match filament_id.as_str() {
                "GFS00" => return Ok(display("PLA-S", "Sup.PLA")),
                "GFS01" => return Ok(display("PA-S", "Sup.PA")),
                _ => {}
            }
        }

        Ok(match filament_type.as_str() {
            "PLA" => display("PLA-S", "Sup.PLA"),
            "PA" => display("PA-S", "Sup.PA"),
            _ => display(&filament_type, &filament_type),
        })
    }

}

fn first_string_entry(value: Option<&Value>, key: &str) -> Result<String, SliceError> {
    let Some(value) = value else {
        return Ok(String::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?;
    let Some(first) = values.first() else {
        return Ok(String::new());
    };
    first
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))
}

fn display(value: &str, displayed: &str) -> FilamentTypeDisplay {
    FilamentTypeDisplay {
        value: value.to_owned(),
        displayed: displayed.to_owned(),
    }
}

fn optional_string_get_at(
    value: Option<&Value>,
    key: &str,
    id: usize,
) -> Result<Option<String>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a string vector")))?;
    source_get_at(values, key, id)?
        .as_str()
        .map(|text| Some(text.to_owned()))
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain strings")))
}

fn optional_bool_get_at(
    value: Option<&Value>,
    key: &str,
    id: usize,
) -> Result<Option<bool>, SliceError> {
    let Some(value) = value else { return Ok(None) };
    let values = value
        .as_array()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a bool vector")))?;
    source_get_at(values, key, id)?
        .as_bool()
        .map(Some)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must contain bools")))
}

fn source_get_at<'a>(values: &'a [Value], key: &str, id: usize) -> Result<&'a Value, SliceError> {
    values
        .get(id)
        .or_else(|| values.first())
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))
}
