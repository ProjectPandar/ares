use super::SliceOptions;

use crate::SliceError;

const BEFORE_LAYER_CHANGE_GCODE_KEY: &str = "before_layer_change_gcode";
const DEFAULT_BEFORE_LAYER_CHANGE_GCODE: &str = "";
const CHANGE_FILAMENT_GCODE_KEY: &str = "change_filament_gcode";
const DEFAULT_CHANGE_FILAMENT_GCODE: &str = "";
const CHANGE_EXTRUSION_ROLE_GCODE_KEY: &str = "change_extrusion_role_gcode";
const DEFAULT_CHANGE_EXTRUSION_ROLE_GCODE: &str = "";
const FILAMENT_CHANGE_EXTRUSION_ROLE_GCODE_KEY: &str =
    "filament_change_extrusion_role_gcode";
const DEFAULT_FILAMENT_CHANGE_EXTRUSION_ROLE_GCODE: &str = "";
const FILAMENT_END_GCODE_KEY: &str = "filament_end_gcode";
const DEFAULT_FILAMENT_END_GCODE: &str = "";
const FILAMENT_START_GCODE_KEY: &str = "filament_start_gcode";
const DEFAULT_FILAMENT_START_GCODE: &str = "";
const FILE_START_GCODE_KEY: &str = "file_start_gcode";
const DEFAULT_FILE_START_GCODE: &str = "";
const LAYER_CHANGE_GCODE_KEY: &str = "layer_change_gcode";
const DEFAULT_LAYER_CHANGE_GCODE: &str = "";
const MACHINE_END_GCODE_KEY: &str = "machine_end_gcode";
const DEFAULT_MACHINE_END_GCODE: &str = "";
const MACHINE_START_GCODE_KEY: &str = "machine_start_gcode";
const DEFAULT_MACHINE_START_GCODE: &str = "";
const PROCESS_CHANGE_EXTRUSION_ROLE_GCODE_KEY: &str = "process_change_extrusion_role_gcode";
const DEFAULT_PROCESS_CHANGE_EXTRUSION_ROLE_GCODE: &str = "";
const TIME_LAPSE_GCODE_KEY: &str = "time_lapse_gcode";
const DEFAULT_TIME_LAPSE_GCODE: &str = "";

impl SliceOptions {
    pub(crate) fn before_layer_change_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(BEFORE_LAYER_CHANGE_GCODE_KEY) else {
            return Ok(DEFAULT_BEFORE_LAYER_CHANGE_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(BEFORE_LAYER_CHANGE_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn change_extrusion_role_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(CHANGE_EXTRUSION_ROLE_GCODE_KEY) else {
            return Ok(DEFAULT_CHANGE_EXTRUSION_ROLE_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(CHANGE_EXTRUSION_ROLE_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn change_filament_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(CHANGE_FILAMENT_GCODE_KEY) else {
            return Ok(DEFAULT_CHANGE_FILAMENT_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(CHANGE_FILAMENT_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn process_change_extrusion_role_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self
            .values()
            .get(PROCESS_CHANGE_EXTRUSION_ROLE_GCODE_KEY)
        else {
            return Ok(DEFAULT_PROCESS_CHANGE_EXTRUSION_ROLE_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(PROCESS_CHANGE_EXTRUSION_ROLE_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn filament_change_extrusion_role_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(FILAMENT_CHANGE_EXTRUSION_ROLE_GCODE_KEY) else {
            return Ok(DEFAULT_FILAMENT_CHANGE_EXTRUSION_ROLE_GCODE);
        };
        match value {
            serde_json::Value::String(value) => Ok(value),
            serde_json::Value::Array(values) => {
                for value in values {
                    if !value.is_string() {
                        return Err(invalid(
                            FILAMENT_CHANGE_EXTRUSION_ROLE_GCODE_KEY,
                            "must be a string or string array",
                        ));
                    }
                }
                Ok(values.first().and_then(|value| value.as_str()).unwrap_or(""))
            }
            _ => Err(invalid(
                FILAMENT_CHANGE_EXTRUSION_ROLE_GCODE_KEY,
                "must be a string or string array",
            )),
        }
    }

    pub(crate) fn filament_end_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(FILAMENT_END_GCODE_KEY) else {
            return Ok(DEFAULT_FILAMENT_END_GCODE);
        };
        match value {
            serde_json::Value::String(value) => Ok(value),
            serde_json::Value::Array(values) => {
                for value in values {
                    if !value.is_string() {
                        return Err(invalid(
                            FILAMENT_END_GCODE_KEY,
                            "must be a string or string array",
                        ));
                    }
                }
                Ok(values.first().and_then(|value| value.as_str()).unwrap_or(""))
            }
            _ => Err(invalid(
                FILAMENT_END_GCODE_KEY,
                "must be a string or string array",
            )),
        }
    }

    pub(crate) fn filament_start_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(FILAMENT_START_GCODE_KEY) else {
            return Ok(DEFAULT_FILAMENT_START_GCODE);
        };
        match value {
            serde_json::Value::String(value) => Ok(value),
            serde_json::Value::Array(values) => {
                for value in values {
                    if !value.is_string() {
                        return Err(invalid(
                            FILAMENT_START_GCODE_KEY,
                            "must be a string or string array",
                        ));
                    }
                }
                Ok(values.first().and_then(|value| value.as_str()).unwrap_or(""))
            }
            _ => Err(invalid(
                FILAMENT_START_GCODE_KEY,
                "must be a string or string array",
            )),
        }
    }

    pub(crate) fn file_start_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(FILE_START_GCODE_KEY) else {
            return Ok(DEFAULT_FILE_START_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(FILE_START_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn layer_change_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(LAYER_CHANGE_GCODE_KEY) else {
            return Ok(DEFAULT_LAYER_CHANGE_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(LAYER_CHANGE_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn machine_end_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(MACHINE_END_GCODE_KEY) else {
            return Ok(DEFAULT_MACHINE_END_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(MACHINE_END_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn machine_start_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(MACHINE_START_GCODE_KEY) else {
            return Ok(DEFAULT_MACHINE_START_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(MACHINE_START_GCODE_KEY, "must be a string"))
    }

    pub(crate) fn time_lapse_gcode(&self) -> Result<&str, SliceError> {
        let Some(value) = self.values().get(TIME_LAPSE_GCODE_KEY) else {
            return Ok(DEFAULT_TIME_LAPSE_GCODE);
        };
        value
            .as_str()
            .ok_or_else(|| invalid(TIME_LAPSE_GCODE_KEY, "must be a string"))
    }
}

fn invalid(key: &str, reason: &str) -> SliceError {
    SliceError::InvalidInput(format!("{key} {reason}"))
}
